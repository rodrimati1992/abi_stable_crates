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
    abi_stability::stable_abi_trait::{AbiInfo,GetAbiInfo},
    const_utils::empty_slice, sabi_types::VersionStrings, 
    sabi_types::CmpIgnored,
    std_types::{RNone, ROption, RSome, RStr, StaticSlice,StaticStr,RSlice},
    prefix_type::{FieldAccessibility,IsConditional},
    reflection::ModReflMode,
};


mod construction;
mod tl_enums;
mod tl_field;
mod tl_fields;
mod tl_functions;
mod tl_other;
pub mod tagging;

pub use self::{
    construction::{
        TypeLayoutParams,
        _private_TypeLayoutDerive,
        ItemInfo,
    },
    tl_enums::{
        TLEnum,
        TLDiscriminant,
        TLDiscriminants,
        DiscriminantRepr,
        GetVariantNames,
        IsExhaustive,
        TLNonExhaustive,
        IncompatibleWithNonExhaustive,
    },
    tl_field::{
        FieldAccessor,
        TLField,
        TLFieldAndType,
    },
    tl_fields::{
        TLFields,
        TLFieldsOrSlice,
        FieldIndex,
        Field1to1,
        WithFieldIndex,
        TLFieldsIterator,
        SliceAndFieldIndices,
    },
    tl_functions::{
        TLFunctions,
        CompTLFunction,
        StartLen,
        TLFunctionRange,
    },
    tl_other::{
        FullType,
        GenericParams,
        LifetimeIndex,
        ModPath,
        ReprAttr,
        TLData,
        TLDataDiscriminant,
        TLPrimitive,
        TLFunction,
        TLPrefixType,
    },
    tagging::Tag,
};


////////////////////////////////////////////////////////////////////////////////


/// The layout of a type,
/// also includes metadata about where the type was defined.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
// #[sabi(debug_print)]
pub struct TypeLayout {
    /// The name of this type,the `Option` in `Òption<T>`.
    pub name: StaticStr,
    /// Contains information about where the type was defined.
    ///
    /// This is (mostly) for the Debug string
    pub item_info:CmpIgnored<ItemInfo>,
    /// The size of the type
    pub size: usize,
    /// The alignment of the type.
    pub alignment: usize,
    /// What kind of type this is,Primitive/Struct/Enum/PrefixType.
    pub data: TLData,
    /// Used for printing the type at runtime,
    pub full_type: FullType,
    /// Phantom fields,which don't have a runtime component(they aren't stored anywhere),
    /// and are checked in layout checking.
    pub phantom_fields: StaticSlice<TLField>,
    /// Extra data stored for reflection,
    /// so as to not break the abi every time that more stuff is added for reflection.
    pub reflection_tag:Tag,
    #[doc(hidden)]
    /// Extra data stored for reflection,
    /// so as to not break the abi every time that more stuff is added.
    pub private_tag:Tag,
    /// A json-like data structure used to add extra checks.
    pub tag:Tag,
    /// The representation attribute(s) of this type.
    pub repr_attr:ReprAttr,
    /// How this type is treated when interpreted as a module.
    pub mod_refl_mode:ModReflMode,
}



///////////////////////////

impl TypeLayout {

    pub(crate) const fn full_type(&self) -> FullType {
        self.full_type
    }

    /// Gets the package of the type.
    pub fn package_and_version(&self)->(StaticStr,VersionStrings){
        let (package,version)=self.item_info.package_and_version();

        (
            StaticStr::new(package),
            VersionStrings::new(version)
        )
    }


    /// Gets the package of the type.
    pub fn package(&self)->StaticStr{
        let (package,_)=self.item_info.package_and_version();
        StaticStr::new(package)
    }

    /// Gets the package version for the package of type.
    pub fn package_version(&self)->VersionStrings{
        let (_,version)=self.item_info.package_and_version();
        VersionStrings::new(version)
    }


    /// Gets in which line the type was defined.
    pub fn line(&self)->u32{
        self.item_info.line
    }

    /// Gets the full path to the module where the type was defined.
    pub fn mod_path(&self)->&ModPath{
        &self.item_info.mod_path
    }
}


impl TypeLayout {
    pub(crate) const fn from_std<T>(
        type_name: &'static str,
        data: TLData,
        repr:ReprAttr,
        item_info:ItemInfo,
        generics: GenericParams,
    ) -> Self {
        Self::from_std_full::<T>(
            type_name, 
            RNone, 
            item_info,
            data, 
            repr,
            generics, 
            empty_slice()
        )
    }

    pub(crate) const fn from_std_full<T>(
        type_name: &'static str,
        prim: ROption<TLPrimitive>,
        item_info:ItemInfo,
        data: TLData,
        repr:ReprAttr,
        genparams: GenericParams,
        phantom: &'static [TLField],
    ) -> Self {
        Self {
            name: StaticStr::new(type_name),
            item_info:CmpIgnored::new(item_info),
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            data,
            full_type: FullType::new(type_name, prim, genparams),
            phantom_fields: StaticSlice::new(phantom),
            reflection_tag:Tag::null(),
            private_tag:Tag::null(),
            tag:Tag::null(),
            mod_refl_mode:ModReflMode::Module,
            repr_attr:repr,
        }
    }

    /// Constructs a TypeLayout from a parameter struct.
    pub const fn from_params<T>(p: TypeLayoutParams) -> Self {
        let name = StaticStr::new(p.name);
        Self {
            name,
            item_info:CmpIgnored::new(p.item_info),
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            data: p.data,
            full_type: FullType {
                name,
                primitive: RNone,
                generics: p.generics,
            },
            phantom_fields: StaticSlice::new(empty_slice()),
            reflection_tag:Tag::null(),
            private_tag:Tag::null(),
            tag:Tag::null(),
            mod_refl_mode:ModReflMode::Module,
            repr_attr:ReprAttr::C(RNone),
        }
    }


    #[doc(hidden)]
    pub const fn from_derive<T>(p: _private_TypeLayoutDerive) -> Self {
        let name = StaticStr::new(p.name);
        Self {
            name,
            item_info:CmpIgnored::new(p.item_info),
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            data: p.data,
            full_type: FullType {
                name,
                primitive: RNone,
                generics: p.generics,
            },
            phantom_fields: StaticSlice::new(p.phantom_fields),
            reflection_tag:Tag::null(),
            private_tag:Tag::null(),
            tag:p.tag,
            mod_refl_mode:p.mod_refl_mode,
            repr_attr:p.repr_attr,
        }
    }

    /// Sets the phantom fields,
    /// fields which don't have a runtime representation
    /// and are checked in the layout checker.
    pub const fn set_phantom_fields(mut self,phantom_fields: &'static [TLField])->Self{
        self.phantom_fields=StaticSlice::new(phantom_fields);
        self
    }

    /// Sets the Tag of the type,checked for compatibility in layotu checking..
    pub const fn set_tag(mut self,tag:Tag)->Self{
        self.tag=tag;
        self
    }

    /// Sets the Tag of the type used for reflection,this is not checked in layout checking.
    pub const fn set_reflection_tag(mut self,reflection_tag:Tag)->Self{
        self.reflection_tag=reflection_tag;
        self
    }

    #[doc(hidden)]
    pub const fn _private_method_set_private_tag(mut self,private_tag:Tag)->Self{
        self.private_tag=private_tag;
        self
    }

    /// Sets the module reflection mode of the type,
    /// determining how this type is accessed when interpreted as a module.
    pub const fn set_mod_refl_mode(mut self,mod_refl_mode:ModReflMode)->Self{
        self.mod_refl_mode=mod_refl_mode;
        self
    }

    /// Sets the `#[repr(_)]` attribute of the type.
    pub const fn set_repr_attr(mut self,repr_attr:ReprAttr)->Self{
        self.repr_attr=repr_attr;
        self
    }
}


