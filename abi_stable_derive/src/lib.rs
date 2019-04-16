/*!
An implementation detail of abi_stable.
*/

extern crate proc_macro;

extern crate abi_stable_derive_lib;

use proc_macro::TokenStream as TokenStream1;


/**

The StableAbi derive macro allows one to implement the StableAbi trait to :

- Assert that the type has a stable representation across Rust version/compiles.

- Produce the layout of the type at runtime to check it against the loaded library.

# Container Attributes

These attributes are applied on the type declaration.

### `#[sabi(unconstrained(TypeParameter))]` 

Removes the implicit `TypeParameter:StableAbi` constraint.

### `#[sabi(bound="Type:ATrait")]`

Adds a bound to the StableAbi impl.

### `#[sabi(debug_print)]`

Prints the generated code,stopping compilation.

# Field attributes

These attributes are applied to fields.

### `#[sabi(unsafe_opaque_field)]`

Does not require the field to implement StableAbi,
and instead uses the StableAbi impl of `UnsafeOpaqueField<FieldType>`.

This is unsafe because the layout of the type won't be verified when loading the library,
which causes Undefined Behavior if the type has a different layout.

# Supported repr attributes

Because repr attributes can cause the type to change layout,
the StableAbi derive macro has to know about every repr attribute applied to the type,
since it might invalidate layout stability.

### `repr(C)`

This is the representation that most StableAbi types will have.

### `repr(transparent)`

`repr(transparent)` types inherit the abi stability of their first field.

### `repr(align(...))`

`repr(align(...))` is supported,
so long as it is used in combination with the other supported repr attributes.

*/

#[proc_macro_derive(StableAbi, attributes(sabi))]
pub fn derive_stable_abi(input: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::derive_stable_abi(input)
}



/*
Deprecated:use `export_sabi_module` instead of this to export abi_stable modules.
*/
#[deprecated]
#[proc_macro_attribute]
pub fn mangle_library_getter(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::mangle_library_getter_attr(attr,item)
}

/**

This attribute is used for functions which export a module in an `implementation crate`.

When applied it creates a mangled function which calls the annotated function,
as well as check its type signature.

This is applied to functions like this:

```ignore

#[export_sabi_module]
pub extern "C" fn get_hello_world_mod() -> WithLayout<TextOperationsMod> {
    extern_fn_panic_handling!{
        let module=TextOperationsMod{
            reverse_string,
        };
        WithLayout::new(module)
    }
}

# fn main(){}

```

# More examples

For a more detailed example look into the abi_stable_example*_impl crates in the 
repository for this crate.



*/
#[proc_macro_attribute]
pub fn export_sabi_module(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    abi_stable_derive_lib::mangle_library_getter_attr(attr,item)
}

