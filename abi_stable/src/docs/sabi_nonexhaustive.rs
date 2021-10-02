/*!

Using the `#[sabi(kind(WithNonExhaustive(...)))]` helper attribute for
`#[derive(StableAbi)]` allows you to store the enum
in `NonExhaustive`,
using it as a non-exhaustive enum across ffi.

The enum can then be wrapped in a
[`NonExhaustive<>`](../../nonexhaustive_enum/struct.NonExhaustive.html),
but can only be converted back into it if the discriminant is valid in that context.

Nonexhaustive enums can safely add variants in minor versions,
giving library authors some flexibility in their design.

# Items

These are the items relevant to nonexhaustive enums:

`Enum`: this is the annotated enum,which does not derive `StableAbi`,
requiring it to be wrapped in a `NonExhaustive<>` to be passed through ffi.

`Enum_NE`(generated): A type alias for `NonExhaustive<Enum,_,_>`.

`Enum_NEMarker`(generated):
A marker type which implements StableAbi with the layout of `Enum`,
used as a phantom field of NonExhaustive.

`Enum_Storage`(generated):
A type used as storage space by the `NonExhaustive<>` type to store the enum.

`Enum_Bounds`(generated):
Acts as an alias for the traits that were specified in the `traits(...)` parameter.
This is only created if the `traits(...)` parameter is specified.

`Enum_Interface`(generated):
Describes the traits required when constructing a `NonExhaustive<Enum,_,_>`
and usable with it afterwards
(this is a type that implements [`InterfaceType`]).

# Parameters

These are the required and optional parameters for the
`#[sabi(kind(WithNonExhaustive(...)))]` helper attribute.

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

- Send: Required by default, must be unrequired with `Send = false`

- Sync: Required by default, must be unrequired with `Sync = false`

- Clone

- Debug

- Display

- Serialize: serde::Serialize.Look below for clarifications on how to use serde.

- Deserialize: serde::Deserialize.Look below for clarifications on how to use serde.

- Eq

- PartialEq

- Ord

- PartialOrd

- Hash

- Error: std::error::Error

### Interface (optional parameter)

This allows using a pre-existing to specify which traits are
required when constructing `NonExhaustive<>` from this enum and are then usable with it.

The type describes which traits are required using the [`InterfaceType`] trait.

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


### NonExhaustive assertions

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

# `serde` support

`NonExhaustive<Enum,Storage,Interface>` only implements serde::{Serialize,Deserialize}
if Interface allows them in its [`InterfaceType`] implementation,
and also implements the [`SerializeEnum`] and [`DeserializeEnum`] traits.

### Defining a (de)serializable nonexhaustive enum.

This defines a nonexhaustive enum and demonstrates how it is (de)serialized.

For a more realistic example you can look at the
"examples/2_nonexhaustive/interface" crate in the repository for this crate.

```

use abi_stable::{
    StableAbi,
    rtry,
    sabi_extern_fn,
    external_types::{RawValueBox,RawValueRef},
    nonexhaustive_enum::{NonExhaustive,SerializeEnum,DeserializeEnum},
    prefix_type::{PrefixTypeTrait, WithMetadata},
    std_types::{RBoxError,RString,RStr,RResult,ROk,RErr},
    traits::IntoReprC,
};

use serde::{Deserialize,Serialize};

#[repr(u8)]
#[derive(StableAbi,Debug,Clone,PartialEq,Deserialize,Serialize)]
#[sabi(kind(WithNonExhaustive(
    // Determines the maximum size of this enum in semver compatible versions.
    size="[usize;10]",
    // Determines the traits that are required when wrapping this enum in NonExhaustive,
    // and are then available with it.
    traits(Debug,Clone,PartialEq,Serialize,Deserialize),
)))]
// The `#[sabi(with_constructor)]` helper attribute here generates constructor functions
// that look take the fields of the variant as parameters and return a `ValidTag_NE`.
#[sabi(with_constructor)]
pub enum ValidTag{
    #[doc(hidden)]
    __NonExhaustive,
    Foo,
    Bar,
    Tag{
        name:RString,
        tag:RString,
    }
}

/*
//This was generated by the StableAbi derive macro on ValidTag.
pub type ValidTag_NE=
    NonExhaustive<
        ValidTag,
        ValidTag_Storage,
        ValidTag_Interface,
    >;
*/

/// This describes how the enum is serialized.
impl SerializeEnum<ValidTag_NE> for ValidTag_Interface {
    /// A type that `ValidTag_NE` is converted into(inside `SerializeEnum::serialize_enum`),
    /// and then serialized.
    type Proxy=RawValueBox;

    fn serialize_enum(this:&ValidTag_NE) -> Result<RawValueBox, RBoxError>{
        Module::VALUE
            .serialize_tag()(this)
            .into_result()

    }
}

/// This describes how the enum is deserialized.
impl<'a> DeserializeEnum<'a,ValidTag_NE> for ValidTag_Interface{
    /// A type that is deserialized,
    /// and then converted into `ValidTag_NE` inside `DeserializeEnum::deserialize_enum`.
    type Proxy=RawValueRef<'a>;

    fn deserialize_enum(s: RawValueRef<'a>) -> Result<ValidTag_NE, RBoxError>{
        Module::VALUE
            .deserialize_tag()(s.get_rstr())
            .into_result()
    }
}


# fn main(){

assert_eq!(
    serde_json::from_str::<ValidTag_NE>(r#""Foo""#).unwrap(),
    ValidTag::Foo_NE()
);

assert_eq!(
    serde_json::from_str::<ValidTag_NE>(r#""Bar""#).unwrap(),
    ValidTag::Bar_NE()
);

assert_eq!(
    serde_json::from_str::<ValidTag_NE>(r#"
        {"Tag":{
            "name":"what",
            "tag":"the"
        }}
    "#).unwrap(),
    ValidTag::Tag_NE("what".into(),"the".into())
);



assert_eq!(
    &serde_json::to_string(&ValidTag::Foo_NE()).unwrap(),
    r#""Foo""#,
);

assert_eq!(
    &serde_json::to_string(&ValidTag::Bar_NE()).unwrap(),
    r#""Bar""#,
);

# }

// In this struct:
//
// - `#[sabi(kind(Prefix))]`
// Declares this type as being a prefix-type, generating both of these types:
//
//     - Module_Prefix`: A struct with the fields up to (and including) the field with the
//     `#[sabi(last_prefix_field)]` attribute.
//
//     - Module_Ref`: An ffi-safe pointer to a `Module`,with methods to get `Module`'s fields.
//
// - `#[sabi(missing_field(panic))]`
//     makes the field accessors of `ModuleRef` panic when attempting to
//     access nonexistent fields instead of the default of returning an Option<FieldType>.
//
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
#[sabi(missing_field(panic))]
pub struct Module{
    pub serialize_tag:extern "C" fn(&ValidTag_NE)->RResult<RawValueBox,RBoxError>,

    /// `#[sabi(last_prefix_field)]`means that it is the last field in the struct
    /// that was defined in the first compatible version of the library
    /// (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    /// requiring new fields to always be added below preexisting ones.
    #[sabi(last_prefix_field)]
    pub deserialize_tag:extern "C" fn(s:RStr<'_>)->RResult<ValidTag_NE,RBoxError>,
}

// This is how you can construct `Module` in a way that allows it to become generic later.
impl Module {
    // This macro declares a `StaticRef<WithMetadata<BoxVtable<T>>>` constant.
    //
    // StaticRef represents a reference to data that lives forever,
    // but is not necessarily `'static` according to the type system.
    //
    // StaticRef not necessary in this case, it's more useful with generic types..
    abi_stable::staticref!(const TMP0: WithMetadata<Self> = WithMetadata::new(
        PrefixTypeTrait::METADATA,
        Self{
            serialize_tag,
            deserialize_tag,
        },
    ));

    const VALUE: Module_Ref = Module_Ref( Self::TMP0.as_prefix() );
}

/////////////////////////////////////////////////////////////////////////////////////////
////   In implementation crate (the one that gets compiled as a dynamic library)    /////
/////////////////////////////////////////////////////////////////////////////////////////

#[sabi_extern_fn]
pub fn serialize_tag(enum_:&ValidTag_NE)->RResult<RawValueBox,RBoxError>{
    let enum_=rtry!( enum_.as_enum().into_c() );

    match serde_json::to_string(&enum_) {
        Ok(v)=>{
            RawValueBox::try_from_string(v)
                .map_err(RBoxError::new)
                .into_c()
        }
        Err(e)=>RErr(RBoxError::new(e)),
    }
}

#[sabi_extern_fn]
pub fn deserialize_tag(s:RStr<'_>)->RResult<ValidTag_NE,RBoxError>{
    match serde_json::from_str::<ValidTag>(s.into()) {
        Ok(x) => ROk(NonExhaustive::new(x)),
        Err(e) => RErr(RBoxError::new(e)),
    }
}


```


# Example,boxing variants of unknown size

This example demonstrates how one can use boxing to store types larger than `[usize;2]`
(the size of `RBox<_>`),
because one of the variant contains a generic type.



```
use abi_stable::{
    StableAbi,
    nonexhaustive_enum::{NonExhaustiveFor,NonExhaustive},
    std_types::{RBox,RString},
    sabi_trait,
};

use std::{
    cmp::PartialEq,
    fmt::{self,Debug,Display},
};


#[repr(u8)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
#[sabi(kind(WithNonExhaustive(
    size="[usize;3]",
    traits(Debug,Display,Clone,PartialEq),
)))]
pub enum Message<T>{
    #[doc(hidden)]
    __NonExhaustive,
    SaysHello,
    SaysGoodbye,

    #[sabi(with_boxed_constructor)]
    Custom(RBox<T>),

    ////////////////////////////////////////
    // Available since 1.1
    ////////////////////////////////////////
    #[sabi(with_boxed_constructor)]
    SaysThankYou(RBox<SaysThankYou>)

}


impl<T> Display for Message<T>
where
    T:Display
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        match self {
            Message::__NonExhaustive=>unreachable!(),
            Message::SaysHello=>write!(f,"Hello!"),
            Message::SaysGoodbye=>write!(f,"Goodbye!"),
            Message::Custom(custom)=>Display::fmt(&**custom,f),
            Message::SaysThankYou(x)=>writeln!(f,"Thank you,{}!",x.to),
        }
    }
}


// Only available since 1.1
#[repr(C)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
pub struct SaysThankYou{
    to:RString,
}

# fn main(){

// Constructing Message::Custom wrapped in a NonExhaustive
{
    let custom_message:Message_NE<RString>=
        Message::Custom_NE("Hello".into());

    let custom_message_desugar:Message_NE<RString>={
        let x=RBox::new("Hello".into());
        let x=Message::Custom(x);
        NonExhaustive::new(x)
    };

    assert_eq!(custom_message,custom_message_desugar);
}


// Constructing Message::SaysThankYou wrapped in a NonExhaustive
// This variant is only available since 1.1
{
    let says_thank_you:Message_NE<RString>=
        Message::SaysThankYou_NE(SaysThankYou{
            to:"Hello".into(),
        });

    let says_thank_you_desugar:Message_NE<RString>={
        let x=SaysThankYou{to:"Hello".into()};
        let x=Message::SaysThankYou(RBox::new(x));
        NonExhaustive::new(x)
    };

    assert_eq!(says_thank_you,says_thank_you_desugar);
}

# }

```



# Example

This example shows how one can use RSmallBox to define a generic nonexhausitve enum.

```

use abi_stable::{
    sabi_types::RSmallBox,
    std_types::{RString,RVec},
    reexports::SelfOps,
    StableAbi,
};

#[repr(u8)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
#[sabi(kind(WithNonExhaustive(
    // Determines the maximum size of this enum in semver compatible versions.
    // This is 11 usize large because:
    //    - The enum discriminant occupies 1 usize(because the enum is usize aligned).
    //    - RSmallBox<T,[usize;8]>: is 10 usize large
    size="[usize;11]",
    // Determines the traits that are required when wrapping this enum in NonExhaustive,
    // and are then available with it.
    traits(Debug,Clone,PartialEq),
)))]
#[sabi(with_constructor)]
pub enum SomeEnum<T>{
    #[doc(hidden)]
    __NonExhaustive,
    Foo,
    Bar,
    Crash{
        reason:RString,
        animal:RString,
    },
    // This variant was added in a newer (compatible) version of the library.
    #[sabi(with_boxed_constructor)]
    Other(RSmallBox<T,[usize;8]>)
}

impl<T> SomeEnum<T>{
    pub fn is_inline(&self)->bool{
        match self {
            SomeEnum::__NonExhaustive=>true,
            SomeEnum::Foo=>true,
            SomeEnum::Bar=>true,
            SomeEnum::Crash{..}=>true,
            SomeEnum::Other(rsbox)=>RSmallBox::is_inline(rsbox),
        }
    }

    pub fn is_heap_allocated(&self)->bool{
        !self.is_inline()
    }

}


#[repr(C)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
pub struct FullName{
    pub name:RString,
    pub surname:RString,
}


/// A way to represent a frozen `Vec<Vec<T>>`.
///
/// This example just constructs NestedVec directly,
/// realistically it would be constructed in an associated function of NestedVec.
#[repr(C)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
pub struct NestedVec<T>{
    indices:RVec<usize>,
    nested:RVec<T>,
    dummy_field:u32,
}


# fn main(){

let crash=SomeEnum::<()>::Crash_NE("No reason".into(),"Bandi____".into());

let other_fullname=
    SomeEnum::Other_NE(FullName{ name:"R__e".into(), surname:"L_____e".into() });

let other_nestedlist={
    let nestedlist=NestedVec{
        indices:vec![0,2,3,5].into(),
        // Each line here is a nested list.
        nested:vec![
            false,false,
            true,
            true,false,
            true,true,true,
        ].into(),
        dummy_field:0,
    };
    SomeEnum::Other_NE(nestedlist)
};




assert!( crash.as_enum().unwrap().is_inline() );
assert!( other_fullname.as_enum().unwrap().is_inline() );
assert!( other_nestedlist.as_enum().unwrap().is_heap_allocated() );


# }



```

# Example

Say that we want to define a "private" enum
(it's exposed to the ABI but it's not public API),
used internally to send information between instances of the same library,
of potentially different (compatible) versions.

If one of the variants from newer versions are sent into a library/binary
that has a previous version of `Event`,
`Event_NE` (an alias for NonExhaustive wrapping an Event)
won't be convertible back into `Event`.

```
use abi_stable::{
    StableAbi,
    nonexhaustive_enum::{NonExhaustiveFor,NonExhaustive},
    std_types::{RString,RArc},
    sabi_trait,
};


#[doc(hidden)]
#[repr(C)]
#[derive(StableAbi,Debug,Clone,Copy,PartialEq)]
pub struct ObjectId(
    pub usize
);

#[doc(hidden)]
#[repr(C)]
#[derive(StableAbi,Debug,Clone,Copy,PartialEq)]
pub struct GroupId(
    pub usize
);


#[repr(u8)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
#[sabi(kind(WithNonExhaustive(
    size="[usize;8]",
    traits(Debug,Clone,PartialEq),
)))]
#[sabi(with_constructor)]
pub enum Event{
    #[doc(hidden)]
    __NonExhaustive,
    CreatedInstance{
        object_id:ObjectId,
    },
    RemovedInstance{
        object_id:ObjectId,
    },

    /////////////////
    // Added in 1.1
    /////////////////
    CreatedGroup{
        name:RString,
        group_id:GroupId,
    },
    RemovedGroup{
        name:RString,
        group_id:GroupId,
    },
    AssociatedWithGroup{
        object_id:ObjectId,
        group_id:GroupId,
    },

    /////////////////
    // Added in 1.2
    /////////////////
    RemovedAssociationWithGroup{
        object_id:ObjectId,
        group_id:GroupId,
    },
    #[sabi(with_boxed_constructor)]
    DummyVariant{
        pointer:RArc<()>,
    },
}

let objectid_0=ObjectId(0);
let objectid_1=ObjectId(1);

let groupid_0=GroupId(0);
let groupid_1=GroupId(0);

// Constructing a Event::CreatedInstance wrapped in a NonExhaustive
{
    let from_ne_constructor:Event_NE=
        Event::CreatedInstance_NE(objectid_0);
    let regular={
        let ev=Event::CreatedInstance{object_id:objectid_0};
        NonExhaustive::new(ev)
    };

    assert_eq!(from_ne_constructor,regular);
}

// Constructing a Event::RemovedInstance wrapped in a NonExhaustive
{
    let from_ne_constructor=Event::RemovedInstance_NE(objectid_0);
    let regular={
        let ev=Event::RemovedInstance{object_id:objectid_0};
        NonExhaustive::new(ev)
    };

    assert_eq!(from_ne_constructor,regular);
}

// Constructing a Event::RemovedInstance wrapped in a NonExhaustive
{
    let from_ne_constructor=Event::RemovedInstance_NE(objectid_0);
    let regular={
        let ev=Event::RemovedInstance{object_id:objectid_0};
        NonExhaustive::new(ev)
    };

    assert_eq!(from_ne_constructor,regular);
}

// Constructing a Event::CreatedGroup wrapped in a NonExhaustive
// This is only available from 1.1
{
    let from_ne_constructor=Event::CreatedGroup_NE("hello".into(),groupid_0);
    let regular={
        let ev=Event::CreatedGroup{name:"hello".into(),group_id:groupid_0};
        NonExhaustive::new(ev)
    };

    assert_eq!(from_ne_constructor,regular);
}

// Constructing a Event::RemovedGroup wrapped in a NonExhaustive
// This is only available from 1.1
{
    let from_ne_constructor=Event::RemovedGroup_NE("hello".into(),groupid_0);
    let regular={
        let ev=Event::RemovedGroup{name:"hello".into(),group_id:groupid_0};
        NonExhaustive::new(ev)
    };

    assert_eq!(from_ne_constructor,regular);
}


// Constructing a Event::AssociatedWithGroup wrapped in a NonExhaustive
// This is only available from 1.1
{
    let from_ne_constructor=Event::AssociatedWithGroup_NE(objectid_0,groupid_0);
    let regular={
        let ev=Event::AssociatedWithGroup{
            object_id:objectid_0,
            group_id:groupid_0,
        };
        NonExhaustive::new(ev)
    };

    assert_eq!(from_ne_constructor,regular);
}


// Constructing a Event::RemovedAssociationWithGroup wrapped in a NonExhaustive
// This is only available from 1.2
{
    let from_ne_constructor=Event::RemovedAssociationWithGroup_NE(objectid_0,groupid_0);
    let regular={
        let ev=Event::RemovedAssociationWithGroup{
            object_id:objectid_0,
            group_id:groupid_0,
        };
        NonExhaustive::new(ev)
    };

    assert_eq!(from_ne_constructor,regular);
}

// Constructing a Event::DummyVariant wrapped in a NonExhaustive
// This is only available from 1.2
{
    let from_ne_constructor=Event::DummyVariant_NE(());
    let regular={
        let x=RArc::new(());
        let x=Event::DummyVariant{
            pointer:x
        };
        NonExhaustive::new(x)
    };

    assert_eq!(from_ne_constructor,regular);
}

```



[`InterfaceType`]: ../../trait.InterfaceType.html
[`SerializeEnum`]: ../../nonexhaustive_enum/trait.SerializeEnum.html
[`DeserializeEnum`]: ../../nonexhaustive_enum/trait.DeserializeEnum.html

*/
