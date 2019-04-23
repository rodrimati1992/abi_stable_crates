This is the changelog,summarising changes in each version.

# 0.2

- Added SharedStableAbi trait to implement prefix-types (vtables and modules).

- Added a "StaticEquivalent:'static" associated type to StableAbi/SharedStableAbi 
    to construct a type-id from any type,for checking their layout only once

- Added impl_InterfaceType macro for implementing InterfaceType with default associated types.

- Renamed OpaqueType to ZeroSized.

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
