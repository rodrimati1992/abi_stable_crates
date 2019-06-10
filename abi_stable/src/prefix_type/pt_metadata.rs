use std::{
    borrow::Cow,
    slice,
};

#[allow(unused_imports)]
use crate::{
    abi_stability::{
        type_layout::{
            TypeLayout,TLField,TLData,TLPrefixType,TLDataDiscriminant,
            TLFieldsOrSlice,TLFieldsIterator,TLFields,
        },
    },
    std_types::*,
};

use super::{
    accessible_fields::{FieldAccessibility,IsAccessible},
    IsConditional,
};


#[allow(unused_imports)]
use core_extensions::SelfOps;


#[derive(Debug,Clone)]
pub(crate) struct PrefixTypeMetadata{
    /// This is the ammount of fields on the prefix of the struct,
    /// which is always the same for the same type,regardless of which library it comes from.
    pub prefix_field_count:usize,

    pub accessible_fields:FieldAccessibility,

    pub conditional_prefix_fields:&'static [IsConditional],

    pub fields:InitialFieldsOrMut,

    /// The layout of the struct,for error messages.
    pub layout:&'static TypeLayout,
}


impl PrefixTypeMetadata{
    #[cfg(test)]
    pub(crate) fn new(layout:&'static TypeLayout)->Self{
        match layout.data {
            TLData::PrefixType(prefix)=>
                Self::with_prefix_layout(prefix,layout),
            _=>panic!(
                "Attempting to construct a PrefixTypeMetadata from a \
                 TypeLayout of a non-prefix-type.\n\
                 Type:{}\nDataVariant:{:?}\nPackage:{}",
                 layout.full_type,
                 layout.data.as_discriminant(),
                 layout.package(),
            ),
        }
    }

    pub(crate) fn with_prefix_layout(prefix:TLPrefixType,layout:&'static TypeLayout)->Self{
        Self{
            fields:InitialFieldsOrMut::from(prefix.fields),
            accessible_fields:prefix.accessible_fields,
            conditional_prefix_fields:prefix.conditional_prefix_fields.as_slice(),
            prefix_field_count:prefix.first_suffix_field,
            layout,
        }
    }

    
    #[cfg(test)]
    pub(crate) fn assert_valid(&self){
        assert_eq!(self.layout.data.as_discriminant(),TLDataDiscriminant::PrefixType );
    }

    /// Returns the maximum prefix.Does not check that they are compatible.
    /// 
    /// # Preconditions
    /// 
    /// The prefixes must already have been checked for compatibility.
    #[cfg(test)]
    pub(crate) fn max(self,other:Self)->Self{
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
    pub(crate) fn min_max(self,other:Self)->(Self,Self){
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
    pub(crate) fn combine_fields_from(&mut self,other:&Self){
        let mut o_fields=other.fields.get_fields();

        let min_field_count=o_fields.len().min(self.fields.len());
        
        for (field_i,(t_acc,o_acc)) in 
            self.accessible_fields.iter_field_count(min_field_count)
                .zip(other.accessible_fields.iter_field_count(min_field_count))
                .enumerate() 
        {
            let o_field=o_fields.next().unwrap();
            if !t_acc.is_accessible() && o_acc.is_accessible() {
                let t_fields=self.fields.to_mut();

                t_fields[field_i]=o_field.into_owned();
            }
        }

        if min_field_count==self.fields.len() {
            let t_fields=self.fields.to_mut();
            
            for (i,o_field) in o_fields.enumerate() {
                let field_i=i+min_field_count;

                t_fields.push(o_field.into_owned());
                self.accessible_fields=
                    self.accessible_fields.set_accessibility(field_i,IsAccessible::Yes);
            }
        }
    }
}


/////////////////////////////////////////////////////////////////////////////////


#[derive(Debug,Clone)]
pub(crate) enum InitialFieldsOrMut{
    TLFields(TLFields),
    Slice(StaticSlice<TLField>),
    Mutable(Vec<TLField>),
}


impl From<TLFieldsOrSlice> for InitialFieldsOrMut{
    fn from(this:TLFieldsOrSlice)->Self{
        match this {
            TLFieldsOrSlice::TLFields(x)=>InitialFieldsOrMut::TLFields(x),
            TLFieldsOrSlice::Slice(x)=>InitialFieldsOrMut::Slice(x),

        }
    }
}


impl InitialFieldsOrMut{
    pub fn to_mut(&mut self)->&mut Vec<TLField>{
        match self {
            InitialFieldsOrMut::Mutable(x)=>x,
            this=>{
                let list=this.get_fields().map(Cow::into_owned).collect::<Vec<TLField>>();
                *this=InitialFieldsOrMut::Mutable(list);
                match this {
                    InitialFieldsOrMut::Mutable(x)=>x,
                    _=>unreachable!()
                }
            }
        }
    }
    pub fn get_fields(&self)->IFOMIter<'_>{
        match self {
            InitialFieldsOrMut::TLFields(x)=>IFOMIter::TLFields(x.get_fields()),
            InitialFieldsOrMut::Slice(x)=>IFOMIter::Slice(x.as_slice().iter()),
            InitialFieldsOrMut::Mutable(x)=>IFOMIter::Slice(x.iter()),
        }
    }
    pub fn len(&self)->usize{
        match self {
            InitialFieldsOrMut::TLFields(x)=>x.field_1to1.len(),
            InitialFieldsOrMut::Slice(x)=>x.len(),
            InitialFieldsOrMut::Mutable(x)=>x.len(),
        }
    }
}


#[repr(C)]
#[derive(Clone,Debug)]
pub enum IFOMIter<'a>{
    TLFields(TLFieldsIterator),
    Slice(slice::Iter<'a,TLField>),
}



impl<'a> Iterator for IFOMIter<'a>{
    type Item=Cow<'a,TLField>;

    fn next(&mut self)->Option<Cow<'a,TLField>>{
        match self {
            IFOMIter::TLFields(iter)=>iter.next().map(Cow::Owned),
            IFOMIter::Slice(iter)=>iter.next().map(Cow::Borrowed),
        }
    }

    fn size_hint(&self)->(usize,Option<usize>){
        match self {
            IFOMIter::TLFields(iter)=>iter.size_hint(),
            IFOMIter::Slice(iter)=>iter.size_hint(),
        }
    }
    fn count(self) -> usize {
        match self {
            IFOMIter::TLFields(iter)=>iter.count(),
            IFOMIter::Slice(iter)=>iter.count(),
        }
    }
}


impl<'a> std::iter::ExactSizeIterator for IFOMIter<'a>{}

