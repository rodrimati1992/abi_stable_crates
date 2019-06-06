use std::{
    borrow::{Borrow},
    marker::PhantomData, 
    mem::ManuallyDrop, 
    sync::Arc,
};

use core_extensions::prelude::*;

use crate::{
    abi_stability::StableAbi,
    pointer_trait::{
        CallReferentDrop, StableDeref, TransmuteElement,
        GetPointerKind,PK_SmartPointer,
    },
    std_types::{RResult},
    std_types::utypeid::{UTypeId,new_utypeid},
    return_value_equality::ReturnValueEquality,
};

#[cfg(all(test,not(feature="only_new_tests")))]
mod test;

mod private {
    use super::*;

    /// Ffi-safe version of ::std::sync::Arc<_>
    #[derive(StableAbi)]
    #[repr(C)]
    pub struct RArc<T> {
        data: *const T,
        // This is a pointer instead of a static reference only because
        // the compiler complains that T doesn't live for the static lifetime,
        // even though ArcVtable<T> doesn't contain any T.
        vtable: *const ArcVtable<T>,
        _marker: PhantomData<T>,
    }

    impl_from_rust_repr! {
        impl[T] From<Arc<T>> for RArc<T> {
            fn(this){
                let out = RArc {
                    data: Arc::into_raw(this),
                    vtable: VTableGetter::LIB_VTABLE.as_prefix_raw(),
                    _marker: Default::default(),
                };
                out
            }
        }
    }

    unsafe impl<T> StableDeref for RArc<T> {}

    unsafe impl<T> GetPointerKind for RArc<T>{
        type Kind=PK_SmartPointer;
    }

    unsafe impl<T, O> TransmuteElement<O> for RArc<T> {
        type TransmutedPtr = RArc<O>;
    }

    impl<T> RArc<T> {
        #[inline(always)]
        pub(super) fn data(&self) -> *const T {
            self.data
        }
        
        #[inline(always)]
        pub(super) unsafe fn data_mut(&mut self) -> *mut T {
            self.data as *mut T
        }

        #[inline]
        pub(crate) fn into_raw(self) -> *const T {
            let this = ManuallyDrop::new(self);
            this.data
        }

        #[inline(always)]
        pub(crate) fn vtable<'a>(&self) -> &'a ArcVtable<T> {
            unsafe { &*self.vtable }
        }

        #[cfg(test)]
        pub(super) fn set_vtable_for_testing(&mut self) {
            self.vtable = VTableGetter::LIB_VTABLE_FOR_TESTING.as_prefix_raw();
        }
    }
}

pub use self::private::RArc;

impl<T> RArc<T> {
    pub fn new(this: T) -> Self {
        Arc::new(this).into()
    }
}

impl<T> RArc<T> {

    /// Converts this into an `Arc<T>`
    ///
    /// # Allocators
    ///
    /// The reason `RArc<T>` cannot always be converted to an `Arc<T>` is 
    /// because their allocators might be different.
    ///
    /// # When is T cloned
    ///
    /// `T` is cloned if the current dynamic_library/executable is 
    /// not the one that created the `RArc<T>`,
    /// and the strong count is greater than 1.
    pub fn into_arc(this: Self) -> Arc<T>
    where
        T: Clone,
    {
        let this_vtable =this.vtable();
        let other_vtable=VTableGetter::LIB_VTABLE.as_prefix();
        if ::std::ptr::eq(this_vtable,other_vtable)||
            this_vtable.type_id()==other_vtable.type_id()
        {
            unsafe { Arc::from_raw(this.into_raw()) }
        } else {
            Self::try_unwrap(this)
                .unwrap_or_else(|x| T::clone(&x))
                .piped(Arc::new)
        }
    }

    /// Attempts to unwrap this `RArc<T>` into a `T`,
    /// returns Err(self) if the `RArc<T>`s strong count is greater than 1.
    #[inline]
    pub fn try_unwrap(this: Self) -> Result<T, Self>{
        let vtable = this.vtable();
        (vtable.try_unwrap())(this).into_result()
    }

    /// Attempts to create a mutable reference to `T`,
    /// failing if `RArc<T>`s strong count is greater than 1.
    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T>{
        let vtable = this.vtable();
        (vtable.get_mut())(this)
    }

    /// Makes a mutable reference to `T`,
    /// if there are other `RArc<T>`s pointing to the same value,
    /// then `T` is cloned to ensure unique ownership of the value.
    ///
    /// # Postconditions
    ///
    /// After this call, the strong count of `this` will be 1,
    /// because either it was 1 before the call,
    /// or because a new `RArc<T>` was created to ensure unique ownership of `T`.
    #[inline]
    pub fn make_mut<'a>(this: &'a mut Self) -> &'a mut T 
    where T:Clone
    {
        // Workaround for non-lexical lifetimes not being smart enough 
        // to figure out that this borrow doesn't continue in the None branch.
        let unbounded_this=unsafe{ &mut *(this as *mut Self) };
        match Self::get_mut(unbounded_this) {
            Some(x)=>x,
            None=>{
                let new_arc=RArc::new((**this).clone());
                *this=new_arc;
                // This is fine,since this is a freshly created arc with a clone of the data.
                unsafe{
                    &mut *this.data_mut()
                }
            }
        }
    }
    
}


////////////////////////////////////////////////////////////////////


impl<T> Borrow<T> for RArc<T>{
    fn borrow(&self)->&T{
        self
    }
}


impl<T> AsRef<T> for RArc<T>{
    fn as_ref(&self)->&T{
        self
    }
}


////////////////////////////////////////////////////////////////////

impl<T> Default for RArc<T>
where
    T: Default,
{
    fn default() -> Self {
        RArc::new(T::default())
    }
}

impl<T> Clone for RArc<T> {
    fn clone(&self) -> Self {
        (self.vtable().clone())(self)
    }
}

impl_into_rust_repr! {
    impl[T] Into<Arc<T>> for RArc<T>
    where[
        T: Clone+StableAbi,
    ]{
        fn(this){
            Self::into_arc(this)
        }
    }
}

impl<T> Drop for RArc<T> {
    fn drop(&mut self) {
        // The layout of the RArc<_> won't change since it doesn't
        // actually support ?Sized types.
        unsafe {
            let vtable = self.vtable();
            (vtable.destructor())((self.data() as *const T).into(), CallReferentDrop::Yes);
        }
    }
}

shared_impls! {pointer
    mod=arc_impls
    new_type=RArc[][T],
    original_type=Arc,
}

unsafe impl<T> Sync for RArc<T> where T: Send + Sync {}

unsafe impl<T> Send for RArc<T> where T: Send + Sync {}

/////////////////////////////////////////////////////////

mod vtable_mod {
    use super::*;
    use crate::prefix_type::{PrefixTypeTrait,WithMetadata};

    pub(super) struct VTableGetter<'a, T>(&'a T);

    impl<'a, T: 'a> VTableGetter<'a, T> {
        const DEFAULT_VTABLE:ArcVtableVal<T>=ArcVtableVal {
            type_id:ReturnValueEquality{
                function:new_utypeid::<RArc<()>>
            },
            destructor: destructor_arc::<T>,
            clone: clone_arc::<T>,
            get_mut: get_mut_arc::<T>,
            try_unwrap: try_unwrap_arc::<T>,
        };

        // The VTABLE for this type in this executable/library
        pub(super) const LIB_VTABLE: &'a WithMetadata<ArcVtableVal<T>> = {
            &WithMetadata::new(PrefixTypeTrait::METADATA,Self::DEFAULT_VTABLE)
        };

        #[cfg(test)]
        pub(super) const LIB_VTABLE_FOR_TESTING: &'a WithMetadata<ArcVtableVal<T>> = {
            &WithMetadata::new(
                PrefixTypeTrait::METADATA,
                ArcVtableVal{
                    type_id:ReturnValueEquality{
                        function:new_utypeid::<RArc<i32>>
                    },
                    ..Self::DEFAULT_VTABLE
                }
            )
        };
    }

    #[derive(StableAbi)]
    #[repr(C)]
    #[sabi(kind(Prefix(prefix_struct="ArcVtable")))]
    #[sabi(missing_field(panic))]
    pub struct ArcVtableVal<T> {
        pub(super) type_id:ReturnValueEquality<UTypeId>,
        pub(super) destructor: unsafe extern "C" fn(*const T, CallReferentDrop),
        pub(super) clone: extern "C" fn(&RArc<T>) -> RArc<T>,
        pub(super) get_mut: extern "C" fn(&mut RArc<T>) -> Option<&mut T>,
        #[sabi(last_prefix_field)]
        pub(super) try_unwrap:extern "C" fn(RArc<T>) -> RResult<T, RArc<T>>,
    }

}
use self::vtable_mod::{ArcVtable, VTableGetter};

unsafe extern "C" fn destructor_arc<T>(this: *const T, call_drop: CallReferentDrop) {
    extern_fn_panic_handling! {
        if call_drop == CallReferentDrop::Yes {
            drop(Arc::from_raw(this));
        } else {
            drop(Arc::from_raw(this as *const ManuallyDrop<T>));
        }
    }
}

extern "C" fn clone_arc<T>(this: &RArc<T>) -> RArc<T> {
    unsafe {
        this.data()
            .piped(|x| Arc::from_raw(x))
            .piped(ManuallyDrop::new)
            .piped(|x| Arc::clone(&x))
            .into()
    }
}

extern "C" fn get_mut_arc<'a,T>(this: &'a mut RArc<T>) -> Option<&'a mut T> {
    unsafe {
        let arc=Arc::from_raw(this.data());
        let mut arc=ManuallyDrop::new(arc);
        // This is fine,since we are only touching the data afterwards,
        // which is guaranteed to have the 'a lifetime.
        let arc:&'a mut Arc<T>=&mut *(&mut *arc as *mut Arc<T>);
        Arc::get_mut(arc)
    }
}

extern "C" fn try_unwrap_arc<T>(this: RArc<T>) -> RResult<T, RArc<T>> {
    unsafe {
        this.into_raw()
            .piped(|x| Arc::from_raw(x))
            .piped(Arc::try_unwrap)
            .map_err(RArc::from)
            .into()
    }
}
