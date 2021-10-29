/*!

This document describes all the things that are unsafe to do with abi_stable.

# De/allocation,and other global state,without going through a vtable.

It is unsafe to rely on allocators being the same across dynamic libraries,
because Rust allows changing the allocator when building a dynamic library/binary.

The way this library handles allocation is by creating a vtable (virtual dispatch table)
for each type that directly allocates/deallocates (ie:`RVec`,`RBox`),
and storing a pointer to the vtable as a field in the type.
Any method that needs it calls vtable functions to do the allocation/deallocation.

# Relating to global constructors in dynamic libraries

It is unsound to do load a library developed with abi_stable or
checking the layout of types with
`abi_stable::abi_stability::check_layout_compatibility` in global constructors,
because `abi_stable` relies on initializing its global global state in the binary
and passing that global state to dynamic libraries before loading any module.

It is **safe** however to use anything else from `abi_stable` in global constructors,

"Global constructor" is any code that runs before any library symbol can be loaded,
including those of abi_stable modules.

*/
