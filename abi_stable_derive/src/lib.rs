extern crate proc_macro;

extern crate abi_stable_derive_lib;

use proc_macro::TokenStream as TokenStream1;

#[proc_macro_derive(StableAbi, attributes(sabi))]
pub fn derive_stable_abi(input: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::derive_stable_abi(input)
}

#[proc_macro_attribute]
pub fn mangle_library_getter(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::mangle_library_getter_attr(attr,item)
}