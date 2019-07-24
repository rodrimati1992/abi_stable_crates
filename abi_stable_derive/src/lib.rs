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

# Generated code

Exporting the root module creates a 
`static THE_NAME_USED_FOR_ALL_ROOT_MODULES:LibHeader= ... ;` 
with these things:

- The abi_stable version number used by the dynamic library.

- A constant describing the layout of the exported root module,and every type it references.

- A lazily initialized reference to the root module.

- The constructor function of the root module.

The name used for root modules is the one returned by 
`abi_stable::library::mangled_root_module_loader_name`.
Because there can't be multiple root modules for a library,
that function returns a constant.


# Remove type layout constant

One can avoid generating the type layout constant for the exported root module by using the
`#[unsafe_no_layout_constant]` attribute,
with the downside that if the layout changes(in an incompatible way)
it could be Undefined Behavior.

This attribute is useful if one wants to minimize the size of the dynamic library when 
doing a public release.

It is strongly encouraged that this attribute is used conditionally,
disabling it in Continuous Integration so that the 
binary compatibility of a dynamic library is checked at some point before releasing it.


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

