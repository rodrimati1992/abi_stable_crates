use crate::{std_types::RStr, type_layout::MonoTypeLayout};

/// Represents the layout of a prefix-type,for use in error messages.
#[repr(C)]
#[derive(Debug, Copy, Clone, StableAbi)]
// #[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct PTStructLayout {
    /// The stringified generic parameters.
    pub generics: RStr<'static>,
    /// The layout information of the type which doesn't depend on generic parameters
    pub mono_layout: &'static MonoTypeLayout,
}

//////////////////////////////////////////////////////////////

impl PTStructLayout {
    /// Constructs a `PTStructLayout`.
    pub const fn new(generics: RStr<'static>, mono_layout: &'static MonoTypeLayout) -> Self {
        Self {
            generics,
            mono_layout,
        }
    }

    /// Gets an iterator over the names of the fields.
    #[inline]
    pub fn get_field_names(&self) -> impl Iterator<Item = &'static str> {
        self.mono_layout.field_names()
    }

    /// Gets a `Vec` with the names of the fields.
    #[inline]
    pub fn get_field_names_vec(&self) -> Vec<&'static str> {
        self.mono_layout.field_names().collect()
    }

    /// Gets the name of the `ith` field, returning `None` if there is no `ith` field.
    #[inline]
    pub fn get_field_name(&self, ith: usize) -> Option<&'static str> {
        self.mono_layout.get_field_name(ith)
    }
}
