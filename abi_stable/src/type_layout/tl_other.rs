use super::*;

use crate::{abi_stability::ConstGeneric, sabi_types::Constructor};

/////////////////////////////////////////////////////

/// The `repr(..)` attribute used on a type.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum ReprAttr {
    /// This is an `Option<NonZeroType>`.
    /// In which the size and alignment of the `Option<_>` is exactly that of its contents.
    ///
    /// When translated to C,it is equivalent to the type parameter.
    OptionNonZero,
    /// This is an ffi-safe primitive type,declared in the compiler.
    Primitive,
    /// A struct whose fields are laid out like C,
    C,
    /// An enum with a `#[repr(C, IntegerType)]` attribute.
    CAndInt(DiscriminantRepr),
    /// A type with the same size,alignment and function ABI as
    /// its only non-zero-sized field.
    Transparent,
    /// Means that only `repr(IntegerType)` was used.
    Int(DiscriminantRepr),
    // Added just in case that I add support for it
    #[doc(hidden)]
    Packed {
        /// The alignment represented as a `1 << alignment_power_of_two`.
        alignment_power_of_two: u8,
    },
}

/////////////////////////////////////////////////////

/// A module path.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct ModPath(NulStr<'static>);

impl ModPath {
    /// An item without a path
    pub const NO_PATH: Self = ModPath(nulstr_trunc!("<no path>"));

    /// An item in the prelude.
    pub const PRELUDE: Self = ModPath(nulstr_trunc!("<prelude>"));

    /// Constructs a ModPath from a string with a module path.
    pub const fn inside(path: NulStr<'static>) -> Self {
        ModPath(path)
    }
}

impl Display for ModPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/////////////////////////////////////////////////////

/// The compressed generic parameters of a type,
/// which can be expanded into a `GenericParams` by calling `expand`.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct CompGenericParams {
    /// The names of the lifetimes declared by a type.
    lifetime: NulStr<'static>,
    /// The type parameters of a type,getting them from the containing TypeLayout.
    types: StartLen,
    /// The const parameters of a type,getting them from the containing TypeLayout.
    consts: StartLen,
    lifetime_count: u8,
}

impl CompGenericParams {
    /// Constructs a CompGenericParams.
    pub const fn new(
        lifetime: NulStr<'static>,
        lifetime_count: u8,
        types: StartLen,
        consts: StartLen,
    ) -> Self {
        Self {
            lifetime,
            lifetime_count,
            types,
            consts,
        }
    }

    /// Expands this `CompGenericParams` into a `GenericParams`.
    pub fn expand(self, shared_vars: &'static SharedVars) -> GenericParams {
        GenericParams {
            lifetime: self.lifetime,
            types: Constructor::wrap_slice(&shared_vars.type_layouts()[self.types.to_range()]),
            consts: &shared_vars.constants()[self.consts.to_range()],
            lifetime_count: self.lifetime_count,
        }
    }
}

/// The generic parameters of a type.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct GenericParams {
    /// The names of the lifetimes declared by a type.
    pub(super) lifetime: NulStr<'static>,
    /// The type parameters of a type,getting them from the containing TypeLayout.
    pub(super) types: &'static [Constructor<&'static TypeLayout>],
    /// The const parameters of a type,getting them from the containing TypeLayout.
    pub(super) consts: &'static [ConstGeneric],
    pub(super) lifetime_count: u8,
}

impl GenericParams {
    /// Whether this contains any generic parameters
    pub fn is_empty(&self) -> bool {
        self.lifetime.to_str().is_empty() && self.types.is_empty() && self.consts.is_empty()
    }

    /// Gets an iterator over the names of the lifetime parameters of the type.
    pub fn lifetimes(&self) -> impl Iterator<Item = &'static str> + Clone + Send + Sync + 'static {
        self.lifetime.to_str().split(',').filter(|x| !x.is_empty())
    }
    /// The amount of lifetimes of the type.
    pub const fn lifetime_count(&self) -> usize {
        self.lifetime_count as usize
    }
    /// The type parameters of the type.
    pub fn type_params(&self) -> &'static [extern "C" fn() -> &'static TypeLayout] {
        Constructor::unwrap_slice(self.types)
    }
    /// The const parameters of the type.
    pub const fn const_params(&self) -> &'static [ConstGeneric] {
        self.consts
    }
}

impl Display for GenericParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("<", f)?;

        let post_iter = |i: usize, len: usize, f: &mut Formatter<'_>| -> fmt::Result {
            if i + 1 < len {
                fmt::Display::fmt(", ", f)?;
            }
            Ok(())
        };

        for (i, param) in self.lifetimes().enumerate() {
            fmt::Display::fmt(param, &mut *f)?;
            post_iter(i, self.lifetime_count(), &mut *f)?;
        }
        for (i, param) in self.types.iter().cloned().enumerate() {
            fmt::Debug::fmt(&param.get().full_type(), &mut *f)?;
            post_iter(i, self.types.len(), &mut *f)?;
        }
        for (i, param) in self.consts.iter().enumerate() {
            fmt::Debug::fmt(param, &mut *f)?;
            post_iter(i, self.consts.len(), &mut *f)?;
        }
        fmt::Display::fmt(">", f)?;
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////

/// Types defined in the compiler
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum TLPrimitive {
    ///
    U8,
    ///
    I8,
    ///
    U16,
    ///
    I16,
    ///
    U32,
    ///
    I32,
    ///
    U64,
    ///
    I64,
    ///
    Usize,
    ///
    Isize,
    ///
    F32,
    ///
    F64,
    ///
    Bool,
    /// A `&T`
    SharedRef,
    /// A `&mut T`
    MutRef,
    /// A `*const T`
    ConstPtr,
    /// A `*mut T`
    MutPtr,
    /// An array.
    Array,
}

///////////////////////////

/// The typename and generics of the type this layout is associated to,
/// used for printing types (eg: `RVec<u8>` ).
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FmtFullType {
    pub(super) name: &'static str,
    pub(super) generics: GenericParams,
    pub(super) primitive: Option<TLPrimitive>,
    pub(super) utypeid: UTypeId,
}

impl FmtFullType {
    /// The name of a type.
    pub const fn name(&self) -> &'static str {
        self.name
    }
    /// The generic parmaters of a type.
    pub const fn generics(&self) -> GenericParams {
        self.generics
    }
}

////////////////////////////////////

/// Either a TLField or a TLFunction.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum TLFieldOrFunction {
    ///
    Field(TLField),
    ///
    Function(TLFunction),
}

impl From<TLField> for TLFieldOrFunction {
    fn from(x: TLField) -> Self {
        TLFieldOrFunction::Field(x)
    }
}

impl From<TLFunction> for TLFieldOrFunction {
    fn from(x: TLFunction) -> Self {
        TLFieldOrFunction::Function(x)
    }
}

impl Display for TLFieldOrFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TLFieldOrFunction::Field(x) => Display::fmt(x, f),
            TLFieldOrFunction::Function(x) => Display::fmt(x, f),
        }
    }
}

impl TLFieldOrFunction {
    /// Outputs this into a String with `Display` formatting.
    pub fn formatted_layout(&self) -> String {
        match self {
            TLFieldOrFunction::Field(x) => x.layout().to_string(),
            TLFieldOrFunction::Function(x) => x.to_string(),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
