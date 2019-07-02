/*!
An implementation detail of abi_stable.
*/

extern crate proc_macro;

extern crate abi_stable_derive_lib;

use proc_macro::TokenStream as TokenStream1;


/**


This macro is documented in abi_stable::docs::stable_abi_derive

*/

#[proc_macro_derive(StableAbi, attributes(sabi))]
pub fn derive_stable_abi(input: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::derive_stable_abi(input)
}




/**

Allows implementing the InterfaceType trait,
providing default values for associated types not specified in the impl block.

For an example look at `abi_stable::erased_types::InterfaceType`.

*/
#[proc_macro]
#[allow(non_snake_case)]
pub fn impl_InterfaceType(input: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::impl_InterfaceType(input)
}





/**

This attribute is used for functions which export a module in an `implementation crate`.

When applied it creates a mangled function which calls the annotated function,
as well as check its type signature.

This is applied to functions like this:

```ignore

use abi_stable::prefix_type::PrefixTypeTrait;

#[export_root_module]
pub fn get_hello_world_mod() -> &'static TextOperationsMod {
    TextOperationsModVal{
        reverse_string,
    }.leak_into_prefix()
}

# fn main(){}

```

# More examples

For a more detailed example look in the README in the repository for this crate.



*/
#[proc_macro_attribute]
pub fn export_root_module(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::mangle_library_getter_attr(attr,item)
}

/**
This macro is documented in abi_stable::docs::sabi_extern_fn
*/
#[proc_macro_attribute]
pub fn sabi_extern_fn(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::sabi_extern_fn(attr,item)
}



#[proc_macro_attribute]
pub fn sabi_trait(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::derive_sabi_trait(attr,item)
}

