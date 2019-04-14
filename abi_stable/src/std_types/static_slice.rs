use std::{marker::PhantomData, mem, ops::Deref};

use crate::std_types::{RSlice};

pub use self::inner::StaticSlice;

// A type-level assertion that &[u8] is 2 usizes large.
type Assertions = [u8; {
    const USIZE_SIZE: usize = mem::size_of::<usize>();
    const SAME_SIZE: bool = 2 * USIZE_SIZE == mem::size_of::<&'static [u8]>();
    const SAME_ALIGN: bool = mem::align_of::<[usize; 2]>() == mem::align_of::<&'static [u8]>();
    ((SAME_SIZE & SAME_ALIGN) as usize) - 1
}];

mod inner {
    use super::*;

    /// Wrapper type around `&'static [T]` as a workaround for the
    /// non-stable-constness of `<[T]>::len`.
    ///
    /// Once `<[T]>::len` is stable in const contests define the RSlice::from_slice const fn
    /// so as to replace this type with RSlice<'static,T>.
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    pub struct StaticSlice<T: 'static> {
        #[sabi(unsafe_opaque_field)]
        s: &'static [T],
        #[sabi(unsafe_opaque_field)]
        conversion: RSliceFromStaticSlice<T>,
        _private_initializer: PhantomData<Assertions>,
    }

    impl<T> Copy for StaticSlice<T> {}
    impl<T> Clone for StaticSlice<T> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<T: 'static> StaticSlice<T> {
        /// Creates a `StaticSlice<T>` from a `&'static [T]`
        #[inline]
        pub const fn new(s: &'static [T]) -> Self {
            StaticSlice {
                s,
                conversion: RSliceFromStaticSlice::<T>::NEW,
                _private_initializer: PhantomData,
            }
        }
        /// Gets the `&'static [Y]` back.
        #[inline]
        pub fn as_slice(&self) -> &'static [T] {
            self.as_rslice().into()
        }
        /// Converts the internal `&'static [T]` into a `RSlice<'static,T>`.
        #[inline]
        pub fn as_rslice(&self) -> RSlice<'static, T> {
            unsafe {
                let s = (&self.s) as *const &'static [T] as *const [usize; 2];
                (self.conversion.conversion)(s)
            }
        }
    }

    //////////////////

    #[repr(transparent)]
    pub struct RSliceFromStaticSlice<T: 'static> {
        conversion: unsafe extern "C" fn(*const [usize; 2]) -> RSlice<'static, T>,
    }

    impl<T> Copy for RSliceFromStaticSlice<T> {}
    impl<T> Clone for RSliceFromStaticSlice<T> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<T: 'static> RSliceFromStaticSlice<T> {
        const NEW: Self = RSliceFromStaticSlice::<T> {
            conversion: slice_conversion,
        };
    }

}

unsafe extern "C" fn slice_conversion<T>(s: *const [usize; 2]) -> RSlice<'static, T> {
    let slice_: &'static [T] = *(s as *const &'static [T]);
    RSlice::from(slice_)
}

impl<T> Deref for StaticSlice<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

shared_impls! {
    mod=slice_impls
    new_type=StaticSlice[][T],
    original_type=T,
}
