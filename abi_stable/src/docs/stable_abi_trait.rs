/*!

The StableAbi derive macro allows one to implement the StableAbi trait to :

- Assert that the type has a stable representation across Rust version/compiles.

- Produce the layout of the type at runtime to check it against the loaded library.

# Container Attributes

These attributes are applied on the type declaration.

### `#[sabi(phantom(TypeParameter))]`

Removes the implicit `TypeParameter:StableAbi` constraint.

### `#[sabi(bound="Type:ATrait")]`

Adds a bound to the StableAbi impl.

### `#[sabi(debug_print)]`

Prints the generated code,stopping compilation.

# Field attributes

These attributes are applied to fields.

### `#[sabi(unsafe_opaque_field)]`

Does not require the field to implement StableAbi,
and instead uses the StableAbi impl of `UnsafeOpaqueField<FieldType>`.

This is unsafe because the layout of the type won't be verified when loading the library,
which causes Undefined Behavior if the type has a different layout.

# Supported repr attributes

Because repr attributes can cause the type to change layout,
the StableAbi derive macro has to know about every repr attribute applied to the type,
since it might invalidate layout stability.

### `repr(C)`

This is the representation that most StableAbi types will have.

### `repr(transparent)`

`repr(transparent)` types inherit the abi stability of their first field.

### `repr(align(...))`

`repr(align(...))` is supported,
so long as it is used in combination with the other supported repr attributes.

*/