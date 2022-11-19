//! Utility functions.

use std::{
    cmp::Ord,
    fmt::{self, Debug, Display},
    mem::{self, ManuallyDrop},
    ptr::NonNull,
};

use core_extensions::{strings::LeftPadder, StringExt, TypeIdentity};

use crate::{
    sabi_types::RMut,
    std_types::{RStr, RString},
};

//////////////////////////////////////

/// Information about a panic, used in `ffi_panic_message`.
#[derive(Debug, Copy, Clone)]
pub struct PanicInfo {
    ///
    pub file: &'static str,
    ///
    pub line: u32,
}

/// Prints an error message for attempting to panic across the
/// ffi boundary and aborts the process.
#[inline(never)]
#[cold]
pub fn ffi_panic_message(info: &'static PanicInfo) -> ! {
    eprintln!("\nfile:{}\nline:{}", info.file, info.line);
    eprintln!("Attempted to panic across the ffi boundary.");
    eprintln!("Aborting to handle the panic...\n");
    std::process::exit(1);
}

//////////////////////////////////

/// Only used inside `PhantomData`,
/// workaround for `PhantomData<&mut T>` not being constructible in const fns.
#[repr(transparent)]
pub(crate) struct MutRef<'a, T>(&'a mut T);

unsafe impl<'a, T> crate::abi_stability::GetStaticEquivalent_ for MutRef<'a, T>
where
    T: crate::abi_stability::GetStaticEquivalent_,
{
    type StaticEquivalent = crate::abi_stability::GetStaticEquivalent<&'a mut T>;
}
unsafe impl<'a, T> crate::StableAbi for MutRef<'a, T>
where
    T: crate::StableAbi + 'a,
{
    type IsNonZeroType = crate::type_level::bools::True;

    const LAYOUT: &'static crate::type_layout::TypeLayout = <&'a mut T as crate::StableAbi>::LAYOUT;
}

//////////////////////////////////

/// Converts a `&T` to a `NonNull<T>`.
///
/// # Example
///
/// ```rust
/// use abi_stable::utils::ref_as_nonnull;
///
/// use std::ptr::NonNull;
///
/// const NUMBER: NonNull<u64> = ref_as_nonnull(&100);
///
/// ```
pub const fn ref_as_nonnull<T: ?Sized>(reference: &T) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(reference as *const T as *mut T) }
}

/// Casts a `&'a mut ManuallyDrop<T>` to `RMut<'a, T>`
pub fn manuallydrop_as_rmut<T>(this: &mut ManuallyDrop<T>) -> RMut<'_, T> {
    unsafe { RMut::new(this).transmute() }
}

/// Casts a `&'a mut ManuallyDrop<T>` to `*mut T`
pub fn manuallydrop_as_raw_mut<T>(this: &mut ManuallyDrop<T>) -> *mut T {
    this as *mut ManuallyDrop<T> as *mut T
}

//////////////////////////////////

#[doc(hidden)]
pub struct AbortBomb {
    pub fuse: &'static PanicInfo,
}

impl Drop for AbortBomb {
    fn drop(&mut self) {
        ffi_panic_message(self.fuse);
    }
}

//////////////////////////////////

/// Helper type for transmuting between `Copy` types
/// without adding any overhead in debug builds.
///
/// # Safety
///
/// Be aware that using this type is equivalent to using [`std::mem::transmute_copy`],
/// which doesn't check that `T` and `U` have the same size.
///
/// [`std::mem::transmute_copy`]: https://doc.rust-lang.org/std/mem/fn.transmute_copy.html
#[repr(C)]
pub union Transmuter<T: Copy, U: Copy> {
    ///
    pub from: T,
    ///
    pub to: U,
}

//////////////////////////////////

#[repr(C)]
pub(crate) union Dereference<'a, T> {
    pub ptr: *const T,
    pub reff: &'a T,
}

macro_rules! deref {
    ($ptr:expr) => {
        crate::utils::Dereference { ptr: $ptr }.reff
    };
}
pub(crate) use deref;

//////////////////////////////////

/// Helper type for transmuting non-Copy types without adding any overhead in debug builds.
///
#[doc(hidden)]
#[repr(C)]
pub union TransmuterMD<T, U> {
    pub from: ManuallyDrop<T>,
    pub to: ManuallyDrop<U>,
}

macro_rules! const_transmute {
    ($from:ty, $to:ty, $val:expr) => {
        $crate::pmr::ManuallyDrop::into_inner(
            $crate::utils::TransmuterMD::<$from, $to> {
                from: $crate::pmr::ManuallyDrop::new($val),
            }
            .to,
        )
    };
}

pub(crate) use const_transmute;

//////////////////////////////////

/// Leaks `value` into the heap, and returns a reference to it.
///
/// # Warning
///
/// You must be careful when calling this function,
/// since this leak is ignored by [miri](https://github.com/rust-lang/miri).
///
#[inline]
pub fn leak_value<'a, T>(value: T) -> &'a T
where
    T: 'a, // T: 'a is for the docs
{
    let x = Box::new(value);
    let leaked: &'a T = Box::leak(x);
    #[cfg(miri)]
    unsafe {
        crate::miri_static_root(leaked as *const T as *const u8);
    }
    leaked
}

/// Transmute a reference to another reference,
/// changing the referent's type.
///
/// # Safety
///
/// This has the same safety concerns that `std::mem::transmute` has, including that
/// `T` has to have an alignment and be compatible with `U`.
#[inline]
#[allow(clippy::needless_lifetimes)]
pub const unsafe fn transmute_reference<T, U>(ref_: &T) -> &U {
    unsafe { &*(ref_ as *const _ as *const U) }
}

/// Transmute a mutable reference to another mutable reference,
/// changing the referent's type.
///
/// # Safety
///
/// This has the same safety concerns that `std::mem::transmute` has, including that
/// `T` has to have an alignment and be compatible with `U`.
#[inline]
#[allow(clippy::needless_lifetimes)]
pub unsafe fn transmute_mut_reference<'a, T, U>(ref_: &'a mut T) -> &'a mut U {
    unsafe { &mut *(ref_ as *mut _ as *mut U) }
}

//////////////////////////////////////

#[allow(dead_code)]
pub(crate) fn min_by<T, F, K>(l: T, r: T, mut f: F) -> T
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    if f(&l) < f(&r) {
        l
    } else {
        r
    }
}

#[allow(dead_code)]
pub(crate) fn max_by<T, F, K>(l: T, r: T, mut f: F) -> T
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    if f(&l) > f(&r) {
        l
    } else {
        r
    }
}

#[doc(hidden)]
pub fn min_max_by<T, F, K>(l: T, r: T, mut f: F) -> (T, T)
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    if f(&l) < f(&r) {
        (l, r)
    } else {
        (r, l)
    }
}

//////////////////////////////////////

pub(crate) trait FmtPadding {
    fn display_pad<'a, T>(
        &'a mut self,
        padding: usize,
        v: &T,
    ) -> Result<LeftPadder<'a>, fmt::Error>
    where
        T: Display;

    fn debug_pad<'a, T>(&'a mut self, padding: usize, v: &T) -> Result<LeftPadder<'a>, fmt::Error>
    where
        T: Debug;
}

macro_rules! impl_fmt_padding {
    ($ty: ty) => {
        impl FmtPadding for $ty {
            fn display_pad<'a, T>(
                &'a mut self,
                padding: usize,
                v: &T,
            ) -> Result<LeftPadder<'a>, fmt::Error>
            where
                T: Display,
            {
                use std::fmt::Write;
                let this = self.as_type_mut();

                this.clear();

                writeln!(this, "{}", v)?;

                Ok(this.left_padder(padding))
            }

            fn debug_pad<'a, T>(
                &'a mut self,
                padding: usize,
                v: &T,
            ) -> Result<LeftPadder<'a>, fmt::Error>
            where
                T: Debug,
            {
                use std::fmt::Write;
                let this = self.as_type_mut();

                this.clear();

                writeln!(this, "{:#?}", v)?;

                Ok(this.left_padder(padding))
            }
        }
    };
}

impl_fmt_padding! { String }
impl_fmt_padding! { RString }

//////////////////////////////////////////////////////////////////////

/// Takes the contents out of a `ManuallyDrop<T>`.
///
/// # Safety
///
/// After this function is called `slot` will become uninitialized and
/// must not be read again.
pub unsafe fn take_manuallydrop<T>(slot: &mut ManuallyDrop<T>) -> T {
    unsafe { ManuallyDrop::take(slot) }
}

#[doc(hidden)]
#[inline(always)]
pub const fn assert_fnonce<F, R>(_: &F)
where
    F: FnOnce() -> R,
{
}

/// This function allows calculating the distance (in `T`s) from `from` to `to`.
///
/// This returns `None` if `from` has a higher address than `to`,
/// or if `T` is a zero sized type.
///
/// # Example
///
/// ```
/// use abi_stable::utils;
///
/// let arr = ["hello", "world", "foo", "bar", "baz"];
///
/// assert_eq!(utils::distance_from(&arr[0], &arr[0]), Some(0));
/// assert_eq!(utils::distance_from(&arr[0], &arr[4]), Some(4));
///
/// assert_eq!(utils::distance_from(&arr[4], &arr[0]), None);
///
/// ```
pub fn distance_from<T>(from: *const T, to: *const T) -> Option<usize> {
    (to as usize)
        .checked_sub(from as usize)?
        .checked_div(mem::size_of::<T>())
}

//////////////////////////////////////////////////////////////////////

#[doc(hidden)]
pub extern "C" fn get_type_name<T>() -> RStr<'static> {
    RStr::from(std::any::type_name::<T>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_from_() {
        let int_array = [0, 1, 2, 3, 4];
        let unit_array = [(), (), (), (), ()];

        for (ix, x) in int_array.iter().enumerate() {
            for (iy, y) in int_array.iter().enumerate() {
                if ix <= iy {
                    assert_eq!(distance_from(x, y), Some(iy - ix));
                } else {
                    assert_eq!(distance_from(x, y), None);
                }
            }
        }

        for x in &unit_array {
            for y in &unit_array {
                assert_eq!(distance_from(x, y), None);
            }
        }
    }
}
