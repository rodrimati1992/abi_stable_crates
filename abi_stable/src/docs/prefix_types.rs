/*!

Prefix-types are types that derive StableAbi along with the 
`#[sabi(kind(Prefix(prefix_ref="PrefixEquivalent")))]` helper attribute.
This is mostly intended for **vtables** and **modules**.

Prefix-types cannot directly be passed through ffi,
instead they must be converted to the type declared with `prefix_ref="PrefixEquivalent"`,
and then pass `&PrefixEquivalent`/`StaticRef<PrefixEquivalent>` instead.

To convert `T` to `&PrefixEquivalent`/`StaticRef<PrefixEquivalent>` you an use one of:

- `PrefixTypeTrait::leak_into_prefix`:<br>
    Which does the conversion directly,but leaks the value.

- `prefix_type::WithMetadata::new` and then `WithMetadata::ref_as_prefix`:<br>
    Use this if you need a compiletime constant and the type has no generic parameters.
    First create a `&'static WithMetadata<Self>` constant,
    then use the `WithMetadata::ref_as_prefix` method at runtime 
    to cast it to `&'static PrefixEquivalent`.

- `prefix_type::WithMetadata::new` and then `WithMetadata::as_prefix`:<br>
    Use this if you need a compiletime constant.
    First create a `StaticRef<WithMetadata<Self>>` constant using 
    the `StaticRef::from_raw` function,
    then use `WithMetadata::as_prefix` to cast it to `StaticRef<PrefixEquivalent>`.


All fields on `&PrefixEquivalent`/`StaticRef<PrefixEquivalent>` are accessed through accessor methods 
with the same name as the fields.

To ensure that libraries stay abi compatible,
the first minor version of the library must apply the `#[sabi(last_prefix_field)]` to some 
field and every minor version after that must add fields at the end (never moving that attribute).
Changing the field that `#[sabi(last_prefix_field)]` is applied to is a breaking change.

Getter methods for fields after the one to which `#[sabi(last_prefix_field)]` was applied to
will return `Option<FieldType>` by default,because those fields might not exist 
(the struct might come from a previous version of the library).
To override how to deal with nonexistent fields,
use the `#[sabi(missing_field())]` attribute,
applied to either the struct or the field.

# Grammar Reference

For the grammar reference,you can look at the documentation for 
[`#[derive(StableAbi)]`](../stable_abi_derive/index.html).

# Examples

###  Example 1 

Declaring a Prefix-type.

```

use abi_stable::{
    StableAbi,
    std_types::{RDuration,RStr},
};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref="Module")))]
#[sabi(missing_field(panic))]
pub struct ModuleVal {
    pub lib_name:RStr<'static>,

    #[sabi(last_prefix_field)]
    pub elapsed:extern "C" fn()->RDuration,

    pub description:RStr<'static>,
}

# fn main(){}

```

In this example:

- `#[sabi(kind(Prefix(prefix_ref="Module")))]` declares this type as being a prefix-type
    with an ffi-safe equivalent called `Module` to which `ModuleVal` can be converted into.

- `#[sabi(missing_field(panic))]` 
    makes the field accessors panic when attempting to 
    access nonexistent fields instead of the default of returning an Option<FieldType>.

- `#[sabi(last_prefix_field)]`means that it is the last field in the struct
    that was defined in the first compatible version of the library
    (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    requiring new fields to always be added bellow preexisting ones.

###  Example 2:Declaring a type with a VTable 

Here is the implementation of a Box-like type,which uses a VTable that is itself a Prefix.

```

use std::{
    ops::{Deref,DerefMut},
    marker::PhantomData,
    mem::ManuallyDrop,
};

use abi_stable::{
    StableAbi,
    extern_fn_panic_handling,
    staticref,
    pointer_trait::{CallReferentDrop, TransmuteElement},
    prefix_type::{PrefixTypeTrait, WithMetadata},
};

/// An ffi-safe `Box<T>`
#[repr(C)]
#[derive(StableAbi)]
pub struct BoxLike<T> {
    data: *mut T,
    
    vtable: BoxVtable_Ref<T>,

    _marker: PhantomData<T>,
}


impl<T> BoxLike<T>{
    pub fn new(value:T)->Self{
        let box_=Box::new(value);
        
        Self{
            data:Box::into_raw(box_),
            vtable: BoxVtable::VTABLE,
            _marker:PhantomData,
        }
    }

    fn vtable(&self)-> BoxVtable_Ref<T>{
        self.vtable
    }

    /// Extracts the value this owns.
    pub fn into_inner(self)->T{
        let this=ManuallyDrop::new(self);
        let vtable=this.vtable();
        unsafe{
            // Must copy this before calling `vtable.destructor()`
            // because otherwise it would be reading from a dangling pointer.
            let ret=this.data.read();
            vtable.destructor()(this.data,CallReferentDrop::No);
            ret
        }
    }
}


impl<T> Deref for BoxLike<T> {
    type Target=T;

    fn deref(&self)->&T{
        unsafe{
            &(*self.data)
        }
    }
}

impl<T> DerefMut for BoxLike<T> {
    fn deref_mut(&mut self)->&mut T{
        unsafe{
            &mut (*self.data)
        }
    }
}


impl<T> Drop for BoxLike<T>{
    fn drop(&mut self){
        let vtable=self.vtable();

        unsafe{
            vtable.destructor()(self.data,CallReferentDrop::Yes)
        }
    }
}


// `#[sabi(kind(Prefix))]` Declares this type as being a prefix-type,
// generating both of these types:
// 
//     - BoxVTable_Prefix`: A struct with the fields up to (and including) the field with the 
//     `#[sabi(last_prefix_field)]` attribute.
// 
//     - BoxVTable_Ref`: An ffi-safe pointer to a `BoxVtable`, with methods to get
//     `BoxVtable`'s fields.
// 
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
pub(crate) struct BoxVtable<T> {

    /// The `#[sabi(last_prefix_field)]` attribute here means that this is 
    /// the last field in this struct that was defined in the 
    /// first compatible version of the library
    /// (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    /// requiring new fields to always be added after it.
    /// 
    /// The `#[sabi(last_prefix_field)]` attribute would stay on this field until the library 
    /// bumps its "major" version,
    /// at which point it would be moved to the last field at the time.
    ///
    #[sabi(last_prefix_field)]
    destructor: unsafe extern "C" fn(*mut T, CallReferentDrop),
}


// This is how ffi-safe pointers to generic prefix types are constructed
// at compile-time.
impl<T> BoxVtable<T>{
    // This macro declares a `StaticRef<WithMetadata<BoxVtable<T>>>` constant.
    //
    // StaticRef represents a reference to data that lives forever,
    // but is not necessarily `'static` according to the type system,
    // eg: `BoxVtable<T>`.
    staticref!(const VTABLE_VAL: WithMetadata<Self> = WithMetadata::new(
        PrefixTypeTrait::METADATA,
        Self{
            destructor:destroy_box::<T>,
        },
    ));

    const VTABLE: BoxVtable_Ref<T> = {
        BoxVtable_Ref( Self::VTABLE_VAL.as_prefix() )
    };
}

unsafe extern "C" fn destroy_box<T>(v: *mut T, call_drop: CallReferentDrop) {
    extern_fn_panic_handling! {
        let mut box_ = Box::from_raw(v as *mut ManuallyDrop<T>);
        if call_drop == CallReferentDrop::Yes {
            ManuallyDrop::drop(&mut *box_);
        }
        drop(box_);
    }
}

# fn main(){}

```


###  Example 3:module 

This declares,initializes,and uses a module.

```
use abi_stable::{
    StableAbi,
    sabi_extern_fn,
    std_types::RDuration,
    prefix_type::{PrefixTypeTrait, WithMetadata},
};


// `#[sabi(kind(Prefix))]` Declares this type as being a prefix-type,
// generating both of these types:
// 
//     - PersonMod_Prefix`: A struct with the fields up to (and including) the field with the 
//     `#[sabi(last_prefix_field)]` attribute.
// 
//     - PersonMod_Ref`:
//      An ffi-safe pointer to a `PersonMod`,with methods to get`PersonMod`'s fields.
// 
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
pub struct PersonMod {

    /// The `#[sabi(last_prefix_field)]` attribute here means that this is 
    /// the last field in this struct that was defined in the 
    /// first compatible version of the library
    /// (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    /// requiring new fields to always be added bellow preexisting ones.
    /// 
    /// The `#[sabi(last_prefix_field)]` attribute would stay on this field until the library 
    /// bumps its "major" version,
    /// at which point it would be moved to the last field at the time.
    ///
    #[sabi(last_prefix_field)]
    pub customer_for: extern "C" fn(Id)->RDuration,

    // The default behavior for the getter is to return an Option<FieldType>,
    // if the field exists it returns Some(_),
    // otherwise it returns None.
    pub bike_count: extern "C" fn(Id)->u32,

    // The getter for this field panics if the field doesn't exist.
    #[sabi(missing_field(panic))]
    pub visits: extern "C" fn(Id)->u32,

    // The getter for this field returns `default_score()` if the field doesn't exist.
    #[sabi(missing_field(with="default_score"))]
    pub score: extern "C" fn(Id)->u32,
    
    // The getter for this field returns `Default::default()` if the field doesn't exist.
    #[sabi(missing_field(default))]
    pub visit_length: Option< extern "C" fn(Id)->RDuration >,

}

fn default_score()-> extern "C" fn(Id)->u32 {
    extern "C" fn default(_:Id)->u32{
        1000
    }

    default
}

type Id=u32;

#
#   static VARS:&[(RDuration,u32)]=&[
#       (RDuration::new(1_000,0),10),
#       (RDuration::new(1_000_000,0),1),
#   ];
#
#   #[sabi_extern_fn]
#   fn customer_for(id:Id)->RDuration{
#       VARS[id as usize].0
#   }
# 
#   #[sabi_extern_fn]
#   fn bike_count(id:Id)->u32{
#       VARS[id as usize].1
#   }
#
#   #[sabi_extern_fn]
#   fn visits(id:Id)->u32{
#       VARS[id as usize].1
#   }
#
#   #[sabi_extern_fn]
#   fn score(id:Id)->u32{
#       VARS[id as usize].1
#   }
#

/*
    ...
    Elided function definitions
    ...
*/

# fn main(){

const MODULE: PersonMod_Ref ={
    PersonMod_Ref(
        WithMetadata::new(
            PrefixTypeTrait::METADATA,
            PersonMod{
                customer_for,
                bike_count,
                visits,
                score,
                visit_length:None,
            }
        ).static_as_prefix()
    )
};

// Getting the value for every field of `MODULE`.

let customer_for: extern "C" fn(Id)->RDuration = 
    MODULE.customer_for();

let bike_count: Option<extern "C" fn(Id)->u32> = 
    MODULE.bike_count();

let visits: extern "C" fn(Id)->u32=
    MODULE.visits();

let score: extern "C" fn(Id)->u32=
    MODULE.score();

let visit_length: Option<extern "C" fn(Id)->RDuration> =
    MODULE.visit_length();

# }


```

*/
