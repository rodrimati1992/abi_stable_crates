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
    const_utils::empty_slice, version::VersionStrings, 
    std_types::{RNone, ROption, RSome, RStr, StaticSlice,StaticStr},
    ignored_wrapper::CmpIgnored,
    prefix_type::{FieldAccessibility,IsConditional},
    reflection::ModReflMode,
};

use super::{
    AbiInfo, 
    GetAbiInfo,
    tagging::Tag,
};


mod construction;
mod tl_field;
mod tl_other;

pub use self::{
    construction::{
        TypeLayoutParams,
        _private_TypeLayoutDerive,
        ItemInfo,
    },
    tl_field::{
        FieldAccessor,
        TLField,
        TLFieldAndType,
    },
    tl_other::{
        DiscriminantRepr,
        FullType,
        GenericParams,
        LifetimeIndex,
        ModPath,
        ReprAttr,
        RustPrimitive,
        TLData,
        TLDataDiscriminant,
        TLDiscriminant,
        TLEnumVariant,
        TLFunction,
        TLPrefixType,
    },
};


////////////////////////////////////////////////////////////////////////////////


/// The layout of a type,
/// also includes metadata about where the type was defined.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
// #[sabi(debug_print)]
pub struct TypeLayout {
    pub name: StaticStr,
    // This is (mostly) for the Debug string
    pub item_info:CmpIgnored<ItemInfo>,
    pub size: usize,
    pub alignment: usize,
    pub data: TLData,
    pub full_type: FullType,
    pub phantom_fields: StaticSlice<TLField>,
    /// Extra data stored for reflection,
    /// so as to not break the abi every time that more stuff is added for reflection.
    pub reflection_tag:Tag,
    /// Extra data stored for reflection,
    /// so as to not break the abi every time that more stuff is added for reflection.
    pub private_tag:Tag,
    pub tag:Tag,
    pub mod_refl_mode:ModReflMode,
    pub repr_attr:ReprAttr,
}



///////////////////////////

impl TypeLayout {

    pub(crate) const fn full_type(&self) -> FullType {
        self.full_type
    }

    pub fn package(&self)->StaticStr{
        self.item_info.package
    }
    pub fn package_version(&self)->&VersionStrings{
        &self.item_info.package_version
    }
    pub fn file(&self)->StaticStr{
        self.item_info.file
    }
    pub fn line(&self)->u32{
        self.item_info.line
    }
    pub fn mod_path(&self)->&ModPath{
        &self.item_info.mod_path
    }
}


impl TypeLayout {
    pub(crate) const fn from_std<T>(
        type_name: &'static str,
        data: TLData,
        item_info:ItemInfo,
        generics: GenericParams,
    ) -> Self {
        Self::from_std_full::<T>(
            type_name, 
            RNone, 
            item_info,
            data, 
            generics, 
            empty_slice()
        )
    }

    pub(crate) const fn from_std_full<T>(
        type_name: &'static str,
        prim: ROption<RustPrimitive>,
        item_info:ItemInfo,
        data: TLData,
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
            repr_attr:ReprAttr::C(RNone),
        }
    }

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

    pub const fn set_phantom_fields(mut self,phantom_fields: &'static [TLField])->Self{
        self.phantom_fields=StaticSlice::new(phantom_fields);
        self
    }

    pub const fn set_tag(mut self,tag:Tag)->Self{
        self.tag=tag;
        self
    }

    pub const fn set_reflection_tag(mut self,reflection_tag:Tag)->Self{
        self.reflection_tag=reflection_tag;
        self
    }

    #[doc(hidden)]
    pub const fn _private_method_set_private_tag(mut self,private_tag:Tag)->Self{
        self.private_tag=private_tag;
        self
    }

    pub const fn set_mod_refl_mode(mut self,mod_refl_mode:ModReflMode)->Self{
        self.mod_refl_mode=mod_refl_mode;
        self
    }

    pub const fn set_repr_attr(mut self,repr_attr:ReprAttr)->Self{
        self.repr_attr=repr_attr;
        self
    }
}


