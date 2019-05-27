/*!
An implementation detail of abi_stable.
*/

#![recursion_limit="192"]
//#![deny(unused_variables)]


extern crate core_extensions;

extern crate proc_macro;

#[macro_use]
mod macros;

mod arenas;
mod attribute_parsing;
mod common_tokens;
mod constants;
mod ignored_wrapper;
mod datastructure;
mod fn_pointer_extractor;
mod impl_interfacetype;


mod lifetimes;

#[doc(hidden)]
pub mod stable_abi;

#[doc(hidden)]
pub mod sabi_trait;

mod to_token_fn;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;

use syn::{DeriveInput,ItemFn};

use quote::{quote, ToTokens};

#[allow(unused_imports)]
use core_extensions::prelude::*;

#[allow(unused_imports)]
use crate::{
    arenas::{AllocMethods, Arenas},
    common_tokens::CommonTokens,
};


fn mangle_ident<S>(kind:&str,name:S)->String
where S: ::std::fmt::Display
{

    let unmangled=format!("_as.{}.{}",kind,name);

    let mut mangled=String::with_capacity(unmangled.len()*3/2);

    for kv in unmangled.split_while(|c| c.is_alphanumeric() ) {
        if kv.key {
            mangled.push_str(kv.str);
            continue
        }
        for c in kv.str.chars() {
            mangled.push_str(match c {
                '.'=>"_0",
                '_'=>"_1",
                '-'=>"_2",
                '<'=>"_3",
                '>'=>"_4",
                '('=>"_5",
                ')'=>"_6",
                '['=>"_7",
                ']'=>"_8",
                '{'=>"_9",
                '}'=>"_a",
                ' '=>"_b",
                ','=>"_c",
                ':'=>"_d",
                ';'=>"_e",
                '!'=>"_f",
                '#'=>"_g",
                '$'=>"_h",
                '%'=>"_i",
                '/'=>"_j",
                '='=>"_k",
                '?'=>"_l",
                '¿'=>"_m",
                '¡'=>"_o",
                '*'=>"_p",
                '+'=>"_q",
                '~'=>"_r",
                '|'=>"_s",
                '°'=>"_t",
                '¬'=>"_u",
                '\''=>"_x",
                '\"'=>"_y",
                '`'=>"_z",
                c=>panic!("cannot currently mangle the '{}' character.", c),
            });
        }
    }

    mangled
}


#[doc(hidden)]
pub fn derive_stable_abi(input: TokenStream1) -> TokenStream1 {
    measure!({
        let input = syn::parse::<DeriveInput>(input).unwrap();
        // println!("deriving StableAbi for {}",input.ident);
        stable_abi::derive(input).into()
    })
}

#[doc(hidden)]
pub fn derive_stable_abi_from_str(s: &str) -> TokenStream2 {
    let input = syn::parse_str::<DeriveInput>(s).unwrap();
    stable_abi::derive(input)
}


#[allow(non_snake_case)]
#[doc(hidden)]
pub fn impl_InterfaceType(input: TokenStream1) -> TokenStream1{
    let input = syn::parse::<syn::ItemImpl>(input).unwrap();
    impl_interfacetype::the_macro(input).into()
}


/// Gets the name of the function that loads the root module of a library.
pub fn mangled_root_module_loader_name()->String{
    mangle_ident("lib_header","root module loader")
}


#[doc(hidden)]
pub fn mangle_library_getter_attr(_attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    use syn::Ident;

    use proc_macro2::Span;
    

    measure!({
        let input = syn::parse::<ItemFn>(item).unwrap();
        
        let vis=&input.vis;
        let attrs=&input.attrs;
        let ret_ty=match &input.decl.output {
            syn::ReturnType::Default=>
                panic!("\n\nThe return type of this function can't be `()`\n\n"),
            syn::ReturnType::Type(_,ty)=>
                &**ty,
        };
        
        let original_fn_ident=&input.ident;

        let export_name=Ident::new(
            &mangled_root_module_loader_name(),
            Span::call_site(),
        );

        quote!(
            #input

            #[no_mangle]
            #(#attrs)*
            #vis static #export_name:abi_stable::library::LibHeader={
                use abi_stable::{
                    library::{LibHeader as __LibHeader},
                    StableAbi,
                };

                pub extern "C" fn _sabi_erased_module(
                )->&'static abi_stable::marker_type::ErasedObject {
                    let ret:#ret_ty=#original_fn_ident();
                    unsafe{
                        abi_stable::utils::transmute_reference(ret)
                    }
                }

                type __ReturnTy=#ret_ty;
                type __ModuleTy=<__ReturnTy as std::ops::Deref>::Target;
                
                unsafe{
                    __LibHeader::from_constructor::<__ModuleTy>(
                        abi_stable::utils::Constructor(_sabi_erased_module),
                        <__ModuleTy as abi_stable::library::RootModule>::CONSTANTS,
                    )
                }
            };
        ).into()
    })
}

