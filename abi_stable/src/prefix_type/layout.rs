use crate::{
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
}


//////////////////////////////////////////////////////////////


impl PTStructLayout{
    pub const fn new(
        generics:RStr<'static>,
        mono_layout:&'static MonoTypeLayout,
    )->Self{
        Self{
            generics,
            mono_layout,
        }
    }

    #[inline]
    pub fn get_field_names(&self)->impl Iterator<Item=&'static str>{
        self.mono_layout.field_names()
    }

    #[inline]
    pub fn get_field_names_vec(&self)->Vec<&'static str>{
        self.mono_layout.field_names().collect()
    }

    #[inline]
    pub fn get_field_name(&self,field_index:usize)->Option<&'static str>{
        self.mono_layout.get_field_name(field_index)
    }
}
