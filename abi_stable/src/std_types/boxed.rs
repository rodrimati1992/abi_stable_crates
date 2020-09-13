/*!
Contains the ffi-safe equivalent of `std::boxed::Box`.
*/


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
    marker_type::NonOwningPhantom,
    pointer_trait::{
        CallReferentDrop,Deallocate, CanTransmuteElement,
        GetPointerKind,PK_SmartPointer,OwnedPointer,
    },
    traits::{IntoReprRust},
    sabi_types::{Constructor,MovePtr},
    std_types::utypeid::{UTypeId,new_utypeid},
    prefix_type::{PrefixTypeTrait,WithMetadata},
};

// #[cfg(test)]
#[cfg(all(test,not(feature="only_new_tests")))]
mod test;

mod private {
    use super::*;

    /**
Ffi-safe equivalent of Box<_>.

# Example

Declaring a recursive datatype.

```
use abi_stable::{
    std_types::{RBox,RString},
    StableAbi,
};

#[repr(u8)]
#[derive(StableAbi)]
enum Command{
    SendProduct{
        id:u64,
    },
    GoProtest{
        cause:RString,
        place:RString,
    },
    SendComplaint{
        cause:RString,
        website:RString,
    },
    WithMetadata{
        command:RBox<Command>,
        metadata:RString,
    },
}


```

    */
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct RBox<T> {
        data: *mut T,
        vtable: BoxVtable_Ref<T>,
        _marker: PhantomData<T>,
    }

    impl<T> RBox<T> {
        /// Constucts an `RBox<T>` from a value.
        ///
        /// # Example 
        ///
        /// ```
        /// use abi_stable::std_types::RBox;
        ///
        /// let baux=RBox::new(100);
        /// assert_eq!(*baux,100);
        ///
        /// ```
        pub fn new(value: T) -> Self {
            Box::new(value).piped(RBox::from_box)
        }
        /// Converts a `Box<T>` to an `RBox<T>`,reusing its heap allocation.
        ///
        /// # Example 
        ///
        /// ```
        /// use abi_stable::std_types::RBox;
        ///
        /// let baux=Box::new(200);
        /// let baux=RBox::from_box(baux);
        /// assert_eq!(*baux,200);
        ///
        /// ```
        pub fn from_box(p: Box<T>) -> RBox<T> {
            RBox {
                data: Box::into_raw(p),
                vtable: VTableGetter::<T>::LIB_VTABLE,
                _marker: PhantomData,
            }
        }

        /// Constructs a `Box<T>` from a `MovePtr<'_,T>`.
        ///
        /// # Example
        ///
        /// ```
        /// use std::mem::ManuallyDrop;
        /// 
        /// use abi_stable::{
        ///     pointer_trait::OwnedPointer,
        ///     sabi_types::RSmallBox,
        ///     std_types::RBox,
        /// };
        ///
        /// let b=RSmallBox::<_,[u8;1]>::new(77u8);
        /// let rbox:RBox<_>=b.in_move_ptr(|x| RBox::from_move_ptr(x) );
        /// 
        /// assert_eq!(*rbox,77);
        ///
        /// ```
        pub fn from_move_ptr(p: MovePtr<'_,T>) -> RBox<T> {
            MovePtr::into_rbox(p)
        }

        pub(super) fn data(&self) -> *mut T {
            self.data
        }
        pub(super) fn vtable<'a>(&self) -> BoxVtable_Ref<T> {
            self.vtable
        }

        #[allow(dead_code)]
        #[cfg(test)]
        pub(super) fn set_vtable_for_testing(&mut self) {
            self.vtable = VTableGetter::<T>::LIB_VTABLE_FOR_TESTING;
        }
    }
}

pub use self::private::RBox;

unsafe impl<T> GetPointerKind for RBox<T>{
    type Kind=PK_SmartPointer;
}

unsafe impl<T, O> CanTransmuteElement<O> for RBox<T> {
    type TransmutedPtr = RBox<O>;
}

impl<T> RBox<T> {
    /// Converts this `RBox<T>` into a `Box<T>`
    ///
    /// # Allocation
    ///
    /// If this is invoked outside of the dynamic library/binary that created the `RBox<T>`,
    /// it will allocate a new `Box<T>` and move the data into it.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RBox;
    ///
    /// let baux:RBox<u32>=RBox::new(200);
    /// let baux:Box<u32>=RBox::into_box(baux);
    /// assert_eq!(*baux,200);
    ///
    /// ```
    pub fn into_box(this: Self) -> Box<T> {
        let this = ManuallyDrop::new(this);

        unsafe {
            let this_vtable =this.vtable();
            let other_vtable= VTableGetter::LIB_VTABLE;
            if ::std::ptr::eq(this_vtable.0.to_raw_ptr(), other_vtable.0.to_raw_ptr())||
                this_vtable.type_id()==other_vtable.type_id()
            {
                Box::from_raw(this.data())
            } else {
                let ret = Box::new(this.data().read());
                // Just deallocating the Box<_>. without dropping the inner value
                (this.vtable().destructor())(
                    this.data() as *mut (),
                    CallReferentDrop::No,Deallocate::Yes
                );
                ret
            }
        }
    }
    /// Unwraps this `Box<T>` into the value it owns on the heap.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RBox;
    ///
    /// let baux:RBox<u32>=RBox::new(200);
    /// let baux:u32=RBox::into_inner(baux);
    /// assert_eq!(baux,200);
    ///
    /// ```
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
    #[inline]
    unsafe fn get_move_ptr(this:&mut ManuallyDrop<Self>)->MovePtr<'_,Self::Target>{
        MovePtr::new(&mut **this)
    }

    #[inline]
    unsafe fn drop_allocation(this:&mut ManuallyDrop<Self>){
        unsafe {
            let data: *mut T = this.data();
            (this.vtable().destructor())(data as *mut (), CallReferentDrop::No,Deallocate::Yes);
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
            (RBox::vtable(self).destructor())(data as *mut (), CallReferentDrop::Yes,Deallocate::Yes);
        }
    }
}

///////////////////////////////////////////////////////////////

#[derive(StableAbi)]
#[repr(C)]
#[sabi(kind(Prefix))]
#[sabi(missing_field(panic))]
pub(crate) struct BoxVtable<T> {
    type_id:Constructor<UTypeId>,
    #[sabi(last_prefix_field)]
    destructor: unsafe extern "C" fn(*mut (), CallReferentDrop,Deallocate),
    _marker: NonOwningPhantom<T>,
}

struct VTableGetter<'a, T>(&'a T);

impl<'a, T: 'a> VTableGetter<'a, T> {
    const DEFAULT_VTABLE:BoxVtable<T>=BoxVtable{
        type_id:Constructor( new_utypeid::<RBox<()>> ),
        destructor: destroy_box::<T>,
        _marker: NonOwningPhantom::NEW,
    };

    // The VTABLE for this type in this executable/library
    const LIB_VTABLE: BoxVtable_Ref<T> = unsafe{
        BoxVtable_Ref(
            WithMetadata::new(
                PrefixTypeTrait::METADATA,
                Self::DEFAULT_VTABLE,
            ).as_prefix()
        )
    };

    #[allow(dead_code)]
    #[cfg(test)]
    const LIB_VTABLE_FOR_TESTING: BoxVtable_Ref<T> = unsafe{
        BoxVtable_Ref(
            WithMetadata::new(
                PrefixTypeTrait::METADATA,
                BoxVtable {
                    type_id:Constructor( new_utypeid::<RBox<i32>> ),
                    ..Self::DEFAULT_VTABLE
                },
            ).as_prefix()
        )
    };
}

unsafe extern "C" fn destroy_box<T>(ptr: *mut (), call_drop: CallReferentDrop,dealloc:Deallocate) {
    extern_fn_panic_handling! {no_early_return;
        let ptr = ptr as *mut T;
        if let CallReferentDrop::Yes=call_drop {
            ptr::drop_in_place(ptr);
        }
        if let Deallocate::Yes=dealloc {
            Box::from_raw(ptr as *mut ManuallyDrop<T>);
        }
    }
}

/////////////////////////////////////////////////////////////////
