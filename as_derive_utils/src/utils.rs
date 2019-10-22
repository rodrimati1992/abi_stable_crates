use std::fmt::Display;

use quote::ToTokens;
use proc_macro2::{
    TokenStream as TokenStream2,
    Span,
};
use syn::spanned::Spanned;


////////////////////////////////////////////////////////////////////////////////

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub struct NoTokens;

impl ToTokens for NoTokens {
    fn to_tokens(&self, _: &mut TokenStream2) {}
}

////////////////////////////////////////////////////////////////////////////////


pub fn spanned_err(tokens:&dyn ToTokens, display:&dyn Display)-> syn::Error {
    syn::Error::new_spanned(tokens,display)
}

#[allow(dead_code)]
pub fn syn_err(span:Span,display:&dyn Display)-> syn::Error {
    syn::Error::new(span,display)
}


////////////////////////////////////////////////////////////////////////////////


pub fn join_spans<I,T>(iter:I)->Span
where
    I:IntoIterator<Item=T>,
    T:Spanned,
{
    let call_site=Span::call_site();
    let mut iter=iter.into_iter();
    let first:Span=match iter.next() {
        Some(x)=>x.span(),
        None=>return call_site,
    };

    iter.fold(first,|l,r| l.join(r.span()).unwrap_or(call_site) )
}


////////////////////////////////////////////////////////////////////////////////

#[inline(never)]
pub fn dummy_ident()->syn::Ident{
    syn::Ident::new("DUMMY_IDENT",Span::call_site())
}

////////////////////////////////////////////////////////////////////////////////

pub fn type_from_ident(ident: syn::Ident) -> syn::Type {
    let path: syn::Path = ident.into();
    let path = syn::TypePath { qself: None, path };
    path.into()
}

pub fn expr_from_ident(ident:syn::Ident)->syn::Expr{
    let x=syn::Path::from(ident);
    let x=syn::ExprPath{
        attrs:Vec::new(),
        qself:None,
        path:x,
    };
    syn::Expr::Path(x)
}

/// Used to tokenize an integer without a type suffix.
pub fn expr_from_int(int:u64)->syn::Expr{
    let x=proc_macro2::Literal::u64_unsuffixed(int);
    let x=syn::LitInt::from(x);
    let x=syn::Lit::Int(x);
    let x=syn::ExprLit{attrs:Vec::new(),lit:x};
    let x=syn::Expr::Lit(x);
    x
}

/// Used to tokenize an integer without a type suffix.
/// This one should be cheaper than `expr_from_int`.
pub fn uint_lit(int:u64)->syn::LitInt{
    let x=proc_macro2::Literal::u64_unsuffixed(int);
    syn::LitInt::from(x)
}


