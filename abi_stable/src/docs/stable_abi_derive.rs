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

Adds a bound to the `StableAbi` impl.

### `#[sabi(prefix_bound="Type:ATrait")]`

This is only valid for Prefix types,declared with `#[sabi(kind(Prefix(..)))]`.

Adds a bound to the `PrefixTypeTrait` impl.

### `#[sabi(tag=" some_expr ")]`

Adds a "tag" associated with the type,
a dynamically typed data structure used to encode extra properties about a type.

This can only be done once,
to add multiple properties you must decide whether you want to use
a map,an array,or a set.

You can only rely on tags for safety if 
the specific tags were present since the first compatible version of the library,
otherwise this only guarantees compatibility between parent and child libraries,
not sibling libraries.

Parent means the library/binary that loaded a library,
or the parents of that one.

Sibling means libraries loaded at runtime by the same library/binary 
(or a parent of that one).

For more information about tags,[look here](../../abi_stability/tagging/index.html)


### `#[sabi(debug_print)]`

Prints the generated code,stopping compilation.

### `#[sabi(kind(Prefix( .. )))]`
Declares the struct as being a prefix-type.

`#[sabi(kind(Prefix(prefix_struct="NameOfPrefixStruct")))]`<br>
Uses "NameOfPrefixStruct" as the name of the prefix struct.


### `#[sabi(module_reflection(...))]` 

Determines how this type is accessed when treated as a module for reflection.

`#[sabi(module_reflection( Module ))]`<br>
The default reflection mode,treats its the public fields as module items.

`#[sabi(module_reflection( Opaque ))]`<br>
Treats this as an empty module.

`#[sabi(module_reflection( Deref ))]`<br>
Delegates the treatment of this type as a module to the type it dereferences to.



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

### `#[sabi(accessible_if=" expression ")]`

This is only valid for Prefix types,declared with `#[sabi(kind(Prefix(..)))]`.

This attribute turns any field conditional based on the const boolean expression 
(which must be valid a bool constant).

Whether this attribute is aplied to any given prefix field must not change in minor versions.

If `expression` is false,the field won't be accessible,
and the type of the field can be anything so long as its size and alignment is compatible.

If `expression` is true,the type of the field must be compatible when checking layout.

If this attribute is apllied to prefix fields,
it will only be compatible with other types if they agree on 
which accessors are conditional for prefix fields.

To do `#[sabi(accessible_if="<TypeParameter as Trait>::CONSTANT")]` you can use the 
`#[sabi(prefix_bound="TypeParameter:Trait")]` attribute.

### `#[sabi(refl(pub_getter=" function_name "))]`

Determines the public getter for a field used by reflection.

The function can return either a reference or a value.

# Field and/or Container attributes

### `#[sabi(missing_field( .. ))]`

This is only valid for Prefix types,declared with `#[sabi(kind(Prefix(..)))]`.

Determines what happens in the accessor method for a field,when the field is missing,
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

### `repr(i8|u8|i16|u16|i32|u32|i64|u64|isize|usize)`

These repr attributes are only supported for enums.

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