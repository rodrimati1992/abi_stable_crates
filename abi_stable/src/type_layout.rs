//! Types for modeling the layout of a datatype

use std::{
    cell::RefCell,
    cmp::{Eq, PartialEq},
    collections::HashSet,
    fmt::{self, Debug, Display, Formatter},
    mem::{self, ManuallyDrop},
};

use core_extensions::{matches, StringExt};

use crate::{
    abi_stability::{
        extra_checks::{ExtraChecksStaticRef, StoredExtraChecks},
        stable_abi_trait::AbiConsts,
    },
    const_utils::log2_usize,
    prefix_type::{FieldAccessibility, FieldConditionality},
    reflection::ModReflMode,
    sabi_types::{CmpIgnored, NulStr, VersionStrings},
    std_types::{RSlice, RStr, UTypeId},
};

mod construction;
pub mod data_structures;
mod iterators;
mod printing;
mod shared_vars;
mod small_types;
pub mod tagging;
mod tl_data;
mod tl_enums;
mod tl_field;
mod tl_fields;
mod tl_functions;
mod tl_lifetimes;
mod tl_multi_tl;
mod tl_other;
mod tl_prefix;
mod tl_reflection;

pub(crate) use self::iterators::ChainOnce;

pub use self::{
    construction::{ItemInfo, _private_MonoTypeLayoutDerive, _private_TypeLayoutDerive},
    shared_vars::{MonoSharedVars, SharedVars},
    small_types::{OptionU16, OptionU8, StartLen, StartLenConverter, StartLenRepr},
    tagging::Tag,
    tl_data::{GenericTLData, MismatchedTLDataVariant, MonoTLData, TLData, TLDataDiscriminant},
    tl_enums::{
        DiscriminantRepr, GenericTLEnum, IncompatibleWithNonExhaustive, IsExhaustive,
        MakeTLNonExhaustive, MonoTLEnum, TLDiscriminant, TLDiscriminants, TLEnum, TLNonExhaustive,
    },
    tl_field::{CompTLField, CompTLFieldRepr, TLField},
    tl_fields::{CompTLFields, TLFields, TLFieldsIterator},
    tl_functions::{
        CompTLFunction, TLFunction, TLFunctionIter, TLFunctionQualifiers, TLFunctionSlice,
        TLFunctions,
    },
    tl_lifetimes::{
        LifetimeArrayOrSlice, LifetimeIndex, LifetimeIndexArray, LifetimeIndexPair,
        LifetimeIndexPairRepr, LifetimeRange,
    },
    tl_multi_tl::{MTLIterator, MultipleTypeLayouts, TypeLayoutIndex, TypeLayoutRange},
    tl_other::{
        CompGenericParams, FmtFullType, GenericParams, ModPath, ReprAttr, TLFieldOrFunction,
        TLPrimitive,
    },
    tl_prefix::{GenericTLPrefixType, MonoTLPrefixType, TLPrefixType},
    tl_reflection::{CompFieldAccessor, FieldAccessor},
};

////////////////////////////////////////////////////////////////////////////////

/// The layout of a type,
/// also includes metadata about where the type was defined.
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
// I am specifically applying this attribute to TypeLayout to make
// ExtraChecks take less time checking its own layout.
//
// Also because checking the layout of TypeLayout is redundant,
// since I have to trust that it's correct to be able to use it
// to check the layout of anything(including itself).
#[sabi(unsafe_sabi_opaque_fields)]
pub struct TypeLayout {
    shared_vars: &'static SharedVars,

    /// The parts of the type layout that never change based on generic parameters.
    mono: &'static MonoTypeLayout,

    /// Whether the type uses non-zero value optimization,
    /// if true then an `Option<Self>` implements StableAbi.
    is_nonzero: bool,

    /// The alignment of the type represented as (1 << self.alignment_power_of_two).
    alignment_power_of_two: u8,

    /// The size of the type
    size: usize,

    tag: Option<&'static Tag>,

    data: GenericTLData,

    /// A json-like data structure used to add extra checks.
    extra_checks: CmpIgnored<Option<&'static ManuallyDrop<StoredExtraChecks>>>,

    /// A function to get the unique identifier for some type
    type_id: extern "C" fn() -> UTypeId,
}

unsafe impl Send for TypeLayout {}
unsafe impl Sync for TypeLayout {}

unsafe impl Send for MonoTypeLayout {}
unsafe impl Sync for MonoTypeLayout {}

///////////////////////////

impl TypeLayout {
    pub(crate) const fn from_std<T>(
        shared_vars: &'static SharedVars,
        mono: &'static MonoTypeLayout,
        abi_consts: AbiConsts,
        data: GenericTLData,
    ) -> Self {
        Self {
            shared_vars,
            mono,
            is_nonzero: abi_consts.is_nonzero,
            type_id: abi_consts.type_id.0,
            alignment_power_of_two: log2_usize(mem::align_of::<T>()),
            size: mem::size_of::<T>(),
            data,
            extra_checks: CmpIgnored::new(None),
            tag: None,
        }
    }

    #[doc(hidden)]
    pub const fn from_derive<T>(p: _private_TypeLayoutDerive) -> Self {
        Self {
            shared_vars: p.shared_vars,
            mono: p.mono,
            is_nonzero: p.abi_consts.is_nonzero,
            type_id: p.abi_consts.type_id.0,
            alignment_power_of_two: log2_usize(mem::align_of::<T>()),
            size: mem::size_of::<T>(),
            data: p.data,
            extra_checks: CmpIgnored::new(p.extra_checks),
            tag: p.tag,
        }
    }

    /// Gets the SharedVars of the type,
    /// containing the slices that many types inside TypeLayout contain ranges into.
    pub const fn shared_vars(&self) -> &'static SharedVars {
        self.shared_vars
    }

    /// Gets a type used to print the type(ie:`Foo<'a,'b,u32,RString,1,2>`)
    #[doc(hidden)]
    pub fn full_type(&self) -> FmtFullType {
        FmtFullType {
            name: self.mono.name(),
            generics: self.generics(),
            primitive: self.mono.data.to_primitive(),
            utypeid: self.get_utypeid(),
        }
    }

    /// Gets the package and package version where the type was declared.
    pub fn package_and_version(&self) -> (RStr<'static>, VersionStrings) {
        let (package, version) = self.item_info().package_and_version();

        (RStr::from_str(package), VersionStrings::new(version))
    }

    /// Gets the package where the type was declared.
    pub fn package(&self) -> RStr<'static> {
        let (package, _) = self.item_info().package_and_version();
        RStr::from_str(package)
    }

    /// Gets the package version for the package where the type was declared.
    pub fn package_version(&self) -> VersionStrings {
        let (_, version) = self.item_info().package_and_version();
        VersionStrings::new(version)
    }

    /// Gets which line the type was defined in.
    pub const fn line(&self) -> u32 {
        self.item_info().line
    }

    /// Gets the full path to the module where the type was defined.
    pub const fn mod_path(&self) -> ModPath {
        self.item_info().mod_path
    }

    /// Gets a trait object used to check extra properties about the type.
    #[inline]
    pub fn extra_checks(&self) -> Option<ExtraChecksStaticRef> {
        self.extra_checks.value.map(|x| x.sabi_reborrow())
    }

    /// Gets the fields of the type.
    ///
    /// # Return value
    ///
    /// If this a:
    ///
    /// - primitive or opaque type:
    ///     It returns `None`.
    ///
    /// - enum:
    ///     It returns `Some()` with all the fields in the order that they were declared,
    ///     ignoring variants.
    ///
    /// - structs/unions/prefix types:
    ///     It returns `Some()` with all the fields in the order that they were declared.
    ///
    pub const fn get_fields(&self) -> Option<TLFields> {
        match self.mono.get_fields() {
            Some(fields) => Some(fields.expand(self.shared_vars)),
            None => None,
        }
    }

    /// Whether this is a prefix-type(module or vtable).
    pub const fn is_prefix_kind(&self) -> bool {
        matches!(self.data, GenericTLData::PrefixType { .. })
    }

    /// Gets the name of the type.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.mono.name()
    }

    /// Gets whether the type is a NonZero type,
    /// which can be put in an `Option` while being ffi-safe.
    #[inline]
    pub const fn is_nonzero(&self) -> bool {
        self.is_nonzero
    }

    #[doc(hidden)]
    #[cfg(feature = "testing")]
    pub const fn _set_is_nonzero(mut self, is_nonzero: bool) -> Self {
        self.is_nonzero = is_nonzero;
        self
    }

    #[doc(hidden)]
    #[cfg(feature = "testing")]
    pub const fn _set_extra_checks(
        mut self,
        extra_checks: CmpIgnored<Option<&'static ManuallyDrop<StoredExtraChecks>>>,
    ) -> Self {
        self.extra_checks = extra_checks;
        self
    }

    #[doc(hidden)]
    #[cfg(feature = "testing")]
    pub const fn _set_type_id(mut self, type_id: extern "C" fn() -> UTypeId) -> Self {
        self.type_id = type_id;
        self
    }

    /// Gets the `UTypeId` for the type,
    /// which is an ffi safe equivalent of `TypeId`.
    #[inline]
    pub fn get_utypeid(&self) -> UTypeId {
        (self.type_id)()
    }

    /// Gets information about where a type was declared.
    #[inline]
    pub const fn item_info(&self) -> &ItemInfo {
        self.mono.item_info()
    }

    /// Gets the alignment of the type.
    #[inline]
    pub const fn alignment(&self) -> usize {
        1_usize << (self.alignment_power_of_two as u32)
    }

    /// Gets the size of the type.
    #[inline]
    pub const fn size(&self) -> usize {
        self.size
    }

    /// Gets the `Tag` associated with a type,
    /// a JSON-like datastructure which is another way to
    /// check extra properties about a type.
    pub const fn tag(&self) -> &'static Tag {
        match self.tag {
            Some(x) => x,
            None => Tag::NULL,
        }
    }

    /// Gets the representation attribute of the type.
    pub const fn repr_attr(&self) -> ReprAttr {
        self.mono.repr_attr()
    }

    /// Gets the `ModReflMode` for the type,
    /// whether this is a module whose definition can be reflected on at runtime.
    pub const fn mod_refl_mode(&self) -> ModReflMode {
        self.mono.mod_refl_mode()
    }

    /// The interior of the type definition,
    /// describing whether the type is a primitive/enum/struct/union and its contents.
    pub fn data(&self) -> TLData {
        self.mono
            .data
            .expand(self.data, self.shared_vars)
            .unwrap_or_else(|e| {
                panic!("\nError inside of '{}' type \n{}", self.full_type(), e);
            })
    }

    /// Describes whether the type is a primitive/enum/struct/union,
    /// every variant corresponds to a `TLData` variant of the same name.
    pub const fn data_discriminant(&self) -> TLDataDiscriminant {
        self.mono.data.as_discriminant()
    }

    /// Gets the virtual fields that aren't part of th type definition,
    /// but are checked as part of the type
    #[inline]
    pub fn phantom_fields(&self) -> TLFields {
        unsafe {
            let slice = std::slice::from_raw_parts(
                self.mono.phantom_fields,
                self.mono.phantom_fields_len as usize,
            );
            TLFields::from_fields(slice, self.shared_vars)
        }
    }

    /// Gets the generic parameters of the type.
    pub fn generics(&self) -> GenericParams {
        self.mono.generics.expand(self.shared_vars)
    }

    /// Gets the parts of the type layout that don't change with generic parameters.
    pub const fn mono_type_layout(&self) -> &MonoTypeLayout {
        self.mono
    }
}

impl PartialEq for TypeLayout {
    fn eq(&self, other: &TypeLayout) -> bool {
        self.get_utypeid() == other.get_utypeid()
    }
}

impl Eq for TypeLayout {}

////////////////////////////////////////////////////////////////////////////////

/// The data in the type layout that does not depend on generic parameters.
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct MonoTypeLayout {
    shared_vars: MonoSharedVars,

    /// The name of the type.
    name: *const u8,

    /// Contains information about where the type was defined.
    ///
    item_info: CmpIgnored<ItemInfo>,

    /// What kind of type this is,Primitive/Struct/Enum/PrefixType.
    data: MonoTLData,
    /// The generic parameters of the type
    generics: CompGenericParams,

    /// Phantom fields,which don't have a runtime component(they aren't stored anywhere),
    /// and are checked in layout checking.
    phantom_fields: *const CompTLField,
    phantom_fields_len: u8,

    /// The representation attribute(s) of the type.
    repr_attr: ReprAttr,

    /// How the type is treated when interpreted as a module.
    mod_refl_mode: ModReflMode,

    name_len: u16,
}

#[allow(clippy::too_many_arguments)]
impl MonoTypeLayout {
    pub(crate) const fn new(
        shared_vars: MonoSharedVars,
        name: RStr<'static>,
        item_info: ItemInfo,
        data: MonoTLData,
        generics: CompGenericParams,
        repr_attr: ReprAttr,
        mod_refl_mode: ModReflMode,
        phantom_fields: RSlice<'static, CompTLField>,
    ) -> Self {
        Self {
            shared_vars,
            name: name.as_ptr(),
            name_len: name.len() as u16,
            item_info: CmpIgnored::new(item_info),
            data,
            generics,
            repr_attr,
            mod_refl_mode,
            phantom_fields: phantom_fields.as_ptr(),
            phantom_fields_len: phantom_fields.len() as u8,
        }
    }

    #[doc(hidden)]
    pub const fn from_derive(p: _private_MonoTypeLayoutDerive) -> Self {
        Self {
            name: p.name.as_ptr(),
            name_len: p.name.len() as u16,
            phantom_fields: p.phantom_fields.as_ptr() as *const CompTLFieldRepr
                as *const CompTLField,
            phantom_fields_len: p.phantom_fields.len() as u8,
            item_info: CmpIgnored::new(p.item_info),
            data: p.data,
            generics: p.generics,
            repr_attr: p.repr_attr,
            mod_refl_mode: p.mod_refl_mode,
            shared_vars: p.shared_vars,
        }
    }

    /// Gets the name of the type.
    pub fn name(&self) -> &'static str {
        unsafe {
            let slic = std::slice::from_raw_parts(self.name, self.name_len as usize);
            std::str::from_utf8_unchecked(slic)
        }
    }

    /// Gets the representation attribute of the type.
    pub const fn repr_attr(&self) -> ReprAttr {
        self.repr_attr
    }

    /// Gets the `ModReflMode` for the type,
    /// whether this is a module whose definition can be reflected on at runtime.
    pub const fn mod_refl_mode(&self) -> ModReflMode {
        self.mod_refl_mode
    }

    /// Gets information about where a type was declared.
    pub const fn item_info(&self) -> &ItemInfo {
        &self.item_info.value
    }

    /// Gets the SharedVars of the type,
    /// containing the slices that many types inside TypeLayout contain ranges into.
    pub const fn shared_vars(&self) -> &MonoSharedVars {
        &self.shared_vars
    }

    /// Gets the SharedVars of the type,
    /// containing the slices that many types inside TypeLayout contain ranges into.
    ///
    /// This was defined as a workaround for an internal compiler error in nightly.
    pub const fn shared_vars_static(&'static self) -> &'static MonoSharedVars {
        &self.shared_vars
    }

    /// Gets the compressed versions of the fields of the type.
    ///
    /// # Return value
    ///
    /// If this a:
    ///
    /// - primitive or opaque type:
    ///     It returns `None`.
    ///
    /// - enum:
    ///     It returns `Some()` with all the fields in the order that they were declared,
    ///     ignoring variants.
    ///
    /// - structs/unions/prefix types:
    ///     It returns `Some()` with all the fields in the order that they were declared.
    ///
    pub const fn get_fields(&self) -> Option<CompTLFields> {
        match self.data {
            MonoTLData::Primitive { .. } => None,
            MonoTLData::Opaque => None,
            MonoTLData::Struct { fields } => Some(fields),
            MonoTLData::Union { fields } => Some(fields),
            MonoTLData::Enum(tlenum) => Some(tlenum.fields),
            MonoTLData::PrefixType(prefix) => Some(prefix.fields),
        }
    }

    /// Gets an iterator over all the names of the fields in the type.
    pub fn field_names(&self) -> impl ExactSizeIterator<Item = &'static str> + Clone + 'static {
        self.get_fields()
            .unwrap_or(CompTLFields::EMPTY)
            .field_names(&self.shared_vars)
    }

    /// Gets the name of the `nth` field in the type.
    /// Returns `None` if there is no `nth` field.
    pub fn get_field_name(&self, nth: usize) -> Option<&'static str> {
        self.get_fields()
            .unwrap_or(CompTLFields::EMPTY)
            .get_field_name(nth, &self.shared_vars)
    }
}

impl Debug for MonoTypeLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MonoTypeLayout")
            .field("name", &self.name())
            .field("item_info", self.item_info())
            .field("repr_attr", &self.repr_attr())
            .field("mod_refl_mode", &self.mod_refl_mode())
            .finish()
    }
}
