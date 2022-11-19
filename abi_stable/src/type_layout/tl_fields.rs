use super::*;

use std::{iter, slice};

/// The layout of all compressed fields in a type definition,
/// one can access the expanded fields by calling the expand method.
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct CompTLFields {
    /// All TLField fields which map 1:1.
    comp_fields: *const CompTLField,

    /// All the function pointer types in the field.
    functions: Option<&'static TLFunctions>,

    comp_fields_len: u16,
}

unsafe impl Sync for CompTLFields {}
unsafe impl Send for CompTLFields {}

impl CompTLFields {
    /// A `CompTLFields` with no fields.
    pub const EMPTY: Self = Self::from_fields(rslice![]);

    /// Constructs a `CompTLFields`.
    pub const fn new(
        comp_fields: RSlice<'static, CompTLFieldRepr>,
        functions: Option<&'static TLFunctions>,
    ) -> Self {
        Self {
            comp_fields: comp_fields.as_ptr() as *const CompTLFieldRepr as *const CompTLField,
            comp_fields_len: comp_fields.len() as u16,

            functions,
        }
    }

    /// Constructs a `CompTLFields` with fields,and without functions.
    pub const fn from_fields(comp_fields: RSlice<'static, CompTLField>) -> Self {
        Self {
            comp_fields: comp_fields.as_ptr(),
            comp_fields_len: comp_fields.len() as u16,

            functions: None,
        }
    }

    /// Accesses a slice of all the compressed fields in this `CompTLFields`.
    pub fn comp_fields(&self) -> &'static [CompTLField] {
        unsafe { slice::from_raw_parts(self.comp_fields, self.comp_fields_len as usize) }
    }

    /// Accesses a slice of all the compressed fields in this `CompTLFields`.
    pub const fn comp_fields_rslice(&self) -> RSlice<'static, CompTLField> {
        unsafe { RSlice::from_raw_parts(self.comp_fields, self.comp_fields_len as usize) }
    }

    /// Constructs an iterator over all the field names.
    pub fn field_names(
        &self,
        shared_vars: &MonoSharedVars,
    ) -> impl ExactSizeIterator<Item = &'static str> + Clone + 'static {
        let fields = self.comp_fields();
        let strings = shared_vars.strings();

        fields.iter().map(move |field| field.name(strings))
    }

    /// Gets the name of the nth field.
    pub fn get_field_name(
        &self,
        index: usize,
        shared_vars: &MonoSharedVars,
    ) -> Option<&'static str> {
        let strings = shared_vars.strings();

        self.comp_fields().get(index).map(|f| f.name(strings))
    }

    /// The amount of fields this represents
    pub const fn len(&self) -> usize {
        self.comp_fields_len as usize
    }

    /// Whether there are no fields.
    pub const fn is_empty(&self) -> bool {
        self.comp_fields_len == 0
    }

    /// Expands this into a TLFields,allowing access to expanded fields.
    pub const fn expand(self, shared_vars: &'static SharedVars) -> TLFields {
        TLFields {
            shared_vars,
            comp_fields: self.comp_fields_rslice(),
            functions: self.functions,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

/// The layout of all the fields in a type definition.
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
pub struct TLFields {
    shared_vars: &'static SharedVars,

    comp_fields: RSlice<'static, CompTLField>,

    /// All the function pointer types in the field.
    functions: Option<&'static TLFunctions>,
}

impl TLFields {
    /// Constructs a TLFields from the compressed fields,without any functions.
    pub const fn from_fields(
        comp_fields: &'static [CompTLField],
        shared_vars: &'static SharedVars,
    ) -> Self {
        Self {
            comp_fields: RSlice::from_slice(comp_fields),
            shared_vars,
            functions: None,
        }
    }

    /// The amount of fields this represents
    pub const fn len(&self) -> usize {
        self.comp_fields.len()
    }

    /// Whether this contains any fields
    pub const fn is_empty(&self) -> bool {
        self.comp_fields.is_empty()
    }

    /// Gets the ith expanded field.Returns None there is no ith field.
    pub fn get(&self, i: usize) -> Option<TLField> {
        self.comp_fields
            .get(i)
            .map(|field| field.expand(i, self.functions, self.shared_vars))
    }

    /// Gets an iterator over the expanded fields.
    pub fn iter(&self) -> TLFieldsIterator {
        TLFieldsIterator {
            shared_vars: self.shared_vars,
            comp_fields: self.comp_fields.as_slice().iter().enumerate(),
            functions: self.functions,
        }
    }

    /// Collects the expanded fields into a `Vec<TLField>`.
    pub fn to_vec(&self) -> Vec<TLField> {
        self.iter().collect()
    }
}

impl IntoIterator for TLFields {
    type IntoIter = TLFieldsIterator;
    type Item = TLField;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Debug for TLFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl Display for TLFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for field in self.iter() {
            Display::fmt(&field, f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Eq for TLFields {}
impl PartialEq for TLFields {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

///////////////////////////////////////////////////////////////////////////////

/// An iterator over all the fields in a type definition.
#[derive(Clone, Debug)]
pub struct TLFieldsIterator {
    shared_vars: &'static SharedVars,

    comp_fields: iter::Enumerate<slice::Iter<'static, CompTLField>>,

    /// All the function pointer types in the field.
    functions: Option<&'static TLFunctions>,
}

impl Iterator for TLFieldsIterator {
    type Item = TLField;

    fn next(&mut self) -> Option<TLField> {
        self.comp_fields
            .next()
            .map(|(i, field)| field.expand(i, self.functions, self.shared_vars))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.comp_fields.len();
        (len, Some(len))
    }
    fn count(self) -> usize {
        self.comp_fields.len()
    }
}

impl std::iter::ExactSizeIterator for TLFieldsIterator {}
