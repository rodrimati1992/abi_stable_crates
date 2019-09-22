use super::FieldConditionality;

use crate::{
    sabi_types::{CmpIgnored,VersionStrings},
    std_types::RStr,
    type_layout::MonoTypeLayout,
};




/// Represents the layout of a prefix-type,for use in error messages.
#[repr(C)]
#[derive(Debug, Copy, Clone, StableAbi)]
// #[derive(Debug, Copy, Clone, PartialEq, StableAbi)]
pub struct PTStructLayout {
    pub generics:RStr<'static>,
    pub mono_layout:&'static MonoTypeLayout,
    pub field_names:RStr<'static>,
    /// Describes whether prefix fields are conditional or not.
    ///
    /// "prefix field" is every field at and before the one with the 
    /// `#[sabi(last_prefix_field)]` attribute.
    ///
    /// A field is conditional if it has the 
    /// `#[sabi(accessible_if=" expression ")]` attribute on it.
    pub prefix_field_conditionality:FieldConditionality,
}


//////////////////////////////////////////////////////////////


impl PTStructLayout{
    pub const fn new(
        generics:RStr<'static>,
        mono_layout:&'static MonoTypeLayout,
        field_names:RStr<'static>,
        prefix_field_conditionality:u64,
    )->Self{
        Self{
            generics,
            mono_layout,
            field_names,
            prefix_field_conditionality:
                FieldConditionality::from_u64(prefix_field_conditionality),
        }
    }

    pub fn get_field_names(&self)->impl Iterator<Item=&'static str>{
        self.field_names.as_str().split(';').filter(|x| !x.is_empty() )
    }

    pub fn get_field_names_vec(&self)->Vec<&'static str>{
        self.get_field_names().collect()
    }

    pub fn get_field_name(&self,field_index:usize)->Option<&'static str>{
        self.get_field_names().nth(field_index)
    }
}
