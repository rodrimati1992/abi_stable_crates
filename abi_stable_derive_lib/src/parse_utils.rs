use syn::{
    parse,
    punctuated::Punctuated,
    token::Add,
    TypeParamBound,
    LitStr,
};

use proc_macro2::Span;


fn parse_str_as<P>(s:&str,err_description:&str)->P
where P:syn::parse::Parse
{
    match syn::parse_str::<P>(s) {
        Ok(v)=>v,
        Err(e)=>panic!("{}:\n\t{}\nError:\n\t{}", err_description,s,e)
    }
}


pub(crate) fn parse_str_as_ident(lit:&str)->syn::Ident{
    syn::Ident::new(&lit,Span::call_site())
}

pub(crate) fn parse_str_as_path(lit:&str)->syn::Path{
    parse_str_as(lit,"Could not parse as a path")
}

pub(crate) fn parse_str_as_type(lit:&str)->syn::Type{
    parse_str_as(lit,"Could not parse as a type")
}



fn parse_str_lit_as<P>(lit:&syn::LitStr,err_description:&str)->P
where P:syn::parse::Parse
{
    match lit.parse::<P>() {
        Ok(x)=>x,
        Err(e)=>panic!("{}:\n\t{}\nError:\n\t{}", err_description,lit.value(),e)
    }
}

pub(crate) fn parse_lit_as_ident(lit:&syn::LitStr)->syn::Ident{
    syn::Ident::new(&lit.value(),Span::call_site())
}

pub(crate) fn parse_lit_as_expr(lit:&syn::LitStr)->syn::Expr{
    parse_str_lit_as(lit,"Could not parse as an expression")
}

pub(crate) fn parse_lit_as_type(lit:&syn::LitStr)->syn::Type{
    parse_str_lit_as(lit,"Could not parse as a type")
}

#[allow(dead_code)]
pub(crate) fn parse_lit_as_type_bound(lit:&syn::LitStr)->TypeParamBound{
    parse_str_lit_as(lit,"Could not parse as an a bound")
}

#[allow(dead_code)]
pub(crate) fn parse_lit_as_type_bounds(str_: &LitStr) -> Punctuated<TypeParamBound, Add> {
    parse_str_lit_as::<ParseBounds>(str_,"Could not parse as type bounds")
        .list
}


pub struct ParseBounds {
    list: Punctuated<TypeParamBound, Add>,
}

impl parse::Parse for ParseBounds {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        Ok(Self{
            list:Punctuated::<TypeParamBound, Add>::parse_terminated(input)?,
        })
    }
}
