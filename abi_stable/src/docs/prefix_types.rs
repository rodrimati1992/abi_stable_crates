/*!

Prefix-types are types that derive StableAbi along with the 
`#[sabi(kind(Prefix(prefix_struct="PrefixEquivalent")))]` attribute.
This is mostly intended for **vtables** and **modules**.

Prefix-types cannot directly be passed through ffi,
instead they must be converted to the type declared with `prefix_struct="PrefixEquivalent"`,
and then pass `&PrefixEquivalent` instead.

To convert `T` to `&PrefixEquivalent` use either:

- `PrefixTypeTrait::leak_into_prefix`:<br>
    Which does the conversion directly,but leaks the value.

- `prefix_type::WithMetadata::new` and then `WithMetadata::as_prefix`:<br>
    Use this if you need a compiletime constant.
    First create a `&'a WithMetadata<Self>` constant,
    then use the `WithMetadata::as_prefix` method at runtime 
    to cast it to `&PrefixEquivalent`.

All fields on `&PrefixEquivalent` are accessed through accessor methods 
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

### Example 1

Declaring a Prefix-type.

```

use abi_stable::{
    StableAbi,
    std_types::{RDuration,RStr},
};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="Module")))]
#[sabi(missing_field(panic))]
pub struct ModuleVal {
    pub lib_name:RStr<'static>,

    #[sabi(last_prefix_field)]
    pub elapsed:extern fn()->RDuration,

    pub description:RStr<'static>,
}

# fn main(){}

```

### Example 2:Declaring a type with a VTable

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
    pointer_trait::{CallReferentDrop, StableDeref, TransmuteElement},
    prefix_type::{PrefixTypeTrait,WithMetadata},
};

/// An ffi-safe `Box<T>`
#[repr(C)]
#[derive(StableAbi)]
pub struct BoxLike<T> {
    data: *mut T,
    
    // This can't be a `&'static BoxVtable<T>` because Rust will complain that 
    // `T` does not live for the `'static` lifetime.
    vtable: *const BoxVtable<T>,

    _marker: PhantomData<T>,
}


impl<T> BoxLike<T>{
    pub fn new(value:T)->Self
    where T:StableAbi
    {
        let box_=Box::new(value);
        
        Self{
            data:Box::into_raw(box_),
            vtable:unsafe{ (*BoxVtableVal::VTABLE).as_prefix() },
            _marker:PhantomData,
        }
    }

    // This is to get around a limitation of the type system where
    // vtables of generic types can't just be `&'static VTable<T>`
    // because it complains that T doesn't live forever.
    fn vtable<'a>(&self)->&'a BoxVtable<T>{
        unsafe{ &(*self.vtable) }
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


#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="BoxVtable")))]
pub(crate) struct BoxVtableVal<T> {
    #[sabi(last_prefix_field)]
    destructor: unsafe extern "C" fn(*mut T, CallReferentDrop),
}


impl<T> BoxVtableVal<T>
where T:StableAbi
{
    const TMP0:Self=Self{
        destructor:destroy_box::<T>,
    };

    // This can't be a `&'static WithMetadata<Self>` because Rust will complain that 
    // `T` does not live for the `'static` lifetime.
    const VTABLE:*const WithMetadata<Self>={
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::TMP0)
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


### Example 3:module

This declares,initializes,and uses a module.

```
use abi_stable::{
    StableAbi,
    std_types::RDuration,
    prefix_type::PrefixTypeTrait,
};


#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="PersonMod")))]
pub struct PersonModVal {

    // The getter for this field is infallible,defined (approximately) like this:
    // ```
    //  extern fn customer_for(&self)->extern fn(Id)->RDuration {
    //      self.customer_for
    //  }
    // ```
    #[sabi(last_prefix_field)]
    pub customer_for: extern fn(Id)->RDuration,

    // The default behavior for the getter is to return an Option<FieldType>,
    // if the field exists it returns Some(_),
    // otherwise it returns None.
    pub bike_count: extern fn(Id)->u32,

    // The getter for this field panics if the field doesn't exist.
    #[sabi(missing_field(panic))]
    pub visits: extern fn(Id)->u32,

    // The getter for this field returns `default_score()` if the field doesn't exist.
    #[sabi(missing_field(with="default_score"))]
    pub score: extern fn(Id)->u32,
    
    // The getter for this field returns `Default::default()` if the field doesn't exist.
    #[sabi(missing_field(default))]
    pub visit_length: Option< extern fn(Id)->RDuration >,

}

fn default_score()-> extern fn(Id)->u32 {
    extern fn default(_:Id)->u32{
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
#   extern fn customer_for(id:Id)->RDuration{
#       VARS[id as usize].0
#   }
# 
#   extern fn bike_count(id:Id)->u32{
#       VARS[id as usize].1
#   }
#   extern fn visits(id:Id)->u32{
#       VARS[id as usize].1
#   }
#   extern fn score(id:Id)->u32{
#       VARS[id as usize].1
#   }
#

/*
    ...
    Elided function definitions
    ...
*/

# fn main(){

let module:&'static PersonMod=
    PersonModVal{
        customer_for,
        bike_count,
        visits,
        score,
        visit_length:None,
    }.leak_into_prefix();


// Getting the value for every field of `module`.

let customer_for: extern fn(Id)->RDuration = 
    module.customer_for();

let bike_count: Option<extern fn(Id)->u32> = 
    module.bike_count();

let visits: extern fn(Id)->u32=
    module.visits();

let score: extern fn(Id)->u32=
    module.score();

let visit_length: Option<extern fn(Id)->RDuration> =
    module.visit_length();

# }


```

*/
