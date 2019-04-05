use std::{
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::DerefMut,
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    pointer_trait::{CallReferentDrop, StableDeref, TransmuteElement},
    traits::FromElement,
    CAbi, IntoReprRust,
};

#[cfg(test)]
mod test;

mod private {
    use super::*;

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    pub struct RBox<T> {
        data: CAbi<*mut T>,
        vtable: CAbi<*const BoxVtable<T>>,
        _marker: PhantomData<T>,
    }

    impl<T> RBox<T> {
        pub fn new(this: T) -> Self {
            Box::new(this).piped(RBox::from_box)
        }
        pub fn from_box(p: Box<T>) -> RBox<T> {
            RBox {
                data: Box::into_raw(p).into(),
                vtable: (VTableGetter::<T>::LIB_VTABLE as *const BoxVtable<T>).into() ,
                _marker: PhantomData,
            }
        }

        pub(super) fn data(&self) -> *mut T {
            self.data.into_inner()
        }
        pub(super) fn vtable<'a>(&self) -> &'a BoxVtable<T> {
            unsafe { &*self.vtable.into_inner() }
        }

        #[cfg(test)]
        pub(super) fn set_vtable_for_testing(&mut self) {
            self.vtable = 
                (VTableGetter::<T>::LIB_VTABLE_FOR_TESTING as *const BoxVtable<T>).into();
        }
    }
}

pub use self::private::RBox;

unsafe impl<T> StableDeref for RBox<T> {}

unsafe impl<T, O> TransmuteElement<O> for RBox<T> {
    type TransmutedPtr = RBox<O>;
}

impl<T> RBox<T> {
    pub fn into_box(this: Self) -> Box<T> {
        let this = ManuallyDrop::new(this);

        unsafe {
            if ::std::ptr::eq(this.vtable(),VTableGetter::<T>::LIB_VTABLE) {
                Box::from_raw(this.data())
            } else {
                let ret = Box::new(this.data().read());
                // Just deallocating the Box<_>. without dropping the inner value
                (this.vtable().destructor)(this.data().into(), CallReferentDrop::No);
                ret
            }
        }
    }
    pub fn into_inner(this: Self) -> T {
        let this = ManuallyDrop::new(this);
        unsafe {
            let value = this.data().read();
            let data: CAbi<*mut T> = this.data().into();
            (this.vtable().destructor)(data, CallReferentDrop::No);
            value
        }
    }

    pub fn as_abi(&self) -> CAbi<&T> {
        CAbi::from(&**self)
    }

    pub fn as_abi_mut(&mut self) -> CAbi<&mut T> {
        CAbi::from(&mut **self)
    }
}

impl<T> DerefMut for RBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data() }
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

impl<T> FromElement for RBox<T> {
    type Element = T;

    #[inline]
    fn from_elem(val: Self::Element) -> Self {
        Self::new(val)
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
            let data: CAbi<*mut T> = self.data().into();
            (RBox::vtable(self).destructor)(data, CallReferentDrop::Yes);
        }
    }
}

///////////////////////////////////////////////////////////////

struct VTableGetter<'a, T>(&'a T);

impl<'a, T: 'a> VTableGetter<'a, T> {
    // The VTABLE for this type in this executable/library
    const LIB_VTABLE: &'a BoxVtable<T> = &BoxVtable {
        destructor: destroy_box::<T>,
    };

    #[cfg(test)]
    const LIB_VTABLE_FOR_TESTING: &'a BoxVtable<T> = &BoxVtable {
        destructor: destroy_box_for_tests,
        ..*VTableGetter::LIB_VTABLE
    };
}

#[derive(StableAbi)]
#[repr(C)]
#[sabi(inside_abi_stable_crate)]
pub struct BoxVtable<T> {
    destructor: unsafe extern "C" fn(CAbi<*mut T>, CallReferentDrop),
}


unsafe extern "C" fn destroy_box<T>(v: CAbi<*mut T>, call_drop: CallReferentDrop) {
    extern_fn_panic_handling!{
        let mut box_ = Box::from_raw(v.into_inner() as *mut ManuallyDrop<T>);
        if call_drop == CallReferentDrop::Yes {
            ManuallyDrop::drop(&mut *box_);
        }
        drop(box_);
    }
}

#[cfg(test)]
unsafe extern "C" fn destroy_box_for_tests<T>(v: CAbi<*mut T>, call_drop: CallReferentDrop) {
    destroy_box(v, call_drop);
}

/////////////////////////////////////////////////////////////////
