/*!
An implementation detail of abi_stable.
*/

#![recursion_limit="192"]
//#![deny(unused_variables)]
#![deny(unreachable_patterns)]
#![deny(unused_doc_comments)]
#![deny(unconditional_recursion)]

extern crate core_extensions;

extern crate proc_macro;

#[macro_use]
mod macros;

#[macro_use]
mod utils;

mod arenas;
mod attribute_parsing;
mod common_tokens;
mod composite_collections;
mod constants;
mod datastructure;
mod fn_pointer_extractor;
mod gen_params_in;
mod get_static_equivalent;
mod ignored_wrapper;
mod impl_interfacetype;
mod lifetimes;
mod mangle_library_getter;
mod my_visibility;
mod parse_utils;
mod sabi_extern_fn_impl;
mod set_span_visitor;
mod to_token_fn;
mod workaround;

#[doc(hidden)]
pub mod stable_abi;

#[doc(hidden)]
pub mod sabi_trait;



use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;

use syn::{DeriveInput,ItemFn};

use quote::{quote, ToTokens, quote_spanned};

#[allow(unused_imports)]
use core_extensions::prelude::*;

#[allow(unused_imports)]
use crate::{
    arenas::{AllocMethods, Arenas},
    utils::PrintDurationOnDrop,
};


pub use self::{
    sabi_extern_fn_impl::sabi_extern_fn,
    mangle_library_getter::mangle_library_getter_attr,
};


#[doc(hidden)]
pub fn derive_stable_abi(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err( input, stable_abi::derive ).into()
}

#[doc(hidden)]
pub fn derive_stable_abi_from_str(s: &str) -> TokenStream2 {
    parse_str_or_compile_err( s, stable_abi::derive )
}


#[allow(non_snake_case)]
#[doc(hidden)]
pub fn impl_InterfaceType(input: TokenStream1) -> TokenStream1{
    parse_or_compile_err( input, impl_interfacetype::the_macro ).into()
}


#[doc(hidden)]
pub fn derive_sabi_trait(_attr: TokenStream1, item: TokenStream1) -> TokenStream1{
    parse_or_compile_err( item, sabi_trait::derive_sabi_trait ).into()
}

#[doc(hidden)]
pub fn derive_sabi_trait_str(item: &str) -> TokenStream2{
    parse_str_or_compile_err( item, sabi_trait::derive_sabi_trait )
}


#[doc(hidden)]
pub fn derive_get_static_equivalent(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err( input, get_static_equivalent::derive ).into()
}


////////////////////////////////////////////////////////////////////////////////


fn parse_or_compile_err<P,F>(input:TokenStream1,f:F)->TokenStream2
where 
    P:syn::parse::Parse,
    F:FnOnce(P)->Result<TokenStream2,syn::Error>
{
    syn::parse::<P>(input)
        .and_then(f)
        .unwrap_or_else(|e| e.to_compile_error() )
}

fn parse_str_or_compile_err<P,F>(input:&str,f:F)->TokenStream2
where 
    P:syn::parse::Parse,
    F:FnOnce(P)->Result<TokenStream2,syn::Error>
{
    syn::parse_str::<P>(input)
        .and_then(f)
        .unwrap_or_else(|e| e.to_compile_error() )
}