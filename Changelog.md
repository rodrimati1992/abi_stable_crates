This is the changelog,summarising changes in each version.

# 0.4

- Added basic module reflection,changing a few details of how layout is represented.

- Created the sabi_extract tool that converts the module structure of an 
    abi_stable dynamic library to json.

- Streamlined how modules are exported,
    removing the need to specify the LOADER_FN in the RootModule trait,
    as well as constructing the module using the `RootModule::load_*` functions.

- Changed how the root module is accessed after it's loaded,
    using the `<RootMod as RootModule>::get_module` function.

- Added fn root_module_statics to declare the statics associated with a RootModule,
    as well as the `declare_root_module_statics` macro to implement it.

- Changed `RootModule::raw_library_ref` to `RootModule::get_raw_library` ,
    returning the already loaded RawLibrary instead of allowing the user 
    to initialize it themselves.

- Changed how all libraries are loaded so that the abi_stable version they 
    use can be checked,mentioning the abi_stable version in the returned error.

- Renamed `Library` to `RawLibrary`.

- Now the RawLibrary is unloaded after layout checking fails,
    leaking it if layout checking passes instead of doing so when it's loaded.

- Added `#[sabi(refl(pub_getter=" function_name "))]` 
    attribute for code generation (using the layout constant for a type),
    to determine how to access private fields(otherwise they're inaccesible).

- Renamed  `export_sabi_module` to `export_root_module`.

- Added RMutex/RRwLock/ROnce,wrapping parking_lot types.

- Added ffi-safe wrappers for crossbeam channels.

- Added support for #[repr(<IntegerType>)],for enums.

- Added checking of enum discriminants(supports all integer types up to u64/i64).

- Renamed LazyStaticRef to LateStaticRef,made if ffi-safe.





# 0.3 

- Dropped support for 1.33 (no requiring 1.34) due to 
    an ICE caused by associated types in associated constants.

- Renamed VirtualWrapper to DynTrait,moving `I:InterfaceType` to second type parameter.

- Added tags,a dynamically typed data structure used 
    when checking the layout of types at runtime.

- DynTrait can now be constructed from non-`'static` types,
    using `DynTrait::from_borrowìng_*`.

- Added conditional accessors to prefix-types,
    allowing those fields to have any type if disabled 
    (so long as they don't change in size/alignment)

- Added these conditional traits to DynTrait:
    - Send
    - Sync.
    - Iterator
    - DoubleEndedIterator
    - std::fmt::Write
    - std::io::{Write,Seek,Read,BufRead}

- Improved documentation of DynTrait,including multiple examples,
    and how to make a pointer compatible with it.

- Improved the std_types error types,
    to be almost the same as the ones in the standard library.

- Added reborrowing to DynTrait,
    going from DynTrait<'a,P<()>,I> to DynTrait<'a,&(),I>/DynTrait<'a,&mut (),I> .

- Added impl_InterfaceType macro to implement InterfaceType,
    emulating default associated types.

- Changed RCow to be closer to its standard library equivalent.

- Added methods/documentation to ROption/RResult.

- Added RHashMap,with an API very close to the standard HashMap.

# 0.2

- Added SharedStableAbi trait to implement prefix-types (vtables and modules).

- Added a "StaticEquivalent:'static" associated type to StableAbi/SharedStableAbi 
    to construct a type-id from any type,for checking their layout only once

- Added impl_InterfaceType macro for 
    implementing InterfaceType with default associated types.

- Tightened safety around phantom type parameters,
    requiring every type to appear in the layout constant of the type.

- Implemented prefix-types,for extensible vtables/modules,
    along with rewriting existing VTables and modules to use them.

- Implemented private multi-key map for layout checking
    (it's not public purely for documentation reasons).

- Moved example crates to example folder

- Replaced LibraryTrait/ModuleTrait with RootModule trait,
    only allowing the root module to be exported.

- Improved example 0,adding a readme,
    and an example of serializing/deserializing json commands.

- Added documentation for interactions the library has with unsafe code/
    how to write unsafe code that uses the library.

- Many small changes to documentation.
