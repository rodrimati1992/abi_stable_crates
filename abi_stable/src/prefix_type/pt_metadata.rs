use std::{
    borrow::Cow,
};

use crate::{
    abi_stability::{
        type_layout::{TypeLayout,TLField,TLData,TLPrefixType,TLDataDiscriminant},
    },
};

use super::{
    accessible_fields::{FieldAccessibility,IsAccessible},
    IsConditional,
};


#[allow(unused_imports)]
use core_extensions::SelfOps;


#[derive(Debug,Clone)]
pub struct PrefixTypeMetadata{
    /// This is the ammount of fields on the prefix of the struct,
    /// which is always the same for the same type,regardless of which library it comes from.
    pub prefix_field_count:usize,

    pub accessible_fields:FieldAccessibility,

    pub conditional_prefix_fields:&'static [IsConditional],

    pub fields:Cow<'static,[TLField]>,

    /// The layout of the struct,for error messages.
    pub layout:&'static TypeLayout,
}
    

impl PrefixTypeMetadata{
    pub fn new(layout:&'static TypeLayout)->Self{
        match layout.data {
            TLData::PrefixType(prefix)=>
                Self::with_prefix_layout(prefix,layout),
            _=>panic!(
                "Attempting to construct a PrefixTypeMetadata from a \
                 TypeLayout of a non-prefix-type.\n\
                 Type:{}\nDataVariant:{:?}\nPackage:{}",
                 layout.full_type,
                 layout.data.discriminant(),
                 layout.package,
            ),
        }
    }

    pub fn with_prefix_layout(prefix:TLPrefixType,layout:&'static TypeLayout)->Self{
        Self{
            fields:prefix.fields.as_slice().into(),
            accessible_fields:prefix.accessible_fields,
            conditional_prefix_fields:prefix.conditional_prefix_fields.as_slice(),
            prefix_field_count:prefix.first_suffix_field,
            layout,
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


    /// Combines the fields from `other` into `self`,
    /// replacing any innaccessible field with one from `other`.
    ///
    /// # Preconditions
    ///
    /// This must be called after both were checked for compatibility,
    /// otherwise fields accessible in both `self` and `other` 
    /// won't be checked for compatibility or copied.
    pub fn combine_fields_from(&mut self,other:&Self){
        let o_fields=&*other.fields;

        let min_field_count=o_fields.len().min(self.fields.len());
        
        for (field_i,(t_acc,o_acc)) in 
            self.accessible_fields.iter_field_count(min_field_count)
                .zip(other.accessible_fields.iter_field_count(min_field_count))
                .enumerate() 
        {
            if !t_acc.is_accessible() && o_acc.is_accessible() {
                let t_fields=self.fields.to_mut();

                t_fields[field_i]=o_fields[field_i];
            }
        }

        if min_field_count==self.fields.len() {
            let t_fields=self.fields.to_mut();
            
            for (i,o_field) in o_fields[min_field_count..].iter().cloned().enumerate() {
                let field_i=i+min_field_count;

                t_fields.push(o_field);
                self.accessible_fields=
                    self.accessible_fields.set_accessibility(field_i,IsAccessible::Yes);
            }
        }
    }
}


