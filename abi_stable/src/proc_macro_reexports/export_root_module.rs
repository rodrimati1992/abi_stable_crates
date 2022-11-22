/**
This attribute is used for functions which export a module in an `implementation crate`.

This is applied to functions like this:

```rust

use abi_stable::prefix_type::PrefixTypeTrait;

#[abi_stable::export_root_module]
pub fn get_hello_world_mod() -> TextOperationsMod_Ref {
    TextOperationsMod { reverse_string }.leak_into_prefix()
}

# #[repr(C)]
# #[derive(abi_stable::StableAbi)]
# #[sabi(kind(Prefix(prefix_ref= TextOperationsMod_Ref)))]
# #[sabi(missing_field(panic))]
# pub struct TextOperationsMod {
#     #[sabi(last_prefix_field)]
#     pub reverse_string: extern "C" fn(),
# }
# 
# extern "C" fn reverse_string() {}

# impl abi_stable::library::RootModule for TextOperationsMod_Ref {
#     abi_stable::declare_root_module_statics!{TextOperationsMod_Ref}
#     const BASE_NAME: &'static str = "stuff";
#     const NAME: &'static str = "stuff";
#     const VERSION_STRINGS: abi_stable::sabi_types::VersionStrings =
#           abi_stable::package_version_strings!();
# }

# fn main(){}


```

# Return Type

The return type of the annotated function can be one of:

- Any type that implements `abi_stable::library::RootModule`

- `Result<M, RBoxError>`, where `M` is any type that implements 
`abi_stable::library::RootModule`

- `RResult<M, RBoxError>`, where `M` is any type that implements 
`abi_stable::library::RootModule`

All those types are supported through the [`IntoRootModuleResult`] trait,
which you can implement if you want to return some other type.

# Generated code

Exporting the root module creates a 
`static THE_NAME_USED_FOR_ALL_ROOT_MODULES: `[`LibHeader`]` = ... ;` 
with these things:

- The version of `abi_stable` used.

- A `#[no_mangle]` function that wraps the annotated root-module constructor function,
converting the return value to [`RootModuleResult`](./library/type.RootModuleResult.html).

- The type layout of the root module,
for checking that the types are compatible with whatever loads that library.

- The version number of the library.

- A [`LateStaticRef`] of the root module.


The name used for generated static is the value of 
[`abi_stable::library::ROOT_MODULE_LOADER_NAME`](./library/constant.ROOT_MODULE_LOADER_NAME.html).

# Remove type layout constant

One can avoid generating the type layout constant for the exported root module by using the
`#[unsafe_no_layout_constant]` attribute,
with the downside that if the layout changes(in an incompatible way)
it could be Undefined Behavior.

This attribute is useful if one wants to minimize the size of the dynamic library when 
doing a public release.

This attribute should not be used unconditionally,
it should be disabled in Continuous Integration so that the 
binary compatibility of a dynamic library is checked at some point before releasing it.

# More examples

For a more detailed example look in the README in the repository for this crate.



[`IntoRootModuleResult`]: ./library/trait.IntoRootModuleResult.html
[`LateStaticRef`]: ./sabi_types/struct.LateStaticRef.html
[`LibHeader`]: ./library/struct.LibHeader.html

*/
#[doc(inline)]
pub use abi_stable_derive::export_root_module;