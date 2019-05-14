use proc_macro2::TokenStream;
use quote::{quote,ToTokens};


#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FieldAccessor {
    /// Accessible with `self.field_name`
    Direct,
    /// Accessible with `fn field_name(&self)->FieldType`
    Method,
    /// Accessible with `fn field_name(&self)->Option<FieldType>`
    MethodOption,
    /// This field is completely inaccessible.
    Opaque,
}


impl ToTokens for FieldAccessor{
    fn to_tokens(&self, ts: &mut TokenStream) {
        match *self {
            FieldAccessor::Direct=>quote!( _sabi_reexports::FieldAccessor::Direct ),
            FieldAccessor::Method=>quote!( _sabi_reexports::FieldAccessor::Method ),
            FieldAccessor::MethodOption=>quote!( _sabi_reexports::FieldAccessor::MethodOption ),
            FieldAccessor::Opaque=>quote!( _sabi_reexports::FieldAccessor::Opaque ),
        }.to_tokens(ts);
    }
}
