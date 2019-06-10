use proc_macro2::TokenStream;
use quote::{quote,ToTokens};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FieldAccessor<'a> {
    /// Accessible with `self.field_name`
    Direct,
    /// Accessible with `fn field_name(&self)->FieldType`
    Method{
        name:Option<&'a str>,
    },
    /// Accessible with `fn field_name(&self)->Option<FieldType>`
    MethodOption,
    /// This field is completely inaccessible.
    Opaque,
}


impl ToTokens for FieldAccessor<'_>{
    fn to_tokens(&self, ts: &mut TokenStream) {
        match *self {
            FieldAccessor::Direct=>
                quote!( __FieldAccessor::Direct ),
            FieldAccessor::Method{name:None}=>
                quote!( 
                    __FieldAccessor::Method{
                        name:None 
                    } 
                ),
            FieldAccessor::Method{name:Some(name)}=>
                quote!( 
                    __FieldAccessor::method_named(&__StaticStr::new(#name))
                ),
            FieldAccessor::MethodOption=>
                quote!( __FieldAccessor::MethodOption ),
            FieldAccessor::Opaque=>
                quote!( __FieldAccessor::Opaque ),
        }.to_tokens(ts);
    }
}



#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum ModReflMode<T>{
    Module,
    Opaque,
    DelegateDeref(T),
}