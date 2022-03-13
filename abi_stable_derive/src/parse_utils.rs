//! Functions for parsing many `syn` types.

use as_derive_utils::ret_err_on_peek;

use syn::{parse, punctuated::Punctuated, token::Add, TypeParamBound};

use proc_macro2::Span;

//use crate::utils::SynResultExt;

pub(crate) fn parse_str_as_ident(lit: &str) -> syn::Ident {
    syn::Ident::new(lit, Span::call_site())
}

pub(crate) fn parse_str_as_path(lit: &str) -> Result<syn::Path, syn::Error> {
    syn::parse_str(lit)
}

pub(crate) fn parse_str_as_trait_bound(lit: &str) -> Result<syn::TraitBound, syn::Error> {
    syn::parse_str(lit)
}

pub(crate) fn parse_str_as_type(lit: &str) -> Result<syn::Type, syn::Error> {
    syn::parse_str(lit)
}

#[allow(dead_code)]
pub(crate) fn parse_lit_as_type_bound(lit: &syn::LitStr) -> Result<TypeParamBound, syn::Error> {
    lit.parse()
}

pub struct ParseBounds {
    pub list: Punctuated<TypeParamBound, Add>,
}

impl parse::Parse for ParseBounds {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        ret_err_on_peek! {input, syn::Lit, "bound", "literal"}

        let list = Punctuated::<TypeParamBound, Add>::parse_terminated(input).map_err(|e| {
            let msg = format!("while parsing bounds: {}", e);
            syn::Error::new(e.span(), msg)
        })?;

        if list.is_empty() {
            Err(input.error("type bounds can't be empty"))
        } else {
            Ok(Self { list })
        }
    }
}

pub struct ParsePunctuated<T, P> {
    pub list: Punctuated<T, P>,
}

impl<T, P> parse::Parse for ParsePunctuated<T, P>
where
    T: parse::Parse,
    P: parse::Parse,
{
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        Ok(Self {
            list: Punctuated::parse_terminated(input)?,
        })
    }
}
