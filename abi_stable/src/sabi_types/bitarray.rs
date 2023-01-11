//! A packed array of boolean enum values.

use crate::const_utils::low_bit_mask_u64;

use std::{
    fmt::{self, Debug},
    iter::ExactSizeIterator,
    marker::PhantomData,
};

#[cfg(all(test, not(feature = "only_new_tests")))]
mod tests;

/// An array of 64 binary enum values.
///
/// # Example
///
/// ```rust
/// use abi_stable::sabi_types::bitarray::{BitArray64, BooleanEnum};
///
/// assert!(SET.at(0).is_even());
/// assert!(SET.at(1).is_odd());
/// assert!(SET.at(20).is_even());
/// assert!(SET.at(21).is_odd());
///
///
///
///
/// static SET: BitArray64<IsEven> = {
///     let mut set = BitArray64::empty();
///
///     let mut i = 0;
///     while i < 64 {
///         set = set.set(i, IsEven::Yes);
///         i += 2;
///     }
///
///     set
/// };
///
///
/// #[derive(Debug, Copy, Clone)]
/// #[repr(u8)]
/// enum IsEven {
///     No,
///     Yes,
/// }
///
/// unsafe impl BooleanEnum for IsEven {
///     const FALSE: Self = Self::No;
///     const TRUE: Self = Self::Yes;
/// }
///
/// impl IsEven {
///     pub const fn is_even(self) -> bool {
///         matches!(self, IsEven::Yes)
///     }
///     pub const fn is_odd(self) -> bool {
///         matches!(self, IsEven::No)
///     }
/// }
///
/// ```
#[must_use = "BitArray64 is returned by value by every mutating method."]
#[derive(StableAbi, PartialEq, Eq)]
#[repr(transparent)]
pub struct BitArray64<E> {
    bits: u64,
    _marker: PhantomData<E>,
}

impl<E> Copy for BitArray64<E> {}
impl<E> Clone for BitArray64<E> {
    fn clone(&self) -> Self {
        Self {
            bits: self.bits,
            _marker: PhantomData,
        }
    }
}

impl<E> BitArray64<E> {
    /// Creates a BitArray64 where the first `count` elements are truthy.
    #[inline]
    pub const fn with_count(count: usize) -> Self
    where
        E: BooleanEnum,
    {
        Self {
            bits: low_bit_mask_u64(count as u32),
            _marker: PhantomData,
        }
    }

    /// Creates a BitArray64 from a u64.
    #[inline]
    pub const fn from_u64(bits: u64) -> Self {
        Self {
            bits,
            _marker: PhantomData,
        }
    }

    /// Creates a BitArray64 where all elements are falsy.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            bits: 0,
            _marker: PhantomData,
        }
    }
}

impl<E> BitArray64<E> {
    /// Gets the value of `E` at `index`
    ///
    /// # Panics
    ///
    /// This function panics if `index >= 64`
    ///
    pub const fn at(self, index: usize) -> E
    where
        E: BooleanEnum,
    {
        Self::assert_index(index);

        bool_to_enum((self.bits & (1u64 << index)) != 0)
    }

    /// Sets the value at `index` to `value`
    ///
    /// # Panics
    ///
    /// This function panics if `index >= 64`
    ///
    pub const fn set(mut self, index: usize, value: E) -> Self
    where
        E: BooleanEnum,
    {
        if enum_to_bool(value) {
            self.bits |= 1u64 << index;
        } else {
            self.bits &= !(1u64 << index);
        }
        self
    }

    #[track_caller]
    const fn assert_index(index: usize) {
        use const_panic::{concat_panic, FmtArg, PanicVal};

        if index >= 64 {
            concat_panic(&[&[
                PanicVal::write_str("index out of bounds: the length is "),
                PanicVal::from_usize(64, FmtArg::DEBUG),
                PanicVal::write_str(" but the index is "),
                PanicVal::from_usize(index, FmtArg::DEBUG),
            ]])
        }
    }
}

impl<E> BitArray64<E> {
    /// Truncates self so that only the first `length` elements are truthy.
    pub const fn truncated(mut self, length: usize) -> Self {
        self.bits &= low_bit_mask_u64(length as u32);
        self
    }

    /// Converts this array to its underlying representation
    #[inline]
    pub const fn bits(self) -> u64 {
        self.bits
    }

    /// An iterator over the first `count` elements of the array.
    #[allow(clippy::missing_const_for_fn)]
    pub fn iter(self) -> BitArray64Iter<E> {
        BitArray64Iter {
            count: 64,
            bits: self.bits(),
            _marker: PhantomData,
        }
    }

    /// Whether this array is equal to `other` up to the `count` element.
    pub fn eq(self, other: Self, count: usize) -> bool {
        let all_accessible = low_bit_mask_u64(count as u32);
        let implication = (!self.bits | other.bits) & all_accessible;
        println!(
            "self:{:b}\nother:{:b}\nall_accessible:{:b}\nimplication:{:b}",
            self.bits, other.bits, all_accessible, implication,
        );
        implication == all_accessible
    }
}

impl<E> Debug for BitArray64<E>
where
    E: BooleanEnum,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A trait for enums with two variants where one is `truthy` and the other one is `falsy`.
///
/// # Safety
///
/// This type must:
/// - be represented as a `u8`
/// - have exactly two possible values
/// - assign one value to the `FALSE` associated constant,
/// - assign another value to the `TRUE` associated constant
///
pub unsafe trait BooleanEnum: Debug + Copy + 'static {
    /// The falsy value of this type
    const FALSE: Self;
    /// The truthy value of this type
    const TRUE: Self;
}

const _: () = assert!(std::mem::size_of::<bool>() == 1);
unsafe impl BooleanEnum for bool {
    const FALSE: Self = false;
    const TRUE: Self = true;
}

/// Converts a bool to a [`BooleanEnum`].
///
/// Converts `true` to [`BooleanEnum::TRUE`],
/// and `false` to [`BooleanEnum::FALSE`],
///
pub const fn bool_to_enum<E>(b: bool) -> E
where
    E: BooleanEnum,
{
    if b {
        E::TRUE
    } else {
        E::FALSE
    }
}

/// Converts a [`BooleanEnum`] to a bool
///
/// Converts [`BooleanEnum::TRUE`] to `true`,
/// and [`BooleanEnum::FALSE`] to `false`,
///
pub const fn enum_to_bool<E>(b: E) -> bool
where
    E: BooleanEnum,
{
    enum_as_u8(b) == EnumConsts::<E>::TRUE_U8
}

const fn enum_as_u8<E: BooleanEnum>(x: E) -> u8 {
    // SAFETY: `BooleanEnum` requires `E` to be represented as a `u8`
    unsafe { const_transmute!(E, u8, x) }
}

struct EnumConsts<E: BooleanEnum>(E);

impl<E: BooleanEnum> EnumConsts<E> {
    const TRUE_U8: u8 = enum_as_u8(E::TRUE);
}

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////

/// Iterator over the enums inside a [`BitArray64`]
#[derive(Debug, Clone)]
pub struct BitArray64Iter<E> {
    count: usize,
    bits: u64,
    _marker: PhantomData<E>,
}

impl<E> BitArray64Iter<E>
where
    E: BooleanEnum,
{
    #[inline]
    fn next_inner<F>(&mut self, f: F) -> Option<E>
    where
        F: FnOnce(&mut Self) -> E,
    {
        if self.count == 0 {
            None
        } else {
            Some(f(self))
        }
    }
}
impl<E> Iterator for BitArray64Iter<E>
where
    E: BooleanEnum,
{
    type Item = E;

    fn next(&mut self) -> Option<E> {
        self.next_inner(|this| {
            this.count -= 1;
            let cond = (this.bits & 1) != 0;
            this.bits >>= 1;
            bool_to_enum(cond)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<E> DoubleEndedIterator for BitArray64Iter<E>
where
    E: BooleanEnum,
{
    fn next_back(&mut self) -> Option<E> {
        self.next_inner(|this| {
            this.count -= 1;
            bool_to_enum((this.bits & (1 << this.count)) != 0)
        })
    }
}

impl<E> ExactSizeIterator for BitArray64Iter<E>
where
    E: BooleanEnum,
{
    #[inline]
    fn len(&self) -> usize {
        self.count
    }
}
