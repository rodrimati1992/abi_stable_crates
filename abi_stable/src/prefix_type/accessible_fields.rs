use crate::sabi_types::bitarray::{bool_to_enum, enum_to_bool, BitArray64, BooleanEnum};

////////////////////////////////////////////////////////////////////////////////

/// Whether a field is accessible.
#[derive(StableAbi, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum IsAccessible {
    ///
    No = 0,
    ///
    Yes = 1,
}

impl IsAccessible {
    /// Constructs an IsAccessible with a bool saying whether this is accessible.
    pub const fn new(is_accessible: bool) -> Self {
        bool_to_enum(is_accessible)
    }
    /// Describes whether this is accessible.
    pub const fn is_accessible(self) -> bool {
        enum_to_bool(self)
    }
}

unsafe impl BooleanEnum for IsAccessible {
    const FALSE: Self = Self::No;
    const TRUE: Self = Self::Yes;
}

/// An array with whether the ith field of a prefix-type
/// is accessible through its accessor method.
pub type FieldAccessibility = BitArray64<IsAccessible>;

////////////////////////////////////////////////////////////////////////////////

/// Whether a field is conditional,
/// whether it has a `#[sabi(accessible_if = expression)]` helper attribute or not.
#[derive(StableAbi, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum IsConditional {
    ///
    No = 0,
    ///
    Yes = 1,
}

impl IsConditional {
    /// Constructs an IsConditional with a bool saying this is conditional.
    pub const fn new(is_conditional: bool) -> Self {
        bool_to_enum(is_conditional)
    }
    /// Describes whether this is conditional.
    pub const fn is_conditional(self) -> bool {
        enum_to_bool(self)
    }
}

unsafe impl BooleanEnum for IsConditional {
    const FALSE: Self = Self::No;
    const TRUE: Self = Self::Yes;
}

/// An array with whether the ith field in the prefix of a prefix-type
/// is conditional,which means whether it has the
/// `#[sabi(accessible_if = expression)]` attribute applied to it.
pub type FieldConditionality = BitArray64<IsConditional>;
