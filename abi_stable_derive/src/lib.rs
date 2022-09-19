//! An implementation detail of abi_stable.

#![recursion_limit = "192"]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![deny(unreachable_patterns)]
#![deny(unused_doc_comments)]
#![deny(unconditional_recursion)]

extern crate proc_macro;

#[proc_macro_derive(StableAbi, attributes(sabi))]
pub fn derive_stable_abi(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err(input, stable_abi::derive).into()
}

#[doc(hidden)]
#[proc_macro]
#[allow(non_snake_case)]
pub fn impl_InterfaceType(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err(input, impl_interfacetype::the_macro).into()
}

#[proc_macro_attribute]
pub fn export_root_module(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    crate::export_root_module_impl::export_root_module_attr(attr, item)
}

#[proc_macro_attribute]
pub fn sabi_extern_fn(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    crate::sabi_extern_fn_impl::sabi_extern_fn(attr, item)
}

#[proc_macro_attribute]
pub fn sabi_trait(_attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    parse_or_compile_err(item, sabi_trait::derive_sabi_trait).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn concatenated_and_ranges(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err(input, concat_and_ranges::macro_impl).into()
}

#[proc_macro_derive(GetStaticEquivalent, attributes(sabi))]
pub fn derive_get_static_equivalent(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err(input, get_static_equivalent::derive).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn construct_abi_header(_: TokenStream1) -> TokenStream1 {
    let abi_major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap();
    let abi_minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap();
    quote!(
        pub const ABI_HEADER:AbiHeader=AbiHeader{
            magic_string:*b"abi stable library for Rust     ",
            abi_major:#abi_major,
            abi_minor:#abi_minor,
            _priv:(),
        };
    )
    .into()
}

/// This is used by testing/version_compatibility to access the exported static.
#[doc(hidden)]
#[proc_macro]
pub fn get_root_module_static(_: TokenStream1) -> TokenStream1 {
    let export_name = syn::Ident::new(
        &abi_stable_shared::mangled_root_module_loader_name(),
        proc_macro2::Span::call_site(),
    );
    quote!( crate::#export_name ).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn __const_mangled_root_module_loader_name(_: TokenStream1) -> TokenStream1 {
    let name = abi_stable_shared::mangled_root_module_loader_name();
    let name_nulled = format!("{}\0", name);

    quote!(
        const PRIV_MANGLED_ROOT_MODULE_LOADER_NAME: &str = #name;
        const PRIV_MANGLED_ROOT_MODULE_LOADER_NAME_NUL: &str = #name_nulled;
    )
    .into()
}

///////////////////////////////////////////////////////////////////////////////

#[macro_use]
mod utils;

mod arenas;
mod attribute_parsing;
mod common_tokens;
mod composite_collections;
mod concat_and_ranges;
mod export_root_module_impl;
mod fn_pointer_extractor;
mod get_static_equivalent;
mod ignored_wrapper;
mod impl_interfacetype;
mod lifetimes;
mod literals_constructors;
mod my_visibility;
mod parse_utils;
mod sabi_extern_fn_impl;
mod set_span_visitor;
mod workaround;

#[cfg(test)]
mod input_code_range_tests;

#[doc(hidden)]
pub(crate) mod stable_abi;

#[doc(hidden)]
pub(crate) mod sabi_trait;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;

use syn::{DeriveInput, ItemFn};

use quote::{quote, quote_spanned, ToTokens};

#[allow(unused_imports)]
use core_extensions::SelfOps;

#[allow(unused_imports)]
use crate::{
    arenas::{AllocMethods, Arenas},
    utils::PrintDurationOnDrop,
};

#[cfg(test)]
pub(crate) fn derive_stable_abi_from_str(s: &str) -> Result<TokenStream2, syn::Error> {
    syn::parse_str(s).and_then(stable_abi::derive)
}

#[cfg(test)]
pub(crate) fn derive_sabi_trait_str(item: &str) -> Result<TokenStream2, syn::Error> {
    syn::parse_str(item).and_then(sabi_trait::derive_sabi_trait)
}

////////////////////////////////////////////////////////////////////////////////

fn parse_or_compile_err<P, F>(input: TokenStream1, f: F) -> TokenStream2
where
    P: syn::parse::Parse,
    F: FnOnce(P) -> Result<TokenStream2, syn::Error>,
{
    syn::parse::<P>(input)
        .and_then(f)
        .unwrap_or_else(|e| e.to_compile_error())
}
