/*!

This attribute generates an ffi-safe trait object on the trait it's applied to.

# Pseudo-types

These types can be used in the trait definition,
and will be replaced with the appropriate value in the generated code:

- TraitObject_:Is the type of the trait object,determined when it is constructed.

# Associated types

The only valid way to refer to associated types in the trait declaration is with 
`Self::AssocType` syntax.

Associated types in the trait object are transformed into type parameters 
that come before those of the trait.

# Object safety

Trait objects generated using this attribute have similar restrictions to built-in trait objects:

- `Self` can only be used to access associated types 
    (using the `Self::AssocType` syntax).

- `self` is a valid method receiver,
    this requires that the pointer that the generated trait object wraps implements `abi_stable::traits::IntoInner`.



*/