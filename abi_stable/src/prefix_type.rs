/*!
Types,traits,and functions used by prefix-types.

*/

use std::marker::PhantomData;

use crate::{
    abi_stability::{
        type_layout::{TypeLayout,TLField,TLData},
        stable_abi_trait::SharedStableAbi,
    },
    std_types::StaticSlice,
    utils::leak_value,
};


use core_extensions::SelfOps;



/// For types deriving `StableAbi` with `#[sabi(kind(Prefix(..)))]`.
pub unsafe trait PrefixTypeTrait:Sized{
    /// Just the metadata of Self,for passing to `WithMetadata::new`,
    /// with `WithMetadata::new(PrefixTypeTrait::METADATA,value)`
    const METADATA:WithMetadataFor<Self,Self::Prefix>=WithMetadataFor{
        _prefix_type_field_count:Self::PREFIX_TYPE_COUNT,
        _prefix_type_layout:<Self::Prefix as SharedStableAbi>::S_ABI_INFO.get().layout,
        _marker:PhantomData,
    };

    const PREFIX_TYPE_COUNT:usize;

    /**
A type only accessible through a shared reference,
with access to the fields of Self at and before `#[sabi(last_prefix_field)]`.

The fields after the `#[sabi(last_prefix_field)]` attribute are 
only accessible through `<field_name>` methods,
since their existence has to be checked at runtime.
This is because multiple versions of the library may be loaded,
where in some of them those fields don't exist.

*/
    type Prefix:SharedStableAbi;

    /// Converts `self` to a `WithMetadata<Self>`,.
    fn into_with_metadata(self)->WithMetadata<Self>{
        WithMetadata::new(Self::METADATA,self)
    }
    
    /// Convers `Self` to its `WithMetadata<Self>`,
    /// then leaks it and casts it to `&'a Self::Prefix`.
    fn leak_into_prefix<'a>(self)->&'a Self::Prefix
    where 
        Self:'a,
        Self::Prefix:'a
    {
        self.into_with_metadata()
            .piped(leak_value)
            .as_prefix()
    }
}

////////////////////////////////////////////////////////////////////////////////


/// Type alias for WithMetadata_<T,P> that 
/// passes <T as PrefixTypeTrait>::Prefix as the second type parameter.
pub type WithMetadata<T>=
    WithMetadata_<T,<T as PrefixTypeTrait>::Prefix>;


/// Wraps a prefix-type,with extra metadata about field count and layout.
///
/// Can be converted to the `PrefixTypeTrait::Prefix` of T with the `as_prefix` methods.
#[repr(C)]
pub struct WithMetadata_<T,P>{
    #[inline(doc)]
    pub _prefix_type_field_count:usize,
    #[inline(doc)]
    pub _prefix_type_layout:&'static TypeLayout,
    pub original:T,
    _marker:PhantomData<P>,
}


impl<T,P> WithMetadata_<T,P> {
    /// Constructs Self with `WithMetadata::new(PrefixTypeTrait::METADATA,value)`
    pub const fn new(metadata:WithMetadataFor<T,P>,value:T)->Self{
        Self{
            _prefix_type_field_count:metadata._prefix_type_field_count,
            _prefix_type_layout     :metadata._prefix_type_layout,
            original:value,
            _marker:PhantomData,
        }
    }

    /// Converts this WithMetadata<T,P> to a `*_Prefix` type.
    pub fn as_prefix(&self)->&P {
        unsafe{
            &*self.as_prefix_raw()
        }
    }
    
    /// Converts this WithMetadata<T,P> to a `*_Prefix` type.
    /// Use this if you need to implement nested vtables at compile-time.
    pub const fn as_prefix_raw(&self)->*const P {
        unsafe{
            self as *const Self as *const P
        }
    }
}


////////////////////////////////////////////////////////////////////////////////


/// The prefix-type metadata for `T`.
/// This is only constructed in PrefixTypeTrait::METADATA.
#[repr(C)]
pub struct WithMetadataFor<T,P>{
    #[inline(doc)]
    _prefix_type_field_count:usize,
    #[inline(doc)]
    _prefix_type_layout:&'static TypeLayout,
    _marker:PhantomData<fn()->(T,P)>,
}


////////////////////////////////////////////////////////////////////////////////


#[derive(Debug,Copy,Clone)]
pub struct PrefixTypeMetadata{
    /// This is the ammount of fields on the prefix of the struct,
    /// which is always the same for the same type,regardless of which library it comes from.
    pub prefix_field_count:usize,

    pub fields:StaticSlice<TLField>,

    /// The layout of the struct,for error messages.
    pub layout:&'static TypeLayout,
}


impl PrefixTypeMetadata{
    pub fn new(layout:&'static TypeLayout)->Self{
        let (first_suffix_field,fields)=match layout.data {
            TLData::PrefixType{first_suffix_field,fields}=>
                (first_suffix_field,fields),
            _=>panic!(
                "Attempting to construct a PrefixTypeMetadata from a \
                 TypeLayout of a non-prefix-type.\n\
                 Type:{}\nDataVariant:{:?}\nPackage:{}",
                 layout.full_type,
                 layout.data.discriminant(),
                 layout.package,
            ),
        };
        Self{
            fields:fields,
            prefix_field_count:first_suffix_field,
            layout,
        }
    }

    /// Returns the maximum prefix.Does not check that they are compatible.
    /// 
    /// # Preconditions
    /// 
    /// The prefixes must already have been checked for compatibility.
    pub fn max(self,other:Self)->Self{
        if self.fields.len() < other.fields.len() {
            other
        }else{
            self
        }
    }
    /// Returns the minimum and maximum prefix.Does not check that they are compatible.
    /// 
    /// # Preconditions
    /// 
    /// The prefixes must already have been checked for compatibility.
    pub fn min_max(self,other:Self)->(Self,Self){
        if self.fields.len() < other.fields.len() {
            (self,other)
        }else{
            (other,self)
        }
    }
}


/// Used to panic with an error message informing the user that a field 
/// is expected to be on the `T` type when it's not.
#[cold]
#[inline(never)]
pub fn panic_on_missing_field_ty<T>(field_index:usize,actual_layout:&'static TypeLayout)->!
where T:SharedStableAbi
{
    panic_on_missing_field_val(field_index,T::S_ABI_INFO.get().layout,actual_layout)
}


/// Used to panic with an error message informing the user that a field 
/// is expected to be on `expected` when it's not.
#[cold]
#[inline(never)]
pub fn panic_on_missing_field_val(
    field_index:usize,
    expected:&'static TypeLayout,
    actual:&'static TypeLayout,
)->! {
    let expected=PrefixTypeMetadata::new(expected);
    let actual=PrefixTypeMetadata::new(actual);

    let field=expected.fields[field_index];

    panic!("\n
Attempting to access nonexistent field:
    index:{index} 
    named:{field_named}
    type:{field_type}

Type:{struct_type}

Package:'{package}' 

Expected:
    Version(expected compatible):{expected_package_version}
    Field count:{expected_field_count}

Found:
    Version:{actual_package_version}
    Field count:{actual_field_count}

\n",
        index=field_index,
        field_named=field.name.as_str(),
        field_type=field.abi_info.get().layout.full_type,
        struct_type=expected.layout.full_type,
        package=expected.layout.package,
        
        expected_package_version =expected.layout.package_version ,
        expected_field_count=expected.fields.len(),
        
        actual_package_version =actual.layout.package_version ,
        actual_field_count=actual.fields.len(),
    );
}