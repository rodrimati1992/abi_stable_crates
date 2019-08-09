/*!
Implementation details of the `#[sabi_extern_fn]` attribute.
*/

use std::mem;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span,TokenStream as TokenStream2,TokenTree};

use quote::{quote, ToTokens};

use syn::{ItemFn,Expr,ExprVerbatim};



#[doc(hidden)]
pub fn sabi_extern_fn(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    sabi_extern_fn_inner(
        attr.into(),
        syn::parse::<ItemFn>(item).unwrap(),
    ).into()
}

#[cfg(test)]
pub(crate) fn sabi_extern_fn_str(attr: &str, item: &str) -> TokenStream2 {
    sabi_extern_fn_inner(
        syn::parse_str::<TokenStream2>(attr).unwrap(),
        syn::parse_str::<ItemFn>(item).unwrap(),
    )
}

/// Whether the function contains an early return or not.
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum WithEarlyReturn{
    No,
    Yes,
}

/// Converts a function into an `extern "C" fn` which aborts on panic.
pub(crate) fn convert_to_sabi_extern_fn(
    with_early_return:WithEarlyReturn,
    item:&mut ItemFn,
){
    let no_early_return=match with_early_return {
        WithEarlyReturn::No=>Some(quote!( no_early_return; )),
        WithEarlyReturn::Yes=>None,
    };
    
    item.abi=Some(syn::Abi{
        extern_token:Default::default(),
        name:Some(syn::LitStr::new("C",Span::call_site()))
    });

    let statements=mem::replace(&mut item.block.stmts,Vec::new());

    let x=quote!{
        ::abi_stable::extern_fn_panic_handling!(
            #no_early_return

            #(#statements)*
        )
    };
    let x=ExprVerbatim{tts:x};
    let x=Expr::Verbatim(x);
    let x=syn::Stmt::Expr(x);
    let x=vec![x];
    item.block.stmts=x;
}


fn sabi_extern_fn_inner(attr:TokenStream2,mut item:ItemFn)->TokenStream2{
    let with_early_return=match attr.into_iter().next() {
        Some(TokenTree::Ident(ref ident))if ident =="no_early_return"=>
            WithEarlyReturn::No,
        Some(tt)=>
            panic!(
                "Unrecognized `#[sabi_extern_fn]` parameter:\n\t{}",
                tt
            ),
        None=>
            WithEarlyReturn::Yes,
    };
    
    convert_to_sabi_extern_fn(with_early_return,&mut item);
    
    item.into_token_stream()
}



#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_output(){
        let list=vec![
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
                    pub extern "C" fn hello()->RString{
                        ::abi_stable::extern_fn_panic_handling!(
                            println!("{}",HELLO);
                            println!(",");
                            println!("{}",WORLD);
                        )
                    }
                )
            ),
            (
                "no_early_return",
                r##"
                    pub(crate) extern "Rust" fn hello()->RStr<'_>{
                        println!("{}",HELLO);
                        println!(",");
                        println!("{}",WORLD);
                        panic!()
                    }
                "##,
                quote!(
                    pub(crate) extern "C" fn hello()->RStr<'_>{
                        ::abi_stable::extern_fn_panic_handling!(
                            no_early_return;
                            println!("{}",HELLO);
                            println!(",");
                            println!("{}",WORLD);
                            panic!()
                        )
                    }
                )
            ),
        ];

        for (attr,item,expected) in list {
            assert_eq!(
                sabi_extern_fn_str(attr,item).to_string(),
                expected.to_string()
            );
        }
    }
}