# #[sabi(kind(..))] attribute

Specifies the StabilityKind of a type.

Valid values for this are:

- #[sabi(kind(Value))]
- #[sabi(kind(unsafe_Prefix))]
- #[sabi(kind(MutPointer))]

# Supported repr attributes

### `repr(C)`

This is the representation that most StableAbi types will have.

### `repr(transparent)`

`repr(transparent)` types inherit the abi stability of their first field.

### `repr(align(...))`

`repr(align(...))` is supported,
so long as it is used in combination with the other supported repr attributes.

# StabilityKind

This determines how abi stability works for a type.


### Value kind

A type for which it is invalid to add fields in minor versions.
This is the default kind when deriving `StableAbi`


# NonZero

Some standard library types have a single value that is invalid for them eg:0,null.
NonZero types are the only which can be stored in a `Option<_>` while implementing AbiStable.

As an alternative for other types you can use `abi_stable::ROption`.

Non-exhaustive list of std types that are NonZero:

- &T (any T).

- &mut T (any T).

- extern fn() :
    Any combination of StableAbi parameter/return types.
    These can't be hidden behind a type alias.

- std::ptr::NonNull

- std::num::{NonZeroU8,NonZeroU16,NonZeroU32,NonZeroU64,NonZeroU128,NonZeroUsize} 

