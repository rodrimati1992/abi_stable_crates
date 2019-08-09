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
mod composite_collections;
mod common_tokens;
mod constants;
mod get_static_equivalent;
mod ignored_wrapper;
mod datastructure;
mod fn_pointer_extractor;
mod impl_interfacetype;
mod parse_utils;
mod mangle_library_getter;
mod my_visibility;
mod gen_params_in;
mod workaround;
mod sabi_extern_fn_impl;
mod set_span_visitor;




mod lifetimes;

#[doc(hidden)]
pub mod stable_abi;

mod to_token_fn;

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
    let input = syn::parse::<DeriveInput>(input).unwrap();
    measure!({
        stable_abi::derive(input)
    }).into()
}

#[doc(hidden)]
pub fn derive_stable_abi_from_str(s: &str) -> TokenStream2 {
    let input = syn::parse_str::<DeriveInput>(s).unwrap();
    measure!({
        stable_abi::derive(input)
    })
}


#[allow(non_snake_case)]
#[doc(hidden)]
pub fn impl_InterfaceType(input: TokenStream1) -> TokenStream1{
    let input = syn::parse::<syn::ItemImpl>(input).unwrap();
    impl_interfacetype::the_macro(input).into()
}



#[doc(hidden)]
pub mod sabi_trait;

#[doc(hidden)]
pub fn derive_sabi_trait(_attr: TokenStream1, item: TokenStream1) -> TokenStream1{
    let item = syn::parse::<syn::ItemTrait>(item).unwrap();
    sabi_trait::derive_sabi_trait(item).into()
}

#[doc(hidden)]
pub fn derive_sabi_trait_str(item: &str) -> TokenStream2{
    let item = syn::parse_str::<syn::ItemTrait>(item).unwrap();
    sabi_trait::derive_sabi_trait(item)
}


#[doc(hidden)]
pub fn derive_get_static_equivalent(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse::<DeriveInput>(input).unwrap();
    measure!({
        get_static_equivalent::derive(input)
    }).into()
}


