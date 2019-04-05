#![recursion_limit="128"]
//#![deny(unused_variables)]

#[macro_use]
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
mod gen_param_in;
mod lifetimes;
mod stable_abi;
mod to_token_fn;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;

use syn::DeriveInput;

use quote::{quote, ToTokens};

use crate::{
    arenas::{AllocMethods, Arenas, ArenasRef},
    common_tokens::CommonTokens,
};


pub fn derive_stable_abi(input: TokenStream1) -> TokenStream1 {
    measure!({
        let input = syn::parse::<DeriveInput>(input).unwrap();
        // println!("deriving StableAbi for {}",input.ident);
        stable_abi::derive(input).into()
    })
}

pub fn derive_stable_abi_from_str(s: &str) -> TokenStream2 {
    let input = syn::parse_str::<DeriveInput>(s).unwrap();
    stable_abi::derive(input)
}


#[test]
fn basic(){

    use core_extensions::measure_time::measure;

    let mut outer_output=TokenStream2::new();

    for _ in 0..10{
        let (dur,output)=measure(||{
            derive_stable_abi_from_str(r##"
        #[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct BufferVTable<T> {
    destructor: extern "C" fn(&mut T),
    grow_capacity_to: extern "C" fn(&mut T, usize, Exactness),
    shrink_to_fit: extern "C" fn(&mut T),
}

            "##)
        });
        outer_output=output;
        println!("took {} to run derive macro",dur );
    }

    panic!("\n\n\n{}\n\n\n",outer_output );

    derive_stable_abi_from_str(r##"
        #[repr(transparent)]
        struct What{
            func:extern fn(&'_ str)->&'_ str,
        }
    "##);
}