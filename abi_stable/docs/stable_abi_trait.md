# #[sabi(kind(..))] attribute

Specifies the StabilityKind of a type.

Valid values for this are:

- #[sabi(kind(Value))]
- #[sabi(kind(Prefix))]
- #[sabi(kind(MutPointer))]

# `#[sabi(override(..))]` attribute

Allows overriding something about the generated code



# Supported repr attributes

### `repr(C)`

This is the representation that most StableAbi types will have.

### `repr(transparent)`

`repr(transparent)` types inherit the abi stability of their first field.


# StabilityKind

This determines how abi stability works for a type.


### Value kind

A type for which it is invalid to add fields in minor versions.

### Prefix kind

A type for which it is valid to add fields at the end in minor versions.

The type must satisfy these properties:

    - Every version must contain every previous version as a prefix.

    - It must be non-Clone/non-Copy,and have a private constructor.
        This is prevent a user of the library from creating a value of this type
        that is smaller than the implementation provides.

    - It be hidden behind a shared pointer (&/*const/RArc) to implement StableAbi.

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
