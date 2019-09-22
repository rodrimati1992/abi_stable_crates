/*!
Types for modeling the layout of a datatype
*/

use std::{
    cmp::{PartialEq,Eq},
    cell::RefCell,
    collections::HashSet,
    fmt::{self, Debug, Display, Formatter},
    mem::{self,ManuallyDrop},
};

use core_extensions::{matches,StringExt};

use crate::{
    abi_stability::{
        stable_abi_trait::{TypeLayoutCtor,AbiConsts},
        StoredExtraChecks,ExtraChecksStaticRef,
    },
    const_utils::log2_usize, 
    sabi_types::VersionStrings, 
    sabi_types::{CmpIgnored,Constructor,NulStr},
    std_types::{RStr,StaticStr,RSlice,UTypeId},
    prefix_type::{FieldAccessibility,FieldConditionality},
    reflection::ModReflMode,
};


mod construction;
pub mod data_structures;
mod shared_vars;
mod small_types;
mod printing;
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

pub use self::{
    construction::{
        _private_TypeLayoutDerive,
        _private_MonoTypeLayoutDerive,
        ItemInfo,
    },
    shared_vars::{
        SharedVars,
        MonoSharedVars,
    },
    small_types::{
        StartLen,
        StartLenRepr,
        StartLenConverter,
        OptionU16,
        OptionU8,
    },
    tl_data::{
        GenericTLData,
        MismatchedTLDataVariant,
        MonoTLData,
        TLData,
        TLDataDiscriminant,
    },
    tl_enums::{
        DiscriminantRepr,
        GenericTLEnum,
        GetVariantNames,
        IncompatibleWithNonExhaustive,
        IsExhaustive,
        MonoTLEnum,
        TLDiscriminant,
        TLDiscriminants,
        TLEnum,
        TLNonExhaustive,
    },
    tl_field::{
        CompTLField,
        CompTLFieldRepr,
        TLField,
    },
    tl_fields::{
        CompTLFields,
        TLFields,
        TLFieldsIterator,
    },
    tl_functions::{
        CompTLFunction,
        TLFunctionIter,
        TLFunctions,
        TLFunctionSlice,
    },
    tl_lifetimes::{
        LifetimeArrayOrSlice,
        LifetimeIndex,
        LifetimeIndexArray,
        LifetimeIndexPair,
        LifetimeIndexPairRepr,
        LifetimeRange,
    },
    tl_multi_tl::{
        TypeLayoutRange,
        MultipleTypeLayouts,
    },
    tl_other::{
        ChainOnce,
        CompGenericParams,
        CustomPrimitive,
        FmtFullType,
        GenericParams,
        GetParamNames,
        ModPath,
        ReprAttr,
        TLFieldOrFunction,
        TLFunction,
        TLPrimitive,
    },
    tl_prefix::{
        GenericTLPrefixType,
        MonoTLPrefixType,
        TLPrefixType,
    },
    tl_reflection::{
        CompFieldAccessor,
        FieldAccessor,
    },
    tagging::Tag,
};


////////////////////////////////////////////////////////////////////////////////


/// The layout of a type,
/// also includes metadata about where the type was defined.
#[repr(C)]
#[derive(Copy, Clone,StableAbi)]
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
    mono:&'static MonoTypeLayout,
    
    /// Whether the type uses non-zero value optimization,
    /// if true then an Option<Self> implements StableAbi.
    is_nonzero: bool,    

    /// The alignment of the type represented as (1 << self.alignment_power_of_two).
    alignment_power_of_two: u8,

    /// The size of the type
    size: usize,
    
    tag:Option<&'static Tag>,

    data:GenericTLData,
    
    /// A json-like data structure used to add extra checks.
    extra_checks:CmpIgnored<Option<&'static ManuallyDrop<StoredExtraChecks>>>,

    /// Equivalent to the UTypeId returned by the function in Constructor.
    type_id:Constructor<UTypeId>,
}

unsafe impl Send for TypeLayout{}
unsafe impl Sync for TypeLayout{}

unsafe impl Send for MonoTypeLayout{}
unsafe impl Sync for MonoTypeLayout{}


///////////////////////////

impl TypeLayout {

    pub(crate) const fn from_std<T>(
        shared_vars: &'static SharedVars,
        mono:&'static MonoTypeLayout,
        abi_consts:AbiConsts,
        data: GenericTLData,
    ) -> Self {
        Self {
            shared_vars,
            mono,
            is_nonzero:abi_consts.is_nonzero,
            type_id:abi_consts.type_id,
            alignment_power_of_two: log2_usize(mem::align_of::<T>()),
            size: mem::size_of::<T>(),
            data,
            extra_checks:CmpIgnored::new(None),
            tag:None,
        }
    }

    #[doc(hidden)]
    pub const fn from_derive<T>(p: _private_TypeLayoutDerive) -> Self {
        Self {
            shared_vars: p.shared_vars,
            mono: p.mono,
            is_nonzero: p.abi_consts.is_nonzero,
            type_id: p.abi_consts.type_id,
            alignment_power_of_two: log2_usize(mem::align_of::<T>()),
            size: mem::size_of::<T>(),
            data: p.data,
            extra_checks:CmpIgnored::new(p.extra_checks),
            tag:p.tag,
        }
    }

    pub const fn shared_vars(&self)->&'static SharedVars{
        self.shared_vars
    }

    pub(crate) fn full_type(&self) -> FmtFullType {
        FmtFullType{
            name: self.mono.name(),
            generics: self.generics(),
            primitive: self.mono.data.to_primitive(),
        }
    }

    /// Gets the package of the type.
    pub fn package_and_version(&self)->(StaticStr,VersionStrings){
        let (package,version)=self.item_info().package_and_version();

        (
            StaticStr::new(package),
            VersionStrings::new(version)
        )
    }

    /// Gets the package of the type.
    pub fn package(&self)->StaticStr{
        let (package,_)=self.item_info().package_and_version();
        StaticStr::new(package)
    }

    /// Gets the package version for the package of type.
    pub fn package_version(&self)->VersionStrings{
        let (_,version)=self.item_info().package_and_version();
        VersionStrings::new(version)
    }

    /// Gets in which line the type was defined.
    pub fn line(&self)->u32{
        self.item_info().line
    }

    /// Gets the full path to the module where the type was defined.
    pub fn mod_path(&self)->ModPath{
        self.item_info().mod_path
    }

    #[inline]
    pub fn extra_checks(&self)->Option<ExtraChecksStaticRef>{
        self.extra_checks.value.map(|x| x.sabi_reborrow() )
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
    pub fn get_fields(&self)->Option<TLFields>{
        let fields=self.mono.get_fields()?;
        Some(fields.expand(self.shared_vars))
    }

    /// Whether this is a prefix-type(module or vtable).
    pub fn is_prefix_kind(&self)->bool{
        matches!(GenericTLData::PrefixType{..}=self.data)
    }

    #[inline]
    pub fn name(&self)->&'static str{
        self.mono.name()
    }

    #[inline]
    pub fn is_nonzero(&self)->bool{
        self.is_nonzero
    }

    #[doc(hidden)]
    #[cfg(test)]
    pub const fn _set_is_nonzero(mut self,is_nonzero:bool)->Self{
        self.is_nonzero=is_nonzero;
        self
    }

    #[doc(hidden)]
    #[cfg(test)]
    pub const fn _set_extra_checks(
        mut self,
        extra_checks:CmpIgnored<Option<&'static ManuallyDrop<StoredExtraChecks>>>
    )->Self{
        self.extra_checks=extra_checks;
        self
    }

    #[doc(hidden)]
    #[cfg(test)]
    pub const fn _set_type_id(
        mut self,
        type_id:Constructor<UTypeId>,
    )->Self{
        self.type_id=type_id;
        self
    }

    #[inline]
    pub fn get_utypeid(&self)->UTypeId{
        self.type_id.get()
    }

    #[inline]
    pub fn item_info(&self)->&ItemInfo{
        &self.mono.item_info()
    }

    #[inline]
    pub fn alignment(&self)->usize{
        1_usize << (self.alignment_power_of_two as u32)
    }

    #[inline]
    pub fn size(&self)->usize{
        self.size
    }

    pub fn tag(&self)->&'static Tag{
        self.tag.unwrap_or(Tag::NULL)
    }

    pub fn repr_attr(&self)->ReprAttr{
        self.mono.repr_attr()
    }

    pub fn mod_refl_mode(&self)->ModReflMode{
        self.mono.mod_refl_mode()
    }

    pub fn data(&self)-> TLData {
        self.mono.data
            .expand(self.data,self.shared_vars)
            .unwrap_or_else(|e|{
                panic!("\nError inside of '{}' type \n{}",self.full_type(),e);
            })
    }

    pub fn data_discriminant(&self)-> TLDataDiscriminant {
        self.mono.data.as_discriminant()
    }

    #[inline]
    pub fn phantom_fields(&self)->TLFields{
        unsafe{
            let slice=std::slice::from_raw_parts(
                self.mono.phantom_fields,
                self.mono.phantom_fields_len as usize,
            );
            TLFields::from_fields(slice,self.shared_vars)
        }
    }

    pub fn generics(&self)->GenericParams{
        self.mono.generics.expand(self.shared_vars)
    }

    pub fn mono_type_layout(&self)->&MonoTypeLayout{
        &self.mono
    }
}


impl PartialEq for TypeLayout{
    fn eq(&self,other:&TypeLayout)->bool{
        self.get_utypeid()==other.get_utypeid()
    }
}


impl Eq for TypeLayout{}


impl Display for TypeLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (package,version)=self.item_info().package_and_version();
        writeln!(
            f,
            "--- Type Layout ---\n\
             type:{ty}\n\
             size:{size} align:{align}\n\
             package:'{package}' version:'{version}'\n\
             line:{line} mod:{mod_path}",
            ty   =self.full_type(),
            size =self.size(),
            align=self.alignment(),
            package=package,
            version=version,
            line=self.item_info().line,
            mod_path=self.item_info().mod_path,
        )?;
        writeln!(f,"data:\n{}",self.data().to_string().left_padder(4))?;
        let phantom_fields=self.phantom_fields();
        if !phantom_fields.is_empty() {
            writeln!(f,"Phantom fields:\n")?;
            for field in phantom_fields {
                write!(f,"{}",field.to_string().left_padder(4))?;
            }
        }
        writeln!(f,"Tag:\n{}",self.tag().to_string().left_padder(4))?;
        let extra_checks=
            match self.extra_checks() {
                Some(x)=>x.to_string(),
                None=>"<nothing>".to_string(),
            };
        writeln!(f,"Extra checks:\n{}",extra_checks.left_padder(4))?;
        writeln!(f,"Repr attribute:{:?}",self.repr_attr())?;
        writeln!(f,"Module reflection mode:{:?}",self.mod_refl_mode())?;
        Ok(())
    }
}


////////////////////////////////////////////////////////////////////////////////


/// The data in the type layout that does not depend on generic parameters.
///
/// This is stored in a static for every type that derives StableAbi.
#[repr(C)]
#[derive(Copy, Clone,StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct MonoTypeLayout{
    shared_vars:MonoSharedVars,

    /// The name of the type.
    name: *const u8,

    /// Contains information about where the type was defined.
    ///
    item_info:CmpIgnored<ItemInfo>,

    /// What kind of type this is,Primitive/Struct/Enum/PrefixType.
    data: MonoTLData,
    /// The generic parameters of the type
    generics: CompGenericParams,

    /// Phantom fields,which don't have a runtime component(they aren't stored anywhere),
    /// and are checked in layout checking.
    phantom_fields: *const CompTLField,
    phantom_fields_len: u8,

    /// The representation attribute(s) of this type.
    repr_attr:ReprAttr,

    /// How this type is treated when interpreted as a module.
    mod_refl_mode:ModReflMode,
    
    name_len: u16,
}


impl MonoTypeLayout{
    pub const fn new(
        shared_vars:MonoSharedVars,
        name: RStr<'static>,
        item_info:ItemInfo,
        data: MonoTLData,
        generics: CompGenericParams,
        repr_attr:ReprAttr,
        mod_refl_mode:ModReflMode,
        phantom_fields:RSlice<'static,CompTLField>,
    )->Self{
        Self{
            shared_vars,
            name    :name.as_ptr(),
            name_len:name.len() as u16,
            item_info:CmpIgnored::new(item_info),
            data,
            generics,
            repr_attr,
            mod_refl_mode,
            phantom_fields: phantom_fields.as_ptr(),
            phantom_fields_len: phantom_fields.len() as u8,
        }
    }

    pub const fn from_derive(p:_private_MonoTypeLayoutDerive)->Self{
        Self{
            name    : p.name.as_ptr(),
            name_len: p.name.len() as u16,
            phantom_fields    : p.phantom_fields.as_ptr() 
                as *const CompTLFieldRepr 
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

    pub fn name(&self)->&'static str{
        unsafe{
            let slic=std::slice::from_raw_parts( self.name, self.name_len as usize );
            std::str::from_utf8_unchecked(slic)
        }
    }

    pub const fn repr_attr(&self)->ReprAttr{
        self.repr_attr
    }

    pub const fn mod_refl_mode(&self)->ModReflMode{
        self.mod_refl_mode
    }

    pub const fn item_info(&self)->&ItemInfo{
        &self.item_info.value
    }

    pub const fn shared_vars(&self)->&MonoSharedVars{
        &self.shared_vars
    }

    pub fn get_fields(&self)->Option<CompTLFields>{
        match self.data {
            MonoTLData::Primitive{..}=>return None,
            MonoTLData::Opaque=>return None,
            MonoTLData::Struct{fields}=>Some(fields),
            MonoTLData::Union{fields}=>Some(fields),
            MonoTLData::Enum (tlenum)=>Some(tlenum.fields),
            MonoTLData::PrefixType(prefix)=>Some(prefix.fields),
        }
    }

    pub fn field_names(&self)->impl Iterator<Item=&'static str>+'static{
        self.get_fields()
            .unwrap_or(CompTLFields::EMPTY)
            .field_names( &self.shared_vars )
    }

    pub fn get_field_name(&self,index:usize)->Option<&'static str>{
        self.get_fields()
            .unwrap_or(CompTLFields::EMPTY)
            .get_field_name( index, &self.shared_vars )
    }
}


impl Debug for MonoTypeLayout{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_struct("MonoTypeLayout")
        .field("name",&self.name())
        .field("item_info",self.item_info())
        .field("repr_attr",&self.repr_attr())
        .field("mod_refl_mode",&self.mod_refl_mode())
        .finish()
    }
}