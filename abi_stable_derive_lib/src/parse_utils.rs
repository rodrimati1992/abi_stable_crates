use syn::{
    parse,
    punctuated::Punctuated,
    token::Add,
    TypeParamBound,
    LitStr,
};



pub(crate) fn parse_str_as_ident(lit:&str)->syn::Ident{
    match syn::parse_str::<syn::Ident>(lit) {
        Ok(ident)=>ident,
        Err(e)=>panic!(
            "Could not parse as an identifier:\n\t{}\nError:\n\t{}", 
            lit,
            e
        )
    }
}

pub(crate) fn parse_str_as_path(lit:&str)->syn::Path{
    match syn::parse_str::<syn::Path>(lit) {
        Ok(ident)=>ident,
        Err(e)=>panic!(
            "Could not parse as a path:\n\t{}\nError:\n\t{}", 
            lit,
            e
        )
    }
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
    parse_str_lit_as(lit,"Could not parse as an identifier")
}

pub(crate) fn parse_lit_as_expr(lit:&syn::LitStr)->syn::Expr{
    parse_str_lit_as(lit,"Could not parse as an expression")
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
