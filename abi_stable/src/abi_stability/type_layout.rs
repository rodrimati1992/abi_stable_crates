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
    utils::empty_slice, version::VersionStrings, RNone, ROption, RSome,
    RStr, StaticSlice, StaticStr,
};

use super::{AbiInfo, GetAbiInfo};

#[repr(C)]
#[derive(Debug, Copy, Clone,PartialEq)]
pub struct TypeLayoutParams {
    pub name: &'static str,
    pub package: &'static str,
    pub package_version: VersionStrings,
    pub data: TLData,
    pub generics: GenericParams,
    pub phantom_fields: &'static [TLField],
}

#[repr(C)]
#[derive(Debug, Copy, Clone,PartialEq)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TypeLayout {
    pub name: StaticStr,
    pub package: StaticStr,
    pub package_version: VersionStrings,
    pub size: usize,
    pub alignment: usize,
    pub data: TLData,
    pub full_type: TypePrinter,
    pub phantom_fields: StaticSlice<TLField>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum LifetimeIndex {
    Static,
    Param(usize),
}

#[repr(C)]
#[derive(Debug, Copy, Clone,PartialEq)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct GenericParams {
    pub lifetime: StaticSlice<StaticStr>,
    pub type_: StaticSlice<&'static TypeLayout>,
    pub const_: StaticSlice<StaticStr>,
}

#[repr(C)]
#[derive(Copy, Clone,PartialEq)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TypePrinter {
    pub name: StaticStr,
    pub primitive: ROption<RustPrimitive>,
    pub generics: GenericParams,
}

#[repr(C)]
#[derive(Debug, Copy, Clone,PartialEq)]
#[derive(StableAbi)]
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
    /// For `#[repr(transparent)]` types.
    ReprTransparent(&'static AbiInfo),
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum TLDataDiscriminant {
    Primitive,
    Struct,
    Enum,
    ReprTransparent,
}

#[repr(C)]
#[derive(Debug, Copy, Clone,PartialEq)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TLEnumVariant {
    pub name: StaticStr,
    pub fields: StaticSlice<TLField>,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TLField {
    pub name: StaticStr,
    /// Which lifetimes in the struct are referenced in the field type.
    pub lifetime_indices: StaticSlice<LifetimeIndex>,
    /// This is a function pointer to avoid infinite recursion,
    /// if you have a `&'static AbiInfo`s with the same address as one of its parent type,
    /// you've encountered a cycle.
    pub abi_info: GetAbiInfo,
}

#[repr(transparent)]
#[derive(Copy, Clone,PartialEq)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TLFieldAndType {
    inner: &'static TLField,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(StableAbi)]
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
        }
    }

    /// Used for calling recursive methods,
    /// so as to avoid infinite recursion in types that reference themselves(even indirectly). 
    fn recursive<F,U>(self,f:F)->U
    where 
        F:FnOnce(TLFieldShallow)->U
    {
        let mut set_was_empty = false;
        let mut already_recursed = false;

        ALREADY_RECURSED.with(|set| {
            let mut set = set.borrow_mut();
            set_was_empty = set.is_empty();
            already_recursed= !set.insert(self.abi_info.get());
        });

        let res=f(TLFieldShallow::new(self,already_recursed));

        if set_was_empty {
            ALREADY_RECURSED.with(|set| {
                set.borrow_mut().clear();
            });
        }

        res
    }
}


impl PartialEq for TLField{
    fn eq(&self,other:&Self)->bool{
        self.recursive(|this|{
            let r=TLFieldShallow::new(*other,this.abi_info.is_some());
            this==r
        })
    }
}


/// Need to avoid recursion somewhere,so I decided to stop at the field level.
impl Debug for TLField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.recursive(|x| fmt::Debug::fmt(&x,f) )
    }
}




thread_local! {
    static ALREADY_RECURSED: RefCell<HashSet<*const AbiInfo>> = RefCell::new(HashSet::default());
}

///////////////////////////

impl TLFieldAndType {
    pub fn new(inner: &'static TLField) -> Self {
        Self { inner }
    }

    pub fn name(&self) -> RStr<'static> {
        self.inner.name.as_rstr()
    }

    pub fn full_type(&self) -> TypePrinter {
        self.inner.abi_info.get().layout.full_type
    }
}

impl Debug for TLFieldAndType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TLFieldAndType")
            .field("field_name:", &self.inner.name)
            .field("type_name:", &self.inner.abi_info.get().layout.full_type())
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
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            data,
            full_type: TypePrinter::new(type_name, prim, genparams),
            phantom_fields: StaticSlice::new(phantom),
        }
    }

    pub(crate) const fn full_type(&self) -> TypePrinter {
        self.full_type
    }

    pub const fn from_params<T>(p: TypeLayoutParams) -> Self {
        let name = StaticStr::new(p.name);
        Self {
            name,
            package: StaticStr::new(p.package),
            package_version: p.package_version,
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            data: p.data,
            full_type: TypePrinter {
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

#[macro_export]
macro_rules! tl_genparams {
    ( $($lt:lifetime),* $(,)? ; $($ty:ty),* $(,)? ; $($const_p:expr),* $(,)? ) => ({
        #[allow(unused_imports)]
        use $crate::{
            abi_stability::{SharedStableAbi,type_layout::GenericParams},
            StaticStr,
            utils::as_slice,
        };

        GenericParams::new(
            &[$( StaticStr::new( stringify!($lt) ) ,)*],
            &[$( <$ty as SharedStableAbi>::LAYOUT ,)*],
            &[$( StaticStr::new( stringify!($const_p) ) ,)*],
        )
    })
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

impl TypePrinter {
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

impl Display for TypePrinter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl Debug for TypePrinter {
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

#[repr(C)]
#[derive(Debug, Copy, Clone,PartialEq)]
struct TLFieldShallow {
    pub name: StaticStr,
    pub full_type: TypePrinter,
    pub lifetime_indices: StaticSlice<LifetimeIndex>,
    /// This is None if it already printed that AbiInfo
    pub abi_info: Option<&'static AbiInfo>,
}


impl TLFieldShallow{
    fn new(field:TLField,include_abi_info:bool)->Self{
        let abi_info=field.abi_info.get();
        TLFieldShallow {
            name: field.name,
            lifetime_indices: field.lifetime_indices,
            abi_info: if include_abi_info { Some(abi_info) }else{ None },
            full_type: abi_info.layout.full_type,
        }
    }
}
