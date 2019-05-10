This is the changelog,summarising changes in each version.

# 0.3 

- Dropped support for 1.33 (no requiring 1.34) due to 
    an ICE caused by associated types in associated constants.

- Renamed VirtualWrapper to DynTrait.

- Added tags,a dynamically typed data structure used when checking the layout of types at runtime.

- DynTrait can now be constructed from non-`'static` types.

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

- Improved the std_types error types,to be almost the same as the ones in the standard library.

- Added reborrowing to DynTrait,
    going from DynTrait<'a,P<()>,I> to DynTrait<'a,&(),I>/DynTrait<'a,&mut (),I> .

- Added impl_InterfaceType macro to emulate default associated types 
    while implementing InterfaceType.

- Changed RCow to be closer to its standard library equivalent.

- Added methods/documentation to ROption/RResult

- Added RHashMap,with an API very close to the standard HashMap.

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
