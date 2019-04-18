/*!
Types for modeling the layout of a datatype
*/

use std::{
    cell::RefCell,
    collections::HashSet,
    fmt::{self, Debug, Display, Formatter},
    mem,
};


use crate::{
    utils::empty_slice, version::VersionStrings, 
    std_types::{RNone, ROption, RSome, RStr, StaticSlice,StaticStr,utypeid::UTypeId},
    ignored_wrapper::CmpIgnored,
    return_value_equality::ReturnValueEquality,
};

use super::{AbiInfo, GetAbiInfo};

/// The parameters for `TypeLayout::from_params`.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TypeLayoutParams {
    pub name: &'static str,
    pub package: &'static str,
    pub package_version: VersionStrings,
    pub file:&'static str,
    pub line:u32,
    pub data: TLData,
    pub generics: GenericParams,
    pub phantom_fields: &'static [TLField],
}


/// The layout of a type,
/// also includes metadata about where the type was defined.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TypeLayout {
    pub name: StaticStr,
    pub package: StaticStr,
    pub package_version: VersionStrings,
    pub file:CmpIgnored<StaticStr>, // This is for the Debug string
    pub line:CmpIgnored<u32>, // This is for the Debug string
    pub size: usize,
    pub alignment: usize,
    pub data: TLData,
    pub full_type: FullType,
    pub phantom_fields: StaticSlice<TLField>,
}


/// Which lifetime is being referenced by a field.
/// Allows lifetimes to be renamed,so long as the "same" lifetime is being referenced.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum LifetimeIndex {
    Static,
    Param(usize),
}

/// Represents all the generic parameters of a type.
/// 
/// This is different for every different generic parameter,
/// if any one of them changes it won't compare equal,
/// `<Vec<u32>>::ABI_INFO.get().layout.full_type.generics`
/// ẁon't compare equal to
/// `<Vec<()>>::ABI_INFO.get().layout.full_type.generics`
/// 
///
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct GenericParams {
    pub lifetime: StaticSlice<StaticStr>,
    pub type_: StaticSlice<&'static TypeLayout>,
    pub const_: StaticSlice<StaticStr>,
}

/// The typename and generics of the type this layout is associated to,
/// used for printing types.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct FullType {
    pub name: StaticStr,
    pub primitive: ROption<RustPrimitive>,
    pub generics: GenericParams,
}

/// What kind of type this is.struct/enum/etc.
///
/// Unions are currently treated as structs.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum TLData {
    /// All the bytes for the type are valid (not necessarily all bit patterns).
    ///
    /// If you use this variant,
    /// you must ensure the continuing validity of the same bit-patterns.
    Primitive,
    /// For structs and unions.
    Struct { fields: StaticSlice<TLField> },
    /// For enums.
    Enum {
        variants: StaticSlice<TLEnumVariant>,
    },
    /// vtables and modules that can be extended in minor versions.
    PrefixType{
        /// The first field in the suffix
        first_suffix_field:usize,
        fields: StaticSlice<TLField>,
    },
    /// For `#[repr(transparent)]` types.
    ReprTransparent(&'static AbiInfo),
}

/// A discriminant-only version of TLData.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum TLDataDiscriminant {
    Primitive,
    Struct,
    Enum,
    PrefixType,
    ReprTransparent,
}

/// The layout of an enum variant.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TLEnumVariant {
    pub name: StaticStr,
    pub fields: StaticSlice<TLField>,
}

/// The layout of a field.
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TLField {
    /// The field's name.
    pub name: StaticStr,
    /// Which lifetimes in the struct are referenced in the field type.
    pub lifetime_indices: StaticSlice<LifetimeIndex>,
    /// The layout of the field's type.
    ///
    /// This is a function pointer to avoid infinite recursion,
    /// if you have a `&'static AbiInfo`s with the same address as one of its parent type,
    /// you've encountered a cycle.
    pub abi_info: GetAbiInfo,
    /// Stores all extracted type parameters and return types of embedded function pointer types.
    pub subfields:StaticSlice<TLField>,
}

/// Used to print a field as its field and its type.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TLFieldAndType {
    inner: &'static TLField,
}

/// What primitive type this is.Used mostly for printing the type.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum RustPrimitive {
    Reference,
    MutReference,
    ConstPtr,
    MutPtr,
    Array,
}

///////////////////////////

impl TLField {
    pub const fn new(
        name: &'static str,
        lifetime_indices: &'static [LifetimeIndex],
        abi_info: GetAbiInfo,
    ) -> Self {
        Self {
            name: StaticStr::new(name),
            lifetime_indices: StaticSlice::new(lifetime_indices),
            abi_info,
            subfields:StaticSlice::new(&[]),
        }
    }

    pub const fn with_subfields(
        name: &'static str,
        lifetime_indices: &'static [LifetimeIndex],
        abi_info: GetAbiInfo,
        subfields:&'static [TLField],
    ) -> Self {
        Self {
            name: StaticStr::new(name),
            lifetime_indices: StaticSlice::new(lifetime_indices),
            abi_info,
            subfields: StaticSlice::new(subfields),
        }
    }

    /// Used for calling recursive methods,
    /// so as to avoid infinite recursion in types that reference themselves(even indirectly).
    fn recursive<F, U>(self, f: F) -> U
    where
        F: FnOnce(usize,TLFieldShallow) -> U,
    {
        let mut already_recursed = false;
        let mut recursion_depth=!0;
        let mut visited_nodes=!0;

        ALREADY_RECURSED.with(|state| {
            let mut state = state.borrow_mut();
            recursion_depth=state.recursion_depth;
            visited_nodes=state.visited_nodes;
            state.recursion_depth+=1;
            state.visited_nodes+=1;
            already_recursed = state.visited.replace(self.abi_info.get()).is_some();
        });

        let _guard=if visited_nodes==0 { Some(ResetRecursion) }else{ None };

        let field=TLFieldShallow::new(self, !already_recursed );
        let res = f( recursion_depth, field);

        ALREADY_RECURSED.with(|state| {
            let mut state = state.borrow_mut();
            state.recursion_depth-=1;
        });

        res
    }
}

impl PartialEq for TLField {
    fn eq(&self, other: &Self) -> bool {
        self.recursive(|_,this| {
            let r = TLFieldShallow::new(*other, this.abi_info.is_some());
            this == r
        })
    }
}

/// Need to avoid recursion somewhere,so I decided to stop at the field level.
impl Debug for TLField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.recursive(|recursion_depth,x|{
            if recursion_depth>=2 {
                writeln!(f,"<printing recursion limit>")
            }else{
                fmt::Debug::fmt(&x, f)
            }
        })
    }
}

///////////////////////////


struct ResetRecursion;

impl Drop for ResetRecursion{
    fn drop(&mut self){
        ALREADY_RECURSED.with(|state|{
            let mut state = state.borrow_mut();
            state.recursion_depth=0;
            state.visited_nodes=0;
            state.visited.clear();
        });
    }
}


struct RecursionState{
    recursion_depth:usize,
    visited_nodes:u64,
    visited:HashSet<*const AbiInfo>,
}


thread_local! {
    static ALREADY_RECURSED: RefCell<RecursionState> = RefCell::new(RecursionState{
        recursion_depth:0,
        visited_nodes:0,
        visited: HashSet::default(),
    });
}

///////////////////////////

impl TLFieldAndType {
    pub fn new(inner: &'static TLField) -> Self {
        Self { inner }
    }

    pub fn name(&self) -> RStr<'static> {
        self.inner.name.as_rstr()
    }

    pub fn full_type(&self) -> FullType {
        self.inner.abi_info.get().layout.full_type
    }
}

impl Debug for TLFieldAndType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TLFieldAndType")
            .field("field_name:", &self.inner.name)
            .field("type:", &self.inner.abi_info.get().layout.full_type())
            .finish()
    }
}

impl Display for TLFieldAndType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.inner.name,
            self.inner.abi_info.get().layout.full_type()
        )
    }
}

///////////////////////////

impl TypeLayout {
    pub(crate) const fn from_std_lib_primitive<T>(
        type_name: &'static str,
        prim: ROption<RustPrimitive>,
        data: TLData,
        generics: GenericParams,
    ) -> Self {
        Self::from_std_lib_phantom::<T>(type_name, prim, data, generics, empty_slice())
    }

    pub(crate) const fn from_std_lib<T>(
        type_name: &'static str,
        data: TLData,
        generics: GenericParams,
    ) -> Self {
        Self::from_std_lib_phantom::<T>(type_name, RNone, data, generics, empty_slice())
    }

    pub(crate) const fn from_std_lib_phantom<T>(
        type_name: &'static str,
        prim: ROption<RustPrimitive>,
        data: TLData,
        genparams: GenericParams,
        phantom: &'static [TLField],
    ) -> Self {
        Self {
            name: StaticStr::new(type_name),
            package: StaticStr::new("std"),
            package_version: VersionStrings {
                major: StaticStr::new("1"),
                minor: StaticStr::new("0"),
                patch: StaticStr::new("0"),
            },
            file:CmpIgnored::new(StaticStr::new("<standard_library>")),
            line:CmpIgnored::new(0),
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            data,
            full_type: FullType::new(type_name, prim, genparams),
            phantom_fields: StaticSlice::new(phantom),
        }
    }

    pub(crate) const fn full_type(&self) -> FullType {
        self.full_type
    }

    pub const fn from_params<T>(p: TypeLayoutParams) -> Self {
        let name = StaticStr::new(p.name);
        Self {
            name,
            package: StaticStr::new(p.package),
            package_version: p.package_version,
            file:CmpIgnored::new(StaticStr::new(p.file)),
            line:CmpIgnored::new(p.line),
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            data: p.data,
            full_type: FullType {
                name,
                primitive: RNone,
                generics: p.generics,
            },
            phantom_fields: StaticSlice::new(p.phantom_fields),
        }
    }
}

///////////////////////////

impl GenericParams {
    pub const fn new(
        lifetime: &'static [StaticStr],
        type_: &'static [&'static TypeLayout],
        const_: &'static [StaticStr],
    ) -> Self {
        Self {
            lifetime: StaticSlice::new(lifetime),
            type_: StaticSlice::new(type_),
            const_: StaticSlice::new(const_),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lifetime.is_empty() && self.type_.is_empty() && self.const_.is_empty()
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

        for (i, param) in self.lifetime.iter().cloned().enumerate() {
            fmt::Display::fmt(param.as_str(), &mut *f)?;
            post_iter(i, self.lifetime.len(), &mut *f)?;
        }
        for (i, param) in self.type_.iter().cloned().enumerate() {
            fmt::Debug::fmt(&param.full_type(), &mut *f)?;
            post_iter(i, self.type_.len(), &mut *f)?;
        }
        for (i, param) in self.const_.iter().cloned().enumerate() {
            fmt::Display::fmt(param.as_str(), &mut *f)?;
            post_iter(i, self.const_.len(), &mut *f)?;
        }
        fmt::Display::fmt(">", f)?;
        Ok(())
    }
}

///////////////////////////

impl TLData {
    pub const fn struct_(fields: &'static [TLField]) -> Self {
        TLData::Struct {
            fields: StaticSlice::new(fields),
        }
    }
    pub const fn enum_(variants: &'static [TLEnumVariant]) -> Self {
        TLData::Enum {
            variants: StaticSlice::new(variants),
        }
    }

    pub fn discriminant(&self) -> TLDataDiscriminant {
        match self {
            TLData::Primitive { .. } => TLDataDiscriminant::Primitive,
            TLData::Struct { .. } => TLDataDiscriminant::Struct,
            TLData::Enum { .. } => TLDataDiscriminant::Enum,
            TLData::PrefixType { .. } => TLDataDiscriminant::PrefixType,
            TLData::ReprTransparent { .. } => TLDataDiscriminant::ReprTransparent,
        }
    }
}

///////////////////////////

impl TLEnumVariant {
    pub const fn new(name: &'static str, fields: &'static [TLField]) -> Self {
        Self {
            name: StaticStr::new(name),
            fields: StaticSlice::new(fields),
        }
    }
}

///////////////////////////

impl FullType {
    pub const fn new(
        name: &'static str,
        primitive: ROption<RustPrimitive>,
        generics: GenericParams,
    ) -> Self {
        Self {
            name: StaticStr::new(name),
            primitive,
            generics,
        }
    }
}

impl Display for FullType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl Debug for FullType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (typename, start_gen, before_ty, ty_sep, end_gen) = match self.primitive {
            RSome(RustPrimitive::Reference) => ("&", "", " ", " ", " "),
            RSome(RustPrimitive::MutReference) => ("&", "", " mut ", " ", " "),
            RSome(RustPrimitive::ConstPtr) => ("*const", " ", "", " ", " "),
            RSome(RustPrimitive::MutPtr) => ("*mut", " ", "", " ", " "),
            RSome(RustPrimitive::Array) => ("", "[", "", ";", "]"),
            RNone => (self.name.as_str(), "<", "", ", ", ">"),
        };

        fmt::Display::fmt(typename, f)?;
        let mut is_before_ty = true;
        let generics = self.generics;
        if !generics.is_empty() {
            fmt::Display::fmt(start_gen, f)?;

            let post_iter = |i: usize, len: usize, f: &mut Formatter<'_>| -> fmt::Result {
                if i + 1 < len {
                    fmt::Display::fmt(ty_sep, f)?;
                }
                Ok(())
            };

            for (i, param) in generics.lifetime.iter().cloned().enumerate() {
                fmt::Display::fmt(param.as_str(), &mut *f)?;
                post_iter(i, generics.lifetime.len(), &mut *f)?;
            }
            for (i, param) in generics.type_.iter().cloned().enumerate() {
                if is_before_ty {
                    fmt::Display::fmt(before_ty, &mut *f)?;
                    is_before_ty = false;
                }
                fmt::Debug::fmt(&param.full_type(), &mut *f)?;
                post_iter(i, generics.type_.len(), &mut *f)?;
            }
            for (i, param) in generics.const_.iter().cloned().enumerate() {
                fmt::Display::fmt(param.as_str(), &mut *f)?;
                post_iter(i, generics.const_.len(), &mut *f)?;
            }
            fmt::Display::fmt(end_gen, f)?;
        }
        Ok(())
    }
}

////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq)]
struct TLFieldShallow {
    pub(crate) name: StaticStr,
    pub(crate) full_type: FullType,
    pub(crate) lifetime_indices: StaticSlice<LifetimeIndex>,
    /// This is None if it already printed that AbiInfo
    pub(crate) abi_info: Option<&'static AbiInfo>,
}

impl TLFieldShallow {
    fn new(field: TLField, include_abi_info: bool) -> Self {
        let abi_info = field.abi_info.get();
        TLFieldShallow {
            name: field.name,
            lifetime_indices: field.lifetime_indices,
            abi_info: if include_abi_info {
                Some(abi_info)
            } else {
                None
            },
            full_type: abi_info.layout.full_type,
        }
    }
}
