use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Error, Ident, Token,
};

use proc_macro2::TokenStream as TokenStream2;

use quote::quote;

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
struct StringAndVariable {
    variable: Ident,
    string: String,
}

#[derive(Debug, Clone)]
struct ConcatenatedStrings {
    concatenated: Ident,
    strings: Punctuated<StringAndVariable, Token![,]>,
}

impl Parse for StringAndVariable {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let variable = input.parse()?;
        let _: Token![=] = input.parse()?;
        let string = input.parse::<syn::LitStr>()?.value();

        Ok(Self { variable, string })
    }
}

impl Parse for ConcatenatedStrings {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let concatenated = input.parse::<Ident>()?;

        let paren_tokens;
        let _ = syn::parenthesized!(paren_tokens in input);

        let strings = paren_tokens.parse_terminated(Parse::parse)?;

        Ok(Self {
            concatenated,
            strings,
        })
    }
}

pub fn macro_impl(input: TokenStream2) -> Result<TokenStream2, syn::Error> {
    let ConcatenatedStrings {
        concatenated: conc_ident,
        strings,
    } = syn::parse2::<ConcatenatedStrings>(input)?;

    let capacity = strings.iter().map(|sav| sav.string.len() + 1).sum();

    let mut concatenated = String::with_capacity(capacity);
    let mut starts = Vec::<u16>::with_capacity(strings.len());
    let mut lengths = Vec::<u16>::with_capacity(strings.len());

    for sav in &strings {
        let start = concatenated.len() as u16;
        starts.push(start);
        concatenated.push_str(&sav.string);
        lengths.push(concatenated.len() as u16 - start);
        concatenated.push(';');
    }

    let concat_len = concatenated.len();
    let variable = strings.iter().map(|x| &x.variable);

    Ok(quote!(
        use abi_stable::{
            std_types::RStr,
            type_layout::StartLen,
        };
        pub const #conc_ident:RStr<'static>=unsafe{
            RStr::from_raw_parts( #concatenated.as_ptr() ,#concat_len )
        };

        #(
            pub const #variable:StartLen= StartLen::new(#starts,#lengths);
        )*
    ))
}
