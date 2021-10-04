/**
This attribute is used for functions which export a module in an `implementation crate`.

When applied it creates a mangled function which calls the annotated function,
as well as check its type signature.

This is applied to functions like this:

```rust

use abi_stable::prefix_type::PrefixTypeTrait;

#[abi_stable::export_root_module]
pub fn get_hello_world_mod() -> TextOperationsMod_Ref {
    TextOperationsMod{
        reverse_string,
    }.leak_into_prefix()
}


# #[repr(C)]
# #[derive(abi_stable::StableAbi)]
# #[sabi(kind(Prefix(prefix_ref="TextOperationsMod_Ref")))]
# #[sabi(missing_field(panic))]
# pub struct TextOperationsMod {
#     #[sabi(last_prefix_field)]
#     pub reverse_string: extern "C" fn(),
# }
# 
# extern "C" fn reverse_string() {}
#
# impl abi_stable::library::RootModule for TextOperationsMod_Ref {
#     abi_stable::declare_root_module_statics!{TextOperationsMod_Ref}
#     const BASE_NAME: &'static str = "stuff";
#     const NAME: &'static str = "stuff";
#     const VERSION_STRINGS: abi_stable::sabi_types::VersionStrings =
#           abi_stable::package_version_strings!();
# }
#
# fn main(){}

```

# Return Type

The return type of the annotated function can be one of:

- Any type that implements `abi_stable::library::RootModule`

- `Result<M, RBoxError>`, where `M` is any type that implements 
`abi_stable::library::RootModule`

- `RResult<M, RBoxError>`, where `M` is any type that implements 
`abi_stable::library::RootModule`

All those types are supported through the `abi_stable::library::IntoRootModuleResult` trait,
which you can implement if you want to return some other type.

# Generated code

Exporting the root module creates a 
`static THE_NAME_USED_FOR_ALL_ROOT_MODULES:LibHeader= ... ;` 
with these things:

- The abi_stable version number used by the dynamic library.

- A constant describing the layout of the exported root module,and every type it references.

- A lazily initialized reference to the root module.

- The constructor function of the root module.

The name used for root modules is the one returned by 
`abi_stable::library::ROOT_MODULE_LOADER_NAME`.
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
#[doc(inline)]
pub use abi_stable_derive::export_root_module;