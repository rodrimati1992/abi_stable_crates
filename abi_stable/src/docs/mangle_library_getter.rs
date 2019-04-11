/*!

This attribute is used for functions which export a module in an `implementation crate`.

When applied it creates a mangled function which calls the annotated function,
as well as check its type signature.

This is applied to functions like this:

```ignore

#[mangle_library_getter]
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

For a more detailed example look into the abi_stable_example* crates in the 
repository for this crate.



*/