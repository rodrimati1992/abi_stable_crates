/*!

Using the `#[sabi(kind(WithNonExhaustive(...)))]` subattribute for 
[`#[derive(StableAbi)]`](../stable_abi_derive/index.html) allows you to store the enum
in 
[`NonExhaustive<>`](../../nonexhaustive_enum/nonexhaustive/struct.NonExhaustive.html),
using it as a non-exhaustive enum across ffi.

The enum can then be wrapped in a 
[`NonExhaustive<>`](../../nonexhaustive_enum/nonexhaustive/struct.NonExhaustive.html),
but can only be converted back into it if the discriminant is valid in that context.

Nonexhaustive enums can safely add variants in minor versions,
giving library authors some flexibility in their design.

# Items 

`Enum`: this is the annotated enum,which does not derive `StableAbi`,
requiring it to be wrapped in a `NonExhaustive<>` to be passed through ffi.

`Enum_NE`: A type alias for the deriving type wrapped in a `NonExhaustive<>`.

`Enum_NEMarker`:
A marker type which implements StableAbi with the layout of `Enum`,
used as a phantom field of NonExhaustive.

`Enum_Storage`:
A type used as storage space by the `NonExhaustive<>` type to store the enum.

`Enum_Bounds`:
Acts as an alias for the traits that were specified in the `traits(...)` parameter.
This is only created if the `traits(...)` parameter is specified.

`Enum_Interface`:
Describes the traits required when constructing a `NonExhaustive<>` and usable with it
(this is a type that implements InterfaceType).<br>
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

*/