use std::{marker::PhantomData, mem::ManuallyDrop, sync::Arc};

use core_extensions::prelude::*;

use crate::{
    abi_stability::StableAbi,
    pointer_trait::{CallReferentDrop, StableDeref, TransmuteElement},
    std_types::{RResult},
};

#[cfg(test)]
mod test;

mod private {
    use super::*;

    /// Ffi-safe version of ::std::sync::Arc<_>
    #[derive(StableAbi)]
    #[repr(C)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(shared_stable_abi(T))]
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
                    vtable: VTableGetter::LIB_VTABLE as *const ArcVtable<T>,
                    _marker: Default::default(),
                };
                out
            }
        }
    }

    unsafe impl<T> StableDeref for RArc<T> {}

    unsafe impl<T, O> TransmuteElement<O> for RArc<T> {
        type TransmutedPtr = RArc<O>;
    }

    impl<T> RArc<T> {
        #[inline(always)]
        pub(super) fn data(&self) -> *const T {
            self.data
        }

        #[inline]
        pub(crate) fn into_raw(self) -> *const T {
            let this = ManuallyDrop::new(self);
            this.data
        }

        #[inline(always)]
        pub fn vtable<'a>(&self) -> &'a ArcVtable<T> {
            unsafe { &*self.vtable }
        }

        #[cfg(test)]
        pub(super) fn set_vtable_for_testing(&mut self) {
            self.vtable = VTableGetter::LIB_VTABLE_FOR_TESTING as *const ArcVtable<T>;
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
    pub fn into_arc(this: Self) -> Arc<T>
    where
        T: Clone + StableAbi,
    {
        if ::std::ptr::eq(this.vtable(), VTableGetter::LIB_VTABLE) {
            unsafe { Arc::from_raw(this.into_raw()) }
        } else {
            Self::try_unwrap(this)
                .unwrap_or_else(|x| T::clone(&x))
                .piped(Arc::new)
        }
    }

    pub fn try_unwrap(this: Self) -> Result<T, Self>
    where
        T: StableAbi,
    {
        let vtable = this.vtable();
        vtable.try_unwrap.get()(this).into()
    }
}

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
        (self.vtable().clone)(self)
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
            (vtable.destructor)((self.data() as *const T).into(), CallReferentDrop::Yes);
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

    pub(super) struct VTableGetter<'a, T>(&'a T);

    impl<'a, T: 'a> VTableGetter<'a, T> {
        // The VTABLE for this type in this executable/library
        pub(super) const LIB_VTABLE: &'a ArcVtable<T> = &ArcVtable {
            destructor: destructor_arc::<T>,
            clone: clone_arc::<T>,
            try_unwrap: TryUnwrap {
                func: try_unwrap_arc::<T>,
            },
        };

        #[cfg(test)]
        pub(super) const LIB_VTABLE_FOR_TESTING: &'a ArcVtable<T> = &ArcVtable {
            destructor: destructor_arc_for_testing,
            ..*Self::LIB_VTABLE
        };
    }

    #[derive(StableAbi)]
    #[repr(C)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(shared_stable_abi(T))]
    pub struct ArcVtable<T> {
        pub(super) destructor: unsafe extern "C" fn(*const T, CallReferentDrop),
        pub(super) clone: extern "C" fn(&RArc<T>) -> RArc<T>,
        // For this to be safe,we must ensure that
        // T:StableAbi whenever this is called.
        #[sabi(unsafe_opaque_field)]
        pub(super) try_unwrap: TryUnwrap<T>,
    }

    #[derive(StableAbi)]
    #[repr(transparent)]
    #[sabi(inside_abi_stable_crate)]
    pub(super) struct TryUnwrap<T> {
        func: extern "C" fn(RArc<T>) -> RResult<T, RArc<T>>,
    }

    impl<T> TryUnwrap<T>
    where
        T: StableAbi,
    {
        pub(super) fn get(self) -> extern "C" fn(RArc<T>) -> RResult<T, RArc<T>> {
            self.func
        }
    }
    impl<T> Copy for TryUnwrap<T> {}
    impl<T> Clone for TryUnwrap<T> {
        fn clone(&self) -> Self {
            *self
        }
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

#[cfg(test)]
unsafe extern "C" fn destructor_arc_for_testing<T>(this: *const T, call_drop: CallReferentDrop) {
    destructor_arc(this, call_drop)
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

extern "C" fn try_unwrap_arc<T>(this: RArc<T>) -> RResult<T, RArc<T>> {
    unsafe {
        this.into_raw()
            .piped(|x| Arc::from_raw(x))
            .piped(Arc::try_unwrap)
            .map_err(RArc::from)
            .into()
    }
}
