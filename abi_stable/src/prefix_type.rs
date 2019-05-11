/*!
Types,traits,and functions used by prefix-types.

*/

use std::{
    borrow::Cow,
    marker::PhantomData,
};

use crate::{
    marker_type::NotCopyNotClone,
    utils::leak_value,
};


use core_extensions::SelfOps;


mod accessible_fields;
mod layout;
mod pt_metadata;

pub use self::{
    accessible_fields::{FieldAccessibility,IsAccessible},
    layout::{PTStructLayout,PTStructLayoutParams,PTField},
};

pub(crate) use self::pt_metadata::PrefixTypeMetadata;


/// For types deriving `StableAbi` with `#[sabi(kind(Prefix(..)))]`.
pub unsafe trait PrefixTypeTrait:Sized{
    /// Just the metadata of Self,for passing to `WithMetadata::new`,
    /// with `WithMetadata::new(PrefixTypeTrait::METADATA,value)`
    const METADATA:WithMetadataFor<Self,Self::Prefix>=WithMetadataFor{
        inner:WithMetadata_{
            _prefix_type_field_acc:Self::PT_FIELD_ACCESSIBILITY,
            _prefix_type_layout:Self::PT_LAYOUT,
            original:(),
            _marker:PhantomData,
            unbounds:NotCopyNotClone,
        },
        _marker:PhantomData,
    };

    /// Describes the layout of the struct,exclusively for use in error messages.
    const PT_LAYOUT:&'static PTStructLayout;

    /// A bit array,where the bit at the field index represents whether that 
    /// field is accessible.
    const PT_FIELD_ACCESSIBILITY:FieldAccessibility;

    #[doc(hidden)]
    // Whether each individual field in the prefix is conditional.
    //
    // This is checked in layout checking to ensure that 
    // both sides agree on whether each field in the prefix is conditional,
    const PT_COND_PREFIX_FIELDS:&'static [IsConditional];

    /**
A type only accessible through a shared reference.

The fields after the `#[sabi(last_prefix_field)]` attribute are 
only potentially accessible in their `<field_name>` methods,
since their existence has to be checked at runtime.
This is because multiple versions of the library may be loaded,
where in some of them those fields don't exist.

*/
    type Prefix;

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
/// Can be converted to the `PrefixTypeTrait::Prefix` of T with the `as_prefix` method.
#[repr(C,align(4))]
pub struct WithMetadata_<T,P>{
    /// A bit array,where the bit at field index represents whether a field is accessible.
    #[inline(doc)]
    pub _prefix_type_field_acc:FieldAccessibility,
    #[inline(doc)]
    pub _prefix_type_layout:&'static PTStructLayout,
    pub original:T,
    _marker:PhantomData<P>,
    // WithMetadata will never implement Copy or Clone.
    // This type does not implement those traits because it is a field of 
    // all `<prefix_struct>` types,and it's UB prone for those types to implement Copy or Clone.
    unbounds:NotCopyNotClone,
}


impl<T,P> WithMetadata_<T,P> {
    /// Constructs Self with `WithMetadata::new(PrefixTypeTrait::METADATA,value)`
    pub const fn new(metadata:WithMetadataFor<T,P>,value:T)->Self{
        Self{
            _prefix_type_field_acc  :metadata.inner._prefix_type_field_acc,
            _prefix_type_layout     :metadata.inner._prefix_type_layout,
            original:value,
            _marker:PhantomData,
            unbounds:NotCopyNotClone,
        }
    }

    /// Converts this WithMetadata<T,P> to a `<prefix_struct>` type.
    pub fn as_prefix(&self)->&P {
        unsafe{
            &*self.as_prefix_raw()
        }
    }
    
    /// Converts this WithMetadata<T,P> to a `<prefix_struct>` type.
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
///
/// `P` is guaranteed to be <T as PrefixTypeTrait>::Prefix,
/// it is a type parameter to get around limitations of `const fn` as of Rust 1.34.
#[repr(C)]
pub struct WithMetadataFor<T,P>{
    inner:WithMetadata_<(),()>,
    _marker:PhantomData<(T,P)>
}


////////////////////////////////////////////////////////////////////////////////



/// Whether a field is conditional,
/// whether it has a `#[sabi(accessible_if=" expression ")]` attribute or not.
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
#[repr(C)]
pub enum IsConditional{
    No=0,
    Yes=1,
}

impl IsConditional{
    pub const fn new(is_accessible:bool)->Self{
        [IsConditional::No,IsConditional::Yes][is_accessible as usize]
    }
    pub const fn is_conditional(self)->bool{
        self as usize!=0
    }
}



////////////////////////////////////////////////////////////////////////////////


/// Used to panic with an error message informing the user that a field 
/// is expected to be on the `T` type when it's not.
#[cold]
#[inline(never)]
pub fn panic_on_missing_field_ty<T>(field_index:usize,actual_layout:&'static PTStructLayout)->!
where T:PrefixTypeTrait
{
    #[inline(never)]
    pub fn inner(
            field_index:usize,
            expected_layout:&'static PTStructLayout,
            actual_layout:&'static PTStructLayout
    )->!{
        let field=expected_layout.fields[field_index];
        panic_on_missing_field_val(Some(field_index),field,expected_layout,actual_layout)
    }


    inner(field_index,T::PT_LAYOUT,actual_layout)
}


/// Used to panic with an error message informing the user that a field 
/// is expected to be on the `T` type when it's not.
#[cold]
#[inline(never)]
pub fn panic_on_missing_fieldname<T,FieldTy>(
    &fieldname:&&'static str,
    actual_layout:&'static PTStructLayout
)->!
where T:PrefixTypeTrait
{
    #[inline(never)]
    fn inner(
        expected_layout:&'static PTStructLayout,
        actual_layout:&'static PTStructLayout,
        default_field:PTField,
    )->! {
        let fieldname=default_field.name.as_str();
        let field_index=expected_layout.fields.iter().position(|f| f.name.as_str()==fieldname );
        let field=match field_index {
            Some(field_index)=>expected_layout.fields[field_index],
            None=>default_field,
        };
        panic_on_missing_field_val(field_index,field,expected_layout,actual_layout)
    }

    inner(
        T::PT_LAYOUT,
        actual_layout,
        PTField::new::<FieldTy>(fieldname,"<unavailable>"),
    )
    
}



/// Used to panic with an error message informing the user that a field 
/// is expected to be on `expected` when it's not.
#[inline(never)]
pub fn panic_on_missing_field_val(
    field_index:Option<usize>,
    field:PTField,
    expected:&'static PTStructLayout,
    actual:&'static PTStructLayout,
)->! {
    

    panic!("\n
Attempting to access nonexistent field:
    index:{index} 
    named:{field_named}
    type(as declared):{field_type}

Inside of:{struct_name}{struct_generics}

Package:'{package}' 

Expected:
    Version:{expected_package_version} (or compatible version number)
    Field count:{expected_field_count}

Found:
    Version:{actual_package_version}
    Field count:{actual_field_count}

\n",
        index=field_index.map_or(Cow::Borrowed("<unavailable>"),|x| x.to_string().into() ),
        field_named=field.name.as_str(),
        field_type=field.ty.as_str(),
        struct_name=expected.name.as_str(),
        struct_generics=expected.generics.as_str(),
        package=expected.package,
        
        expected_package_version =expected.package_version ,
        expected_field_count=expected.fields.len(),
        
        actual_package_version =actual.package_version ,
        actual_field_count=actual.fields.len(),
    );
}