/*!
Types for modeling the layout of a datatype
*/

use std::{
    cell::RefCell,
    collections::HashSet,
    fmt::{self, Debug, Display, Formatter},
    mem,
};

use core_extensions::{matches,StringExt};

use crate::{
    abi_stability::{
        stable_abi_trait::{GetTypeLayout,AbiConsts,TypeKind},
        ExtraChecksStaticRef,
    },
    const_utils::empty_slice, sabi_types::VersionStrings, 
    sabi_types::{CmpIgnored,Constructor},
    std_types::{RNone, ROption, RSome, RStr, StaticSlice,StaticStr,RSlice,UTypeId},
    prefix_type::{FieldAccessibility,IsConditional},
    reflection::ModReflMode,
};


mod construction;
mod tl_enums;
mod tl_field;
mod tl_fields;
mod tl_functions;
mod tl_other;
mod printing;
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
    },
    tl_fields::{
        TLFields,
        TLFOSIter,
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
        TLFieldOrFunction,
    },
    tagging::Tag,
};


////////////////////////////////////////////////////////////////////////////////


/// The layout of a type,
/// also includes metadata about where the type was defined.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq,StableAbi)]
// I am specifically applying this attribute to TypeLayout to make
// ExtraChecks take less time checking its own layout.
#[sabi(unsafe_sabi_opaque_fields)]
pub struct TypeLayout {
    /// Used for printing the type at runtime,
    pub full_type: FullType,
    /// The name of this type,the `Option` in `Òption<T>`.
    pub name: StaticStr,
    /// Contains constants equivalent to the associated types in 
    /// the SharedStableAbi impl of the type this is the layout for.
    pub abi_consts:AbiConsts,
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
    /// Phantom fields,which don't have a runtime component(they aren't stored anywhere),
    /// and are checked in layout checking.
    pub phantom_fields: StaticSlice<TLField>,
    /// Extra data stored for reflection,
    /// so as to not break the abi every time that more stuff is added for reflection.
    pub reflection_tag:&'static Tag,
    /// A json-like data structure used to add extra checks.
    pub tag:&'static Tag,
    /// A json-like data structure used to add extra checks.
    pub extra_checks:CmpIgnored<Option<Constructor<ExtraChecksStaticRef>>>,
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

    #[inline]
    pub fn extra_checks(&self)->Option<ExtraChecksStaticRef>{
        self.extra_checks.value.map(Constructor::get)
    }

/**
Gets the fields of this type.

# Return value

If this a:

- primitive or opaque type:
    It returns `None`.

- enum:
    It returns `Some()` with all the fields in the order that they were declared,
    ignoring variants.

- structs/unions/prefix types:
    It returns `Some()` with all the fields in the order that the were declared.

*/
    pub fn get_fields(&self)->Option<TLFieldsOrSlice>{
        match &self.data {
            TLData::Primitive{..}|TLData::Opaque=>
                None,
            TLData::Struct{fields}=>Some(*fields),
            TLData::Union{fields}=>Some(*fields),
            TLData::Enum (tlenum)=>Some(tlenum.fields),
            TLData::PrefixType(prefix)=>Some(prefix.fields),
        }
    }

}


impl TypeLayout {
    pub(crate) const fn from_std<T>(
        abi_consts:AbiConsts,
        type_name: &'static str,
        data: TLData,
        repr:ReprAttr,
        item_info:ItemInfo,
        generics: GenericParams,
    ) -> Self {
        Self::from_std_full::<T>(
            abi_consts,
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
        abi_consts:AbiConsts,
        type_name: &'static str,
        prim: ROption<TLPrimitive>,
        item_info:ItemInfo,
        data: TLData,
        repr:ReprAttr,
        genparams: GenericParams,
        phantom: &'static [TLField],
    ) -> Self {
        Self {
            abi_consts,
            name: StaticStr::new(type_name),
            item_info:CmpIgnored::new(item_info),
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            data,
            full_type: FullType::new(type_name, prim, genparams),
            phantom_fields: StaticSlice::new(phantom),
            reflection_tag:Tag::NULL,
            tag:Tag::NULL,
            extra_checks:CmpIgnored::new(None),
            mod_refl_mode:ModReflMode::Module,
            repr_attr:repr,
        }
    }

    /// Constructs a TypeLayout from a parameter struct.
    pub const fn from_params<T>(p: TypeLayoutParams) -> Self {
        let name = StaticStr::new(p.name);
        Self {
            abi_consts:p.abi_consts,
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
            reflection_tag:Tag::NULL,
            tag:Tag::NULL,
            extra_checks:CmpIgnored::new(None),
            mod_refl_mode:ModReflMode::Module,
            repr_attr:ReprAttr::C(RNone),
        }
    }


    #[doc(hidden)]
    pub const fn from_derive<T>(p: _private_TypeLayoutDerive) -> Self {
        let name = StaticStr::new(p.name);
        Self {
            abi_consts:p.abi_consts,
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
            reflection_tag:Tag::NULL,
            tag:p.tag,
            extra_checks:CmpIgnored::new(p.extra_checks),
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
    pub const fn set_tag(mut self,tag:&'static Tag)->Self{
        self.tag=tag;
        self
    }

    /// Sets the Tag of the type used for reflection,this is not checked in layout checking.
    pub const fn set_reflection_tag(mut self,reflection_tag:&'static Tag)->Self{
        self.reflection_tag=reflection_tag;
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

    /// Whether this is a prefix-type(module or vtable).
    pub fn is_prefix_kind(&self)->bool{
        matches!(TLData::PrefixType{..}=self.data)
    }


    pub fn type_kind(&self)->TypeKind{
        self.abi_consts.kind
    }

    pub fn is_nonzero(&self)->bool{
        self.abi_consts.is_nonzero
    }

    pub fn get_utypeid(&self)->UTypeId{
        self.abi_consts.type_id.get()
    }
}





impl Display for TypeLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (package,version)=self.item_info.package_and_version();
        writeln!(
            f,
            "--- Type Layout ---\n\
             type:{ty}\n\
             size:{size} align:{align}\n\
             package:'{package}' version:'{version}'\n\
             line:{line} mod:{mod_path}",
            ty   =self.full_type(),
            size =self.size,
            align=self.alignment,
            package=package,
            version=version,
            line=self.item_info.line,
            mod_path=self.item_info.mod_path,
        )?;
        writeln!(f,"data:\n{}",self.data.to_string().left_padder(4))?;
        if !self.phantom_fields.is_empty() {
            writeln!(f,"Phantom fields:\n")?;
            for field in &*self.phantom_fields {
                write!(f,"{}",field.to_string().left_padder(4))?;
            }
        }
        writeln!(f,"Tag:\n{}",self.tag.to_string().left_padder(4))?;
        writeln!(f,"Reflection Tag:\n{}",self.reflection_tag.to_string().left_padder(4))?;
        let extra_checks=
            match self.extra_checks() {
                Some(x)=>x.to_string(),
                None=>"<nothing>".to_string(),
            };
        writeln!(f,"Extra checks:\n{}",extra_checks.left_padder(4))?;
        writeln!(f,"Repr attribute:{:?}",self.repr_attr)?;
        writeln!(f,"Module reflection mode:{:?}",self.mod_refl_mode)?;
        Ok(())
    }
}
