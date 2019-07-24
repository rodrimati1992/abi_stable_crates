use std::{
    borrow::{Borrow,BorrowMut},
    marker::PhantomData, 
    mem::ManuallyDrop, 
    ops::DerefMut,
    ptr,
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    pointer_trait::{
        CallReferentDrop,Deallocate, TransmuteElement,
        GetPointerKind,PK_SmartPointer,OwnedPointer,
    },
    traits::{IntoReprRust},
    sabi_types::{MovePtr,ReturnValueEquality},
    std_types::utypeid::{UTypeId,new_utypeid},
    prefix_type::{PrefixTypeTrait,WithMetadata},
};

// #[cfg(test)]
#[cfg(all(test,not(feature="only_new_tests")))]
mod test;

mod private {
    use super::*;

    /// Ffi-safe equivalent of Box<_>.
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct RBox<T> {
        data: *mut T,
        vtable: *const BoxVtable<T>,
        _marker: PhantomData<T>,
    }

    impl<T> RBox<T> {
        /// Constucts an `RBox<T>` from a value.
        pub fn new(value: T) -> Self {
            Box::new(value).piped(RBox::from_box)
        }
        /// Converts a `Box<T>` to an `RBox<T>`,reusing its heap allocation.
        pub fn from_box(p: Box<T>) -> RBox<T> {
            RBox {
                data: Box::into_raw(p),
                vtable: VTableGetter::<T>::LIB_VTABLE.as_prefix_raw(),
                _marker: PhantomData,
            }
        }

        /// Constructs a `Box<T>` from a `MovePtr<'_,T>`.
        pub fn from_move_ptr(p: MovePtr<'_,T>) -> RBox<T> {
            p.into_rbox()
        }

        pub(super) fn data(&self) -> *mut T {
            self.data
        }
        pub(super) fn vtable<'a>(&self) -> &'a BoxVtable<T> {
            unsafe { &*self.vtable }
        }

        #[cfg(test)]
        pub(super) fn set_vtable_for_testing(&mut self) {
            self.vtable = VTableGetter::<T>::LIB_VTABLE_FOR_TESTING.as_prefix_raw();
        }
    }
}

pub use self::private::RBox;

unsafe impl<T> GetPointerKind for RBox<T>{
    type Kind=PK_SmartPointer;
}

unsafe impl<T, O> TransmuteElement<O> for RBox<T> {
    type TransmutedPtr = RBox<O>;
}

impl<T> RBox<T> {
    /// Converts this `RBox<T>` into a `Box<T>`
    ///
    /// # Allocation
    ///
    /// If this is invoked outside of the dynamic library/binary that created the `RBox<T>`,
    /// it will allocate a new `Box<T>` and move the data into it.
    pub fn into_box(this: Self) -> Box<T> {
        let this = ManuallyDrop::new(this);

        unsafe {
            let this_vtable =this.vtable();
            let other_vtable=VTableGetter::LIB_VTABLE.as_prefix();
            if ::std::ptr::eq(this_vtable,other_vtable)||
                this_vtable.type_id()==other_vtable.type_id()
            {
                Box::from_raw(this.data())
            } else {
                let ret = Box::new(this.data().read());
                // Just deallocating the Box<_>. without dropping the inner value
                (this.vtable().destructor())(this.data(), CallReferentDrop::No,Deallocate::Yes);
                ret
            }
        }
    }
    /// Unwraps this `Box<T>` into the value it owns on the heap.
    pub fn into_inner(this: Self) -> T {
        unsafe {
            let value = this.data().read();
            Self::drop_allocation(&mut ManuallyDrop::new(this));
            value
        }
    }
}

impl<T> DerefMut for RBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data() }
    }
}

/////////////////////////////////////////////////////////////////



unsafe impl<T> OwnedPointer for RBox<T>{
    type Target=T;

    #[inline]
    unsafe fn get_move_ptr(this:&mut ManuallyDrop<Self>)->MovePtr<'_,Self::Target>{
        MovePtr::new(&mut **this)
    }

    #[inline]
    unsafe fn drop_allocation(this:&mut ManuallyDrop<Self>){
        unsafe {
            let data: *mut T = this.data();
            (this.vtable().destructor())(data, CallReferentDrop::No,Deallocate::Yes);
        }
    }
}


/////////////////////////////////////////////////////////////////


impl<T> Borrow<T> for RBox<T>{
    fn borrow(&self)->&T{
        self
    }
}


impl<T> BorrowMut<T> for RBox<T>{
    fn borrow_mut(&mut self)->&mut T{
        self
    }
}


impl<T> AsRef<T> for RBox<T>{
    fn as_ref(&self)->&T{
        self
    }
}


impl<T> AsMut<T> for RBox<T>{
    fn as_mut(&mut self)->&mut T{
        self
    }
}


/////////////////////////////////////////////////////////////////

impl_from_rust_repr! {
    impl[T] From<Box<T>> for RBox<T> {
        fn(this){
            RBox::from_box(this)
        }
    }
}

/////////////////////////////////////////////////////////////////

impl<T> IntoReprRust for RBox<T> {
    type ReprRust = Box<T>;

    fn into_rust(self) -> Self::ReprRust {
        Self::into_box(self)
    }
}

/////////////////////////////////////////////////////////////////

impl<T> Default for RBox<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Clone for RBox<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        (**self).clone().piped(Box::new).into()
    }
}

shared_impls! {pointer
    mod=box_impls
    new_type=RBox[][T],
    original_type=Box,
}

unsafe impl<T: Send> Send for RBox<T> {}
unsafe impl<T: Sync> Sync for RBox<T> {}

///////////////////////////////////////////////////////////////

impl<T> Drop for RBox<T> {
    fn drop(&mut self) {
        unsafe {
            let data = self.data();
            (RBox::vtable(self).destructor())(data, CallReferentDrop::Yes,Deallocate::Yes);
        }
    }
}

///////////////////////////////////////////////////////////////

#[derive(StableAbi)]
#[repr(C)]
#[sabi(kind(Prefix(prefix_struct="BoxVtable")))]
#[sabi(missing_field(panic))]
pub(crate) struct BoxVtableVal<T> {
    type_id:ReturnValueEquality<UTypeId>,
    #[sabi(last_prefix_field)]
    destructor: unsafe extern "C" fn(*mut T, CallReferentDrop,Deallocate),
}

struct VTableGetter<'a, T>(&'a T);

impl<'a, T: 'a> VTableGetter<'a, T> {
    const DEFAULT_VTABLE:BoxVtableVal<T>=BoxVtableVal{
        type_id:ReturnValueEquality{
            function:new_utypeid::<RBox<()>>
        },
        destructor: destroy_box::<T>,
    };

    // The VTABLE for this type in this executable/library
    const LIB_VTABLE: &'a WithMetadata<BoxVtableVal<T>> = 
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::DEFAULT_VTABLE);

    #[cfg(test)]
    const LIB_VTABLE_FOR_TESTING: &'a WithMetadata<BoxVtableVal<T>> = 
        &WithMetadata::new(
            PrefixTypeTrait::METADATA,
            BoxVtableVal {
                type_id:ReturnValueEquality{
                    function: new_utypeid::<RBox<i32>>
                },
                ..Self::DEFAULT_VTABLE
            }
        );
}

unsafe extern "C" fn destroy_box<T>(ptr: *mut T, call_drop: CallReferentDrop,dealloc:Deallocate) {
    extern_fn_panic_handling! {no_early_return;
        if let CallReferentDrop::Yes=call_drop {
            ptr::drop_in_place(ptr);
        }
        if let Deallocate::Yes=dealloc {
            Box::from_raw(ptr as *mut ManuallyDrop<T>);
        }
    }
}

/////////////////////////////////////////////////////////////////
