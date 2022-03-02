use super::*;

use crate::const_utils::{min_u16, min_u8};

use std::ops::{Range, RangeInclusive};

////////////////////////////////////////////////////////////////////////////////

/// The start and length of a slice into `TLFunctions`.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, StableAbi)]
pub struct StartLen {
    bits: u32,
}

/// The internal representation of `StartLen`.
pub type StartLenRepr = u32;

impl StartLen {
    /// Constructs a range.
    #[inline]
    pub const fn new(start: u16, len: u16) -> Self {
        Self {
            bits: (start as u32) | ((len as u32) << 16),
        }
    }

    /// Gets the start of the range.
    #[inline]
    pub const fn start(self) -> u16 {
        self.bits as u16
    }
    /// Gets the length of the range.
    #[inline]
    pub const fn len(self) -> u16 {
        (self.bits >> 16) as u16
    }

    /// Whether the range is empty.
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Gets the start of the range as a usize.
    #[inline]
    pub const fn start_usize(self) -> usize {
        (self.bits & 0xffff) as usize
    }
    #[inline]
    /// Gets the length of the range as a usize.
    pub const fn len_usize(self) -> usize {
        (self.bits >> 16) as usize
    }
    /// Gets the exclusive end of the range as a usize.
    #[inline]
    pub const fn end_usize(self) -> usize {
        self.start_usize() + self.len_usize()
    }

    /// Converts this range to a `std::ops::Range`.
    #[inline]
    pub const fn to_range(self) -> Range<usize> {
        self.start_usize()..self.end_usize()
    }

    /// Constructs this `StartLen` from its internal representation.
    #[inline]
    pub const fn from_u32(n: StartLenRepr) -> Self {
        Self { bits: n }
    }

    /// An empty range.
    pub const EMPTY: Self = Self::new(0, 0);

    abi_stable_shared::declare_start_len_bit_methods! {}
}

/// Used to convert the arguments passed to the `tl_genparams` macro to a `StartLen`.
pub struct StartLenConverter<T>(pub T);

#[allow(clippy::wrong_self_convention)]
impl StartLenConverter<()> {
    /// Constructs an empty `StartLen`.
    pub const fn to_start_len(self) -> StartLen {
        StartLen::EMPTY
    }
}

#[allow(clippy::wrong_self_convention)]
impl StartLenConverter<usize> {
    /// Constructs a `StartLen` from `0` to `self.0` exclusive.
    pub const fn to_start_len(self) -> StartLen {
        StartLen::new(self.0 as u16, 1)
    }
}

#[allow(clippy::wrong_self_convention)]
impl StartLenConverter<Range<usize>> {
    /// Constructs a `StartLen` from the `Range`.
    pub const fn to_start_len(self) -> StartLen {
        let start = self.0.start as u16;
        let len = (self.0.end - self.0.start) as u16;
        StartLen::new(start, len)
    }
}

#[allow(clippy::wrong_self_convention)]
impl StartLenConverter<RangeInclusive<usize>> {
    /// Constructs a `StartLen` from the `RangeInclusive`.
    pub const fn to_start_len(self) -> StartLen {
        let start = *self.0.start();
        let end = *self.0.end() + 1;
        StartLen::new(start as u16, (end - start) as u16)
    }
}

#[allow(clippy::wrong_self_convention)]
impl StartLenConverter<StartLen> {
    /// Unwraps this back into a `StartLen`.
    pub const fn to_start_len(self) -> StartLen {
        self.0
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

/// An optional u16 which represents None as `u16::max_value()`
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, StableAbi)]
pub struct OptionU16(u16);

impl OptionU16 {
    /// Equivalent to Option::None
    #[allow(non_upper_case_globals)]
    pub const None: Self = OptionU16(!0);

    const MAX_VAL: u16 = !0 - 1;

    /// Constructs the equivalent of `Some(value)`,
    /// which saturates the `u16::max_value()` value down to `u16::max_value()-1`
    pub const fn some(value: u16) -> Self {
        OptionU16(min_u16(value, Self::MAX_VAL))
    }

    /// Const equivalent of `OptionU16 == OptionU16`
    pub const fn eq(self, other: Self) -> bool {
        self.0 == other.0
    }

    /// Const equivalent of `OptionU16 != OptionU16`
    pub const fn ne(self, other: Self) -> bool {
        self.0 != other.0
    }

    /// Whether this is the Some variant.
    pub const fn is_some(self) -> bool {
        self.ne(Self::None)
    }
    /// Whether this is the None variant.
    pub const fn is_none(self) -> bool {
        self.eq(Self::None)
    }

    /// Converts this to an `Option<u16>`.
    pub const fn to_option(self) -> Option<u16> {
        if self.is_some() {
            Some(self.0)
        } else {
            None
        }
    }
}

impl Debug for OptionU16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.to_option(), f)
    }
}

impl Display for OptionU16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_some() {
            Display::fmt("None", f)
        } else {
            Display::fmt(&self.0, f)
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

/// An optional u8 which represents None as `u8::max_value()`
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, StableAbi)]
pub struct OptionU8(u8);

impl OptionU8 {
    /// Equivalent to Option::None
    #[allow(non_upper_case_globals)]
    pub const None: Self = OptionU8(!0);

    const MAX_VAL: u8 = !0 - 1;

    /// Constructs the equivalent of `Some(value)`,
    /// which saturates the `u8::max_value()` value down to `u8::max_value()-1`
    pub const fn some(value: u8) -> Self {
        OptionU8(min_u8(value, Self::MAX_VAL))
    }

    /// Const equivalent of `OptionU8 == OptionU8`
    pub const fn eq(self, other: Self) -> bool {
        self.0 == other.0
    }

    /// Const equivalent of `OptionU8 != OptionU8`
    pub const fn ne(self, other: Self) -> bool {
        self.0 != other.0
    }

    /// Whether this is the Some variant.
    pub const fn is_some(self) -> bool {
        self.ne(Self::None)
    }
    /// Whether this is the None variant.
    pub const fn is_none(self) -> bool {
        self.eq(Self::None)
    }

    /// Converts this to an `Option<u8>`.
    pub const fn to_option(self) -> Option<u8> {
        if self.is_some() {
            Some(self.0)
        } else {
            None
        }
    }
}

impl Debug for OptionU8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.to_option(), f)
    }
}

impl Display for OptionU8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_some() {
            Display::fmt("None", f)
        } else {
            Display::fmt(&self.0, f)
        }
    }
}
