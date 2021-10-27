//! Functions for parsing many `syn` types.

use as_derive_utils::spanned_err;

use syn::{
    parse,
    punctuated::Punctuated,
    token::Add,
    LitStr,
    //Ident,
    TypeParamBound,
};

use proc_macro2::Span;

//use crate::utils::SynResultExt;

pub(crate) fn parse_str_as_ident(lit: &str) -> syn::Ident {
    syn::Ident::new(&lit, Span::call_site())
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

pub(crate) fn parse_lit_as_expr(lit: &syn::LitStr) -> Result<syn::Expr, syn::Error> {
    lit.parse()
}

pub(crate) fn parse_lit_as_type(lit: &syn::LitStr) -> Result<syn::Type, syn::Error> {
    lit.parse()
}

#[allow(dead_code)]
pub(crate) fn parse_lit_as_type_bound(lit: &syn::LitStr) -> Result<TypeParamBound, syn::Error> {
    lit.parse()
}

#[allow(dead_code)]
pub(crate) fn parse_lit_as_type_bounds(
    str_: &LitStr,
) -> Result<Punctuated<TypeParamBound, Add>, syn::Error> {
    str_.parse::<ParseBounds>().and_then(|x| {
        if x.list.is_empty() {
            Err(spanned_err!(str_, "type bounds can't be empty"))
        } else {
            Ok(x.list)
        }
    })
}

pub struct ParseBounds {
    list: Punctuated<TypeParamBound, Add>,
}

impl parse::Parse for ParseBounds {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        Ok(Self {
            list: Punctuated::<TypeParamBound, Add>::parse_terminated(input)?,
        })
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
