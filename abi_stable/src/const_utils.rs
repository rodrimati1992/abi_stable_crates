/*!
Utilities for const contexts.
*/

use crate::std_types::RStr;

pub use abi_stable_shared::const_utils::low_bit_mask_u64;

//////////////////////////////////////////////////////////////////

// Used to test trait bounds in proc-macros.
#[doc(hidden)]
pub trait AssocStr {
    const STR: RStr<'static>;
}

macro_rules! impl_assoc_str {
    ( $($ty:ty),* ) => (
        $(
            impl AssocStr for $ty {
                const STR:RStr<'static>=RStr::from_str(stringify!( $ty ));
            }
        )*
    )
}

impl_assoc_str! { i8,i16,i32,i64,isize,u8,u16,u32,u64,usize }

//////////////////////////////////////////////////////////////////

// Used to test trait bounds in proc-macros.
#[doc(hidden)]
pub trait AssocInt {
    const NUM: usize;
}

macro_rules! impl_assoc_str {
    ( $($ty:ty=$val:expr),* $(,)* ) => (
        $(
            impl AssocInt for $ty {
                const NUM:usize=$val;
            }
        )*
    )
}

impl_assoc_str! {
    i8=0,i16=1,i32=2,i64=3,isize=4,
    u8=5,u16=6,u32=7,u64=8,usize=9,
}

//////////////////////////////////////

/// Creates an empty slice.
pub const fn empty_slice<'a, T>() -> &'a [T]
where
    T: 'a,
{
    GetEmptySlice::<'a, T>::EMPTY
}

struct GetEmptySlice<'a, T>(&'a T);

impl<'a, T> GetEmptySlice<'a, T>
where
    T: 'a,
{
    const EMPTY: &'a [T] = &[];
}

//////////////////////////////////////

/// The minimum of two `u64`s
pub const fn min_u8(l: u8, r: u8) -> u8 {
    [r, l][(l < r) as usize]
}

/// The minimum of two `u64`s
pub const fn min_u16(l: u16, r: u16) -> u16 {
    [r, l][(l < r) as usize]
}

/// The minimum of two `u64`s
pub const fn min_u64(l: u64, r: u64) -> u64 {
    [r, l][(l < r) as usize]
}

/// The minimum of two `usize`s
pub const fn min_usize(l: usize, r: usize) -> usize {
    [r, l][(l < r) as usize]
}

/// The maximum of two `u64`s
pub const fn max_u64(l: u64, r: u64) -> u64 {
    [l, r][(l < r) as usize]
}

/// The maximum of two `usize`s
pub const fn max_usize(l: usize, r: usize) -> usize {
    [l, r][(l < r) as usize]
}

/// The minimum and maximum of two `usize`s
pub const fn min_max_usize(l: usize, r: usize) -> (usize, usize) {
    [(r, l), (l, r)][(l < r) as usize]
}

//////////////////////////////////////

/// Gets the absolute value of a usize subtraction.
pub const fn abs_sub_usize(l: usize, r: usize) -> usize {
    let (min, max) = min_max_usize(l, r);
    max - min
}

//////////////////////////////////////

/// Saturating substraction of `usize`s.
pub const fn saturating_sub_usize(l: usize, r: usize) -> usize {
    let mask = -((r < l) as isize);
    l.wrapping_sub(r) & (mask as usize)
}

/// Saturating substraction of `u8`s.
pub const fn saturating_sub_u8(l: u8, r: u8) -> u8 {
    let mask = -((r < l) as i8);
    l.wrapping_sub(r) & (mask as u8)
}

/// The base 2 logarithm of a usize.
pub const fn log2_usize(n: usize) -> u8 {
    const USIZE_BITS: u8 = (std::mem::size_of::<usize>() * 8) as u8;
    saturating_sub_u8(USIZE_BITS - n.leading_zeros() as u8, 1) as u8
}

//////////////////////////////////////

/// Allows converting between `Copy` generic types that are the same concrete type.
///
/// # Safety
///
/// This is safe to do,
/// since both types are required to be the same concrete type inside the macro.
#[macro_export]
macro_rules! type_identity {
    ($from:ty=>$to:ty; $expr:expr ) => {
        unsafe {
            let _: $crate::pmr::AssertEq<$from, $to>;

            $crate::utils::Transmuter::<$from, $to> { from: $expr }.to
        }
    };
}

//////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    const USIZE_BITS: u8 = (std::mem::size_of::<usize>() * 8) as u8;

    #[test]
    fn abs_sub_test() {
        for p in 0..USIZE_BITS {
            let n = 1usize << p;
            assert_eq!(abs_sub_usize(0, n), n);
            assert_eq!(abs_sub_usize(n, 0), n);
        }
        assert_eq!(abs_sub_usize(4, 5), 1);
        assert_eq!(abs_sub_usize(5, 5), 0);
        assert_eq!(abs_sub_usize(5, 4), 1);
        assert_eq!(abs_sub_usize(5, 0), 5);
        assert_eq!(abs_sub_usize(0, 5), 5);
    }

    #[test]
    fn log2_usize_test() {
        assert_eq!(log2_usize(0), 0);
        assert_eq!(log2_usize(1), 0);
        for power in 1..USIZE_BITS {
            let n = 1usize << power;
            assert_eq!(log2_usize(n - 1), power - 1, "power:{} n:{}", power, n);
            assert_eq!(log2_usize(n), power, "power:{} n:{}", power, n);
            assert_eq!(log2_usize(n + 1), power, "power:{} n:{}", power, n);
        }
    }
}
