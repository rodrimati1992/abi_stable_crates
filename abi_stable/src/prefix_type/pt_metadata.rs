use std::{
    borrow::Cow,
};

use crate::{
    abi_stability::{
        type_layout::{TypeLayout,TLField,TLData,TLDataDiscriminant},
        StableAbi,
    },
};


use core_extensions::SelfOps;


#[derive(Debug,Clone)]
pub struct PrefixTypeMetadata{
    /// This is the ammount of fields on the prefix of the struct,
    /// which is always the same for the same type,regardless of which library it comes from.
    pub prefix_field_count:usize,

    pub fields:Cow<'static,[TLField]>,

    pub accessible_fields:FieldAccessibility,

    /// The layout of the struct,for error messages.
    pub layout:&'static TypeLayout,
}


impl PrefixTypeMetadata{
    pub fn new(layout:&'static TypeLayout)->Self{
        let (first_suffix_field,accessible_fields,fields)=match layout.data {
            TLData::PrefixType{first_suffix_field,accessible_fields,fields}=>
                (first_suffix_field,accessible_fields,fields),
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
            fields:fields.as_slice().into(),
            prefix_field_count:first_suffix_field,
            accessible_fields,
            layout,
        }
    }

    pub fn invalid()->Self{
        const LAYOUT:&'static TypeLayout=<() as StableAbi>::ABI_INFO.get().layout;

        Self{
            prefix_field_count:0,
            fields:Cow::Borrowed(&[]),
            accessible_fields:FieldAccessibility::with_field_count(0),
            layout:LAYOUT,
        }
    }

    pub fn assert_valid(&self){
        assert_eq!(self.layout.data.discriminant(),TLDataDiscriminant::PrefixType );
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

