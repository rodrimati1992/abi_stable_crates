/*!

The StableAbi derive macro allows one to implement the StableAbi trait to :

- Assert that the type has a stable representation across Rust version/compiles.

- Produce the layout of the type at runtime to check it against the loaded library.

# Container Attributes

These attributes are applied on the type declaration.

### `#[sabi(unconstrained(TypeParameter))]` 

Removes the implicit `TypeParameter:StableAbi` constraint.

This is only necessary if you are passing `TypeParameter` to `UnsafeIgnoredType`

### `#[sabi(bound="Type:ATrait")]`

Adds a bound to the StableAbi impl.

### `#[sabi(debug_print)]`

Prints the generated code,stopping compilation.

### `#[sabi(kind(Prefix( .. )))]`
Declares the struct as being a prefix-type.

`#[sabi(kind(Prefix(prefix_struct="NameOfPrefixStruct")))]`<br>
Uses "NameOfPrefixStruct" as the name of the prefix struct.

`#[sabi(kind(Prefix(prefix_struct="default")))]`<br>
Generates the name of the prefix struct appending "\_Prefix" to the deriving struct's name.

# Field attributes

These attributes are applied to fields.

### `#[sabi(rename="ident")]`

Renames the field in the generated layout information.
Use this when renaming private fields.

### `#[sabi(unsafe_opaque_field)]`

Does not require the field to implement StableAbi,
and instead uses the StableAbi impl of `UnsafeOpaqueField<FieldType>`.

This is unsafe because the layout of the type won't be verified when loading the library,
which causes Undefined Behavior if the type has a different layout.


### `#[sabi(last_prefix_field)]`

This is only valid for Prefix types,declared with `#[sabi(kind(Prefix(..)))]`.

Declares that the field it is applied to is the last field in the prefix,
where every field up to it is guaranteed to exist.

# Field and/or Container attributes

### `#[sabi(missing_field( .. ))]`

This is only valid for Prefix types,declared with `#[sabi(kind(Prefix(..)))]`.

Determines what happens in the getter method for a field,when the field is missing,
the default is that it returns an `Option<FieldType>`,
returning None if the field is absent,Some(field_value) if it's present.

If the attribute is on the struct,it's applied to all fields(this is overridable)
after the `#[sabi(last_prefix_field)]` attribute.

If the attribute is on a field,it's applied to that field only,
overriding the setting on the struct.

`#[sabi(missing_field(panic))]`<br>
Panics if the field doesn't exist,with an informative error message.

`#[sabi(missing_field(option))]`<br>
Returns None if the field doesn't exist,Some(fieldvalue) if it does.
This is the default.

`#[sabi(missing_field(with="somefunction"))]`<br>
Returns `somefunction()` if the field doesn't exist.

`#[sabi(missing_field(default))]`<br>
Returns `Default::default()` if the field doesn't exist.




# Supported repr attributes

Because repr attributes can cause the type to change layout,
the StableAbi derive macro has to know about every repr attribute applied to the type,
since it might invalidate layout stability.

### `repr(C)`

This is the representation that most StableAbi types will have.

### `repr(transparent)`

`repr(transparent)` types are supported,
though their layout is not considered equivalent to their only non-zero-sized field,
since this library considers all types as being meaningful even if zero-sized.

### `repr(align(...))`

`repr(align(...))` is supported,
so long as it is used in combination with the other supported repr attributes.


# Examples 

### Basic example

```

use abi_stable::StableAbi;

#[repr(C)]
#[derive(StableAbi)]
struct Point2D{
    x:u32,
    y:u32,
}

```

### Prefix-types

For examples of Prefix-types [look here](../prefix_types/index.html#examples).


*/