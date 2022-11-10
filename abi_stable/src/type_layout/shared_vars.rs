use super::*;

use crate::abi_stability::ConstGeneric;

use std::{
    fmt::{self, Debug},
    slice,
};

////////////////////////////////////////////////////////////////////////////////

/// A few static slices that many types in the `type_layout` module contain ranges into,
/// requiring this type to be passed as a parameter.
#[repr(C)]
#[derive(StableAbi)]
pub struct SharedVars {
    mono: &'static MonoSharedVars,
    type_layouts: *const extern "C" fn() -> &'static TypeLayout,
    constants: *const ConstGeneric,
    type_layouts_len: u16,
    constants_len: u16,
}

unsafe impl Sync for SharedVars {}
unsafe impl Send for SharedVars {}

impl SharedVars {
    /// Constructs a `SharedVars`.
    pub const fn new(
        mono: &'static MonoSharedVars,
        type_layouts: RSlice<'static, extern "C" fn() -> &'static TypeLayout>,
        constants: RSlice<'static, ConstGeneric>,
    ) -> Self {
        Self {
            mono,

            type_layouts: type_layouts.as_ptr(),
            type_layouts_len: type_layouts.len() as u16,

            constants: constants.as_ptr(),
            constants_len: constants.len() as u16,
        }
    }

    /// A string containing many strings that types in the `type_layout`
    /// module store substrings inside of.
    #[inline]
    pub fn strings(&self) -> &'static str {
        self.mono.strings()
    }
    /// Many lifetimes that types in the `type_layout` module reference.
    #[inline]
    pub fn lifetime_indices(&self) -> &'static [LifetimeIndexPair] {
        self.mono.lifetime_indices()
    }
    /// Many type layouts that types in the `type_layout`
    /// module reference.
    ///
    /// The `StableAbi` derive macro deduplicates identical looking types
    /// when constructing SharedVars.
    #[inline]
    pub fn type_layouts(&self) -> &'static [extern "C" fn() -> &'static TypeLayout] {
        unsafe { slice::from_raw_parts(self.type_layouts, self.type_layouts_len as usize) }
    }
    /// Many constants that types in the `type_layout` module contain ranges into.
    #[inline]
    pub fn constants(&self) -> &'static [ConstGeneric] {
        unsafe { slice::from_raw_parts(self.constants, self.constants_len as usize) }
    }
}

impl Debug for SharedVars {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedVars").finish()
    }
}

/// A few static slices that many types in the `type_layout` module contain ranges into,
/// requiring this type to be passed as a parameter.
#[repr(C)]
#[derive(StableAbi, Copy, Clone)]
pub struct MonoSharedVars {
    /// Many strings,separated with ";".
    strings: *const u8,
    /// Stores the lifetime indices for lifetimes referenced in a type.
    ///
    /// Note that this only stores those indices if the type references more than 3 lifetimes,
    /// otherwise it is stored in the range itself.
    ///
    /// Lifetimes indices are stored for these in order:
    ///
    /// - For fields
    ///
    /// - For parameters and return types in function pointers in fields.
    ///
    lifetime_indices: *const LifetimeIndexPair,
    strings_len: u16,
    lifetime_indices_len: u16,
}

impl MonoSharedVars {
    /// Constructs a `MonoSharedVars`.
    pub const fn new(
        strings: RStr<'static>,
        lifetime_indices: RSlice<'static, LifetimeIndexPairRepr>,
    ) -> Self {
        Self {
            strings: strings.as_ptr(),
            strings_len: strings.len() as u16,

            lifetime_indices: lifetime_indices.as_ptr() as *const LifetimeIndexPairRepr
                as *const LifetimeIndexPair,
            lifetime_indices_len: lifetime_indices.len() as u16,
        }
    }

    /// A string that types in the `type_layout` module store substrings inside of.
    #[inline]
    pub fn strings(&self) -> &'static str {
        unsafe {
            let slice = slice::from_raw_parts(self.strings, self.strings_len as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }

    /// Many lifetimes that types in the `type_layout` module reference.
    #[inline]
    pub fn lifetime_indices(&self) -> &'static [LifetimeIndexPair] {
        unsafe { slice::from_raw_parts(self.lifetime_indices, self.lifetime_indices_len as usize) }
    }
}
