use super::*;

////////////////////////////////////////////////////////////////////////////////

/// The parts of TLData that don't change based on generic parameters.
#[repr(u8)]
#[derive(Copy, Clone, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum MonoTLData {
    ///
    Primitive(TLPrimitive),
    /// A type that's only compared for size and alignment.
    Opaque,
    ///
    Struct {
        ///
        fields: CompTLFields,
    },
    ///
    Union {
        ///
        fields: CompTLFields,
    },
    ///
    Enum(MonoTLEnum),
    ///
    PrefixType(MonoTLPrefixType),
}

impl MonoTLData {
    /// Teh `MonoTLData` for an empty struct.
    pub const EMPTY: Self = MonoTLData::Struct {
        fields: CompTLFields::EMPTY,
    };

    /// Constructs `MonoTLData::Struct` from a slice of its fields.
    pub const fn struct_(fields: RSlice<'static, CompTLField>) -> Self {
        MonoTLData::Struct {
            fields: CompTLFields::from_fields(fields),
        }
    }

    #[doc(hidden)]
    pub const fn derive_struct(fields: CompTLFields) -> Self {
        MonoTLData::Struct { fields }
    }

    /// Constructs `MonoTLData::Union` from a slice of its fields.
    pub const fn union_(fields: RSlice<'static, CompTLField>) -> Self {
        MonoTLData::Union {
            fields: CompTLFields::from_fields(fields),
        }
    }

    #[doc(hidden)]
    pub const fn derive_union(fields: CompTLFields) -> Self {
        MonoTLData::Union { fields }
    }

    /// Constructs a `MonoTLData::PrefixType`
    pub const fn prefix_type(
        first_suffix_field: usize,
        conditional_prefix_fields: FieldConditionality,
        fields: RSlice<'static, CompTLField>,
    ) -> Self {
        MonoTLData::PrefixType(MonoTLPrefixType {
            first_suffix_field: first_suffix_field as u8,
            conditional_prefix_fields,
            fields: CompTLFields::from_fields(fields),
        })
    }

    /// Constructs `MonoTLData::Struct` from a slice of its fields.
    pub const fn struct_derive(fields: CompTLFields) -> Self {
        MonoTLData::Struct { fields }
    }

    /// Constructs `MonoTLData::Union` from a slice of its fields.
    pub const fn union_derive(fields: CompTLFields) -> Self {
        MonoTLData::Union { fields }
    }

    /// Constructs a `MonoTLData::PrefixType`
    pub const fn prefix_type_derive(
        first_suffix_field: usize,
        conditional_prefix_fields: u64,
        fields: CompTLFields,
    ) -> Self {
        MonoTLData::PrefixType(MonoTLPrefixType {
            first_suffix_field: first_suffix_field as u8,
            conditional_prefix_fields: FieldConditionality::from_u64(conditional_prefix_fields),
            fields,
        })
    }

    /// Converts this into a `TLDataDiscriminant`,
    /// allowing one to query which discriminant this is.
    pub const fn as_discriminant(&self) -> TLDataDiscriminant {
        match self {
            MonoTLData::Primitive { .. } => TLDataDiscriminant::Primitive,
            MonoTLData::Opaque { .. } => TLDataDiscriminant::Opaque,
            MonoTLData::Struct { .. } => TLDataDiscriminant::Struct,
            MonoTLData::Union { .. } => TLDataDiscriminant::Union,
            MonoTLData::Enum { .. } => TLDataDiscriminant::Enum,
            MonoTLData::PrefixType { .. } => TLDataDiscriminant::PrefixType,
        }
    }

    pub(super) const fn to_primitive(self) -> Option<TLPrimitive> {
        match self {
            MonoTLData::Primitive(x) => Some(x),
            _ => None,
        }
    }

    /// Expands this `MonoTLData`.
    ///
    /// # Errors
    ///
    /// This returns a `MismatchedTLDataVariant` if `self` and `generic`
    /// are variant of different names.
    pub fn expand(
        self,
        generic: GenericTLData,
        shared_vars: &'static SharedVars,
    ) -> Result<TLData, MismatchedTLDataVariant> {
        Ok(match (self, generic) {
            (MonoTLData::Primitive(prim), GenericTLData::Primitive) => TLData::Primitive(prim),
            (MonoTLData::Opaque, GenericTLData::Opaque) => TLData::Opaque,
            (MonoTLData::Struct { fields }, GenericTLData::Struct) => TLData::Struct {
                fields: fields.expand(shared_vars),
            },
            (MonoTLData::Union { fields }, GenericTLData::Union) => TLData::Union {
                fields: fields.expand(shared_vars),
            },
            (MonoTLData::Enum(nongeneric), GenericTLData::Enum(generic)) => {
                TLData::Enum(nongeneric.expand(generic, shared_vars))
            }
            (MonoTLData::PrefixType(nongeneric), GenericTLData::PrefixType(generic)) => {
                TLData::PrefixType(nongeneric.expand(generic, shared_vars))
            }
            _ => {
                return Err(MismatchedTLDataVariant {
                    nongeneric: self.as_discriminant(),
                    generic: generic.as_discriminant(),
                })
            }
        })
    }
}

///////////////////////////

/// An error returned by `MonoTLData::expand` because
/// the `GenericTLData` it tried to combine itself with was a different variant.
#[derive(Debug, Clone)]
pub struct MismatchedTLDataVariant {
    nongeneric: TLDataDiscriminant,
    generic: TLDataDiscriminant,
}

impl Display for MismatchedTLDataVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Error combining TLData::{:?} and GenericTLData::{:?}",
            self.nongeneric, self.generic,
        )
    }
}

///////////////////////////

/// A discriminant-only version of TLData.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum TLDataDiscriminant {
    ///
    Primitive,
    /// A type that's only compared for size and alignment.
    Opaque,
    ///
    Struct,
    ///
    Union,
    ///
    Enum,
    ///
    PrefixType,
}

/////////////////////////////////////////////////////

/// The part of TLData that can change based on generic parameters.
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum GenericTLData {
    ///
    Primitive,
    /// A type that's only compared for size and alignment.
    Opaque,
    ///
    Struct,
    ///
    Union,
    ///
    Enum(GenericTLEnum),
    ///
    PrefixType(GenericTLPrefixType),
}

impl GenericTLData {
    /// Converts this into a `TLDataDiscriminant`,
    /// allowing one to query which discriminant this is.
    pub const fn as_discriminant(&self) -> TLDataDiscriminant {
        match self {
            GenericTLData::Primitive { .. } => TLDataDiscriminant::Primitive,
            GenericTLData::Opaque { .. } => TLDataDiscriminant::Opaque,
            GenericTLData::Struct { .. } => TLDataDiscriminant::Struct,
            GenericTLData::Union { .. } => TLDataDiscriminant::Union,
            GenericTLData::Enum { .. } => TLDataDiscriminant::Enum,
            GenericTLData::PrefixType { .. } => TLDataDiscriminant::PrefixType,
        }
    }

    #[doc(hidden)]
    pub const fn prefix_type_derive(accessible_fields: FieldAccessibility) -> Self {
        GenericTLData::PrefixType(GenericTLPrefixType { accessible_fields })
    }
}

/////////////////////////////////////////////////////

/// The interior of the type definition,
/// describing whether the type is a primitive/enum/struct/union and its contents.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TLData {
    /// Types defined in the compiler.
    Primitive(TLPrimitive),
    /// The type can't be inspected,and has no properties other than size/alignment.
    ///
    /// When translated to C,this would be a struct with a single array field
    /// whose element type is the alignment in this layout,
    /// with the same byte length as this layout .
    Opaque,
    /// For structs.
    Struct {
        ///
        fields: TLFields,
    },
    /// For unions.
    Union {
        ///
        fields: TLFields,
    },
    /// For enums.
    Enum(TLEnum),
    /// vtables and modules that can be extended in minor versions.
    PrefixType(TLPrefixType),
}

impl Display for TLData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TLData::Primitive(prim) => {
                writeln!(f, "Primitive:{:?}", prim)?;
            }
            TLData::Opaque => {
                writeln!(f, "Opaque data")?;
            }
            TLData::Struct { fields } => {
                writeln!(
                    f,
                    "Struct with Fields:\n{}",
                    fields.to_string().left_padder(4)
                )?;
            }
            TLData::Union { fields } => {
                writeln!(
                    f,
                    "Union with Fields:\n{}",
                    fields.to_string().left_padder(4)
                )?;
            }
            TLData::Enum(enum_) => {
                writeln!(f, "Enum:")?;
                Display::fmt(enum_, f)?;
            }
            TLData::PrefixType(prefix) => {
                writeln!(f, "Prefix type:")?;
                Display::fmt(prefix, f)?;
            }
        }
        Ok(())
    }
}
