use proc_macro2::TokenStream;
use quote::{quote,ToTokens};


/// Whether this is a module whose definition can be reflected on at runtime,
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum ModReflMode{
    /// For modules that are reflected on at runtime..
    Module,
    /// For types whose layout can't be iterated over.
    Opaque,
    /// Delegates the layout to some other type,this is generally for references.
    Delegate,
}


impl Default for ModReflMode{
    fn default()->Self{
        ModReflMode::Opaque
    }
}


impl ToTokens for ModReflMode{
    fn to_tokens(&self, ts: &mut TokenStream) {
        match *self {
            ModReflMode::Module  =>quote!( _sabi_reexports::ModReflMode::Module ),
            ModReflMode::Opaque  =>quote!( _sabi_reexports::ModReflMode::Opaque ),
            ModReflMode::Delegate=>quote!( _sabi_reexports::ModReflMode::Delegate ),
        }.to_tokens(ts);
    }
}