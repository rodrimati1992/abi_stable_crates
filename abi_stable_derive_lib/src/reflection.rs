use proc_macro2::TokenStream;
use quote::{quote,ToTokens};


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
            FieldAccessor::Direct=>quote!( __FieldAccessor::Direct ),
            FieldAccessor::Method=>quote!( __FieldAccessor::Method ),
            FieldAccessor::MethodOption=>quote!( __FieldAccessor::MethodOption ),
            FieldAccessor::Opaque=>quote!( __FieldAccessor::Opaque ),
        }.to_tokens(ts);
    }
}



#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum ModReflMode<T>{
    Module,
    Opaque,
    DelegateDeref(T),
}