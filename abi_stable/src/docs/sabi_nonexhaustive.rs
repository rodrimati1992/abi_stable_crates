/*!

Using the `#[sabi(kind(WithNonExhaustive(...)))]` subattribute for 
[`#[derive(StableAbi)]`](../stable_abi_derive/index.html) allows you to store the enum
in 
[`NonExhaustive<>`](../../nonexhaustive_enum/nonexhaustive/struct.NonExhaustive.html),
using it as a non-exhaustive enum across ffi.

The enum can then be wrapped in a 
[`NonExhaustive<>`](../../nonexhaustive_enum/nonexhaustive/struct.NonExhaustive.html),
but can only be converted back into it if the discriminant is valid in that context.

# Items 

`Enum`: this is the annotated enum,which does not derive `StableAbi`,
requiring it to be wrapped in a `NonExhaustive<>` to be passed through ffi.

`Enum_NEMarker`:
A marker type which implements StableAbi with the layout of `Enum`,
used as a phantom field of NonExhaustive.

`Enum_Storage`:
A type used as storage space by the `NonExhaustive<>` type to store the enum.

`Enum_Interface`:
Describes the traits required when constructing a `NonExhaustive<>` and usable with it.<br>
This is only created if the `traits` parameter is passed to `#[sabi(kind(WithNonExhaustive(..)))]`.

# Parameters

These are the required and optional parameters for the 
`#[sabi(kind(WithNonExhaustive(...)))]` subattribute.

### Specifying alignment (optional parameter)

Specifies the alignment of Enum_Storage.

With a specific alignemnt.<br>
Syntax:`align=integer_literal`<br>
Example:`align=8`<br>

With the same alignment is that of another type.<br>
Syntax:`align="type"`<br>
Example:`align="usize"`<br>

### size (required parameter)

Specifies the size of Enum_Storage.

The size of Enum_TE in bytes.<br>
Syntax:`size=integer_literal`<br>
Example:`size=8`<br>

The size of Enum_TE is that of of another type<br>
Syntax:`size="type"`<br>
Example:`size="[usize;8]"`<br>
Recommendation:
Use a type that has a constant layout,generally a concrete type.
It is a bad idea to use `Enum` since its size is allowed to change.<br>

### Traits (optional parameter)

Specifies the traits required when constructing NonExhaustive from this enum and 
usable after constructing it.

If neither this parameter nor interface are specified,
no traits will be required in `NonExhaustive<>` and none will be usable.

Syntax:`traits( trait0,trait1=false,trait2=true,trait3 )`

Example0:`traits(Debug,Display)`<br>
Example1:`traits(Sync=false,Debug,Display)`<br>
Example2:`traits(Sync=false,Send=false,Debug,Display)`<br>
Example3:`traits(Clone,Debug,Display,Error)`<br>

All the traits are optional.

These are the valid traits:

- Send:Which is required by default,you must write `Send=false` to unrequire it.

- Sync:Which is required by default,you must write `Sync=false` to unrequire it.

- Clone

- Debug

- Display

- Serialize: serde::Serialize

- Deserialize: serde::Deserialize

- Eq

- PartialEq

- Ord

- PartialOrd

- Hash

- Error: std::error::Error

### Interface (optional parameter)

This is like `traits(..)` in that it allows specifying which traits are 
required when constructing `NonExhaustive<>` from this enum and are then usable with it.
The difference is that this allows one to specify a pre-existing InterfaceType,
instead of generating a new one (that is `Enum_Interface`).

Syntax:`interface="type"`

Example0:`interface="()"`.
This means that no trait is usable/required.<br>

Example1:`interface="CloneInterface"`.
This means that only Clone is usable/required.<br>

Example2:`interface="PartialEqInterface"`.
This means that only Debug/PartialEq are usable/required.<br>

Example3:`interface="CloneEqInterface"`.
This means that only Debug/Clone/Eq/PartialEq are usable/required.<br>

The `*Interface` types from the examples come from the 
`abi_stable::erased_types::interfaces` module.



# NonExhaustive assertions

This generates a test that checks that the listed types can be stored within `NonExhaustive`.

You must run those tests with `cargo test`,they are not static assertions.

Once static assertions can be done in a non-hacky way,
this library will provide another attribute which generates static assertions.

Syntax:`assert_nonexhaustive="type" )`<br>
Example:`assert_nonexhaustive="Foo<u8>")`<br>
Example:`assert_nonexhaustive="Foo<RArc<u8>>")`<br>
Example:`assert_nonexhaustive="Foo<RBox<u8>>")`<br>

Syntax:`assert_nonexhaustive("type0","type1")`<br>
Example:`assert_nonexhaustive("Foo<RArc<u8>>")`<br>
Example:`assert_nonexhaustive("Foo<u8>","Foo<RVec<()>>")`<br>

*/