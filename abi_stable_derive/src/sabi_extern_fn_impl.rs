//! Implementation details of the `#[sabi_extern_fn]` attribute.

use std::mem;

use as_derive_utils::return_spanned_err;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};

use quote::{quote, ToTokens};

use syn::{Expr, ItemFn};

use crate::parse_or_compile_err;

#[doc(hidden)]
pub fn sabi_extern_fn(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    parse_or_compile_err(item, move |item| sabi_extern_fn_inner(attr.into(), item)).into()
}

#[cfg(test)]
pub(crate) fn sabi_extern_fn_str(attr: &str, item: &str) -> Result<TokenStream2, syn::Error> {
    syn::parse_str(item).and_then(move |item| {
        let attr = syn::parse_str::<TokenStream2>(attr)?;
        sabi_extern_fn_inner(attr, item)
    })
}

/// Whether the function contains an early return or not.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WithEarlyReturn {
    No,
    Yes,
}

/// Converts a function into an `extern "C" fn` which aborts on panic.
pub(crate) fn convert_to_sabi_extern_fn(with_early_return: WithEarlyReturn, item: &mut ItemFn) {
    let no_early_return = match with_early_return {
        WithEarlyReturn::No => Some(quote!( no_early_return; )),
        WithEarlyReturn::Yes => None,
    };

    item.sig.abi = Some(syn::Abi {
        extern_token: Default::default(),
        name: Some(syn::LitStr::new("C", Span::call_site())),
    });

    let statements = mem::take(&mut item.block.stmts);

    let x = quote! {
        ::abi_stable::extern_fn_panic_handling!(
            #no_early_return

            #(#statements)*
        )
    };
    let x = Expr::Verbatim(x);
    let x = syn::Stmt::Expr(x);
    let x = vec![x];
    item.block.stmts = x;
}

fn sabi_extern_fn_inner(attr: TokenStream2, mut item: ItemFn) -> Result<TokenStream2, syn::Error> {
    let with_early_return = match attr.into_iter().next() {
        Some(TokenTree::Ident(ref ident)) if ident == "no_early_return" => WithEarlyReturn::No,
        Some(tt) => return_spanned_err!(tt, "Unrecognized `#[sabi_extern_fn]` parameter",),
        None => WithEarlyReturn::Yes,
    };

    convert_to_sabi_extern_fn(with_early_return, &mut item);

    Ok(item.into_token_stream())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output() {
        let list = vec![
            (
                "",
                r##"
                    pub fn hello()->RString{
                        println!("{}",HELLO);
                        println!(",");
                        println!("{}",WORLD);
                    }
                "##,
                quote!(
                    pub extern "C" fn hello() -> RString {
                        ::abi_stable::extern_fn_panic_handling!(
                            println!("{}",HELLO);
                            println!(",");
                            println!("{}",WORLD);
                        )
                    }
                ),
            ),
            (
                "no_early_return",
                r##"
                    pub(crate) extern "Rust" fn hello()->RStr<'_>{
                        println!("{}",HELLO);
                        println!(",");
                        println!("{}",WORLD);
                        "stuff".into()
                    }
                "##,
                quote!(
                    pub(crate) extern "C" fn hello() -> RStr<'_> {
                        ::abi_stable::extern_fn_panic_handling!(
                            no_early_return;
                            println!("{}",HELLO);
                            println!(",");
                            println!("{}",WORLD);
                            "stuff".into()
                        )
                    }
                ),
            ),
        ];

        for (attr, item, expected) in list {
            assert_eq!(
                sabi_extern_fn_str(attr, item).unwrap().to_string(),
                expected.to_string()
            );
        }
    }
}
