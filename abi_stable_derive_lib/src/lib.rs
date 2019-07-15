/*!
An implementation detail of abi_stable.
*/

#![recursion_limit="192"]
//#![deny(unused_variables)]
#![deny(unreachable_patterns)]
#![deny(unused_doc_comments)]
#![deny(unconditional_recursion)]

extern crate core_extensions;

extern crate proc_macro;

#[macro_use]
mod macros;

#[macro_use]
mod utils;

mod arenas;
mod attribute_parsing;
mod common_tokens;
mod composite_collections;
mod constants;
mod either;
mod ignored_wrapper;
mod datastructure;
mod fn_pointer_extractor;
mod impl_interfacetype;
mod parse_utils;
mod my_visibility;
mod gen_params_in;
mod workaround;
mod sabi_extern_fn_impl;




mod lifetimes;

#[doc(hidden)]
pub mod stable_abi;

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
    utils::PrintDurationOnDrop,
};


pub use self::sabi_extern_fn_impl::sabi_extern_fn;



#[doc(hidden)]
pub fn derive_stable_abi(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse::<DeriveInput>(input).unwrap();
    measure!({
        stable_abi::derive(input)
    }).into()
}

#[doc(hidden)]
pub fn derive_stable_abi_from_str(s: &str) -> TokenStream2 {
    let input = syn::parse_str::<DeriveInput>(s).unwrap();
    measure!({
        stable_abi::derive(input)
    })
}


#[allow(non_snake_case)]
#[doc(hidden)]
pub fn impl_InterfaceType(input: TokenStream1) -> TokenStream1{
    let input = syn::parse::<syn::ItemImpl>(input).unwrap();
    impl_interfacetype::the_macro(input).into()
}



#[doc(hidden)]
pub mod sabi_trait;

#[doc(hidden)]
pub fn derive_sabi_trait(_attr: TokenStream1, item: TokenStream1) -> TokenStream1{
    let item = syn::parse::<syn::ItemTrait>(item).unwrap();
    sabi_trait::derive_sabi_trait(item).into()
}

#[doc(hidden)]
pub fn derive_sabi_trait_str(item: &str) -> TokenStream2{
    let item = syn::parse_str::<syn::ItemTrait>(item).unwrap();
    sabi_trait::derive_sabi_trait(item)
}


#[doc(hidden)]
pub fn mangle_library_getter_attr(_attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    use syn::Ident;

    use proc_macro2::Span;

    use abi_stable_shared::mangled_root_module_loader_name;

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
                    ::abi_stable::extern_fn_panic_handling!(
                        let ret:#ret_ty=#original_fn_ident();
                        let _=abi_stable::library::RootModule::load_module_with(||{
                            Ok::<_,()>(ret)
                        });
                        unsafe{
                            abi_stable::utils::transmute_reference(ret)
                        }
                    )
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


