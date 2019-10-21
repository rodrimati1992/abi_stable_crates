use quote::ToTokens;
use proc_macro2::TokenStream as TokenStream2;

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub struct NoTokens;

impl ToTokens for NoTokens {
    fn to_tokens(&self, _: &mut TokenStream2) {}
}