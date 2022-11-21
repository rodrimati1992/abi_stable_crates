/*!

This document describes what changes are valid/invalid for a library using `abi_stable`,

Note that all of these only applies to types that implement `StableAbi`,
and are checked when loading the dynamic libraries using
the functions in `abi_stable::library::RootModule`.
Those dynamic libraries use the [`export_root_module`] attribute on some function
that export the root module
([a struct of function pointers and other nested modules](../prefix_types/index.html)).


# Semver in/compatible changes

These are the changes to pre-existing data structures that are
allowed/disallowed in semver compatible versions
(0.y.z < 0.(y+1).0 , x.y.z < (x+1).0.0).

It is never allowed to remove fields or variants in newer versions of a library.

Types cannot be renamed.

A type cannot be replaced with a `#[repr(transparent)]` types wrappig it.

If you rename a field,remember to use the `#[sabi(rename=the_old_name)]` attribute,
field names are part of the ABI of a type.


### Structs

It's only valid to add fields to structs if they are
[prefix types (vtables or modules)](../prefix_types/index.html),
and only after the last field.


### Exhaustive Enums

It is not possible to add variants or fields to exhaustive enums.

Exhaustive enums being ones that are declared like this:
```rust
use abi_stable::{std_types::RString, StableAbi};

#[repr(u8)]
#[derive(StableAbi)]
enum Exhaustive {
    A,
    B(RString),
    C,
    D { hello: u32, world: i64 },
}

# fn main(){}

```

### Non-exhaustive enums

It's possible to add variants with either:

- Field-less enums implemented as a struct wrapping an integer,
    with associated constants as the variants.

- [Enums which use the
    `#[sabi(kind(WithNonExhaustive())]` attribute
    ](../sabi_nonexhaustive/index.html),
    wrapped inside `NonExhaustive<>`.

Neither one allow enums to change their size or alignment.

<br>

Example field-less enum implemented as a struct wrapping an integer:

```

use abi_stable::StableAbi;

#[repr(transparent)]
#[derive(StableAbi, Eq, PartialEq)]
pub struct Direction(u8);

impl Direction {
    pub const LEFT: Self = Direction(0);
    pub const RIGHT: Self = Direction(1);
    pub const UP: Self = Direction(2);
    pub const DOWN: Self = Direction(3);
}

# fn main(){}


```


### Unions

It's not possible to add fields to unions,this is not currently possible
because the original author of`abi_stable` didn't see any need for it
(if you need it create an issue for it).

# Semver in/compatible additions

It is always valid to declare new types in a library.

### Wrapping non-StableAbi types

You must be very careful when wrapping types which don't implement StableAbi
from external libraries,
since they might not guarantee any of these properties:

- Their size,alignment,representation attribute,layout in general.

- Their dependence on global state,
    which could cause undefined behavior if passed between dynamic libraries,
    or just be unpredictable.

The potential dependence on global state is why `abi_stable` uses dynamic dispatch
for all the types it wraps in `abi_stable::external_types`

# abi_stable specific

If you add StableAbi types to abi_stable,make sure to add them to the list of types in
`version_compatibility_interface::ManyTypes`
(the crate is in testing/version_compatibility/interface/)



[`export_root_module`]: crate::export_root_module
*/
