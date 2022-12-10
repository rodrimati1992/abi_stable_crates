/*!

Prefix-types are types that derive StableAbi along with the
`#[sabi(kind(Prefix(....)))]` helper attribute.
This is mostly intended for **vtables** and **modules**.

Prefix-types cannot directly be passed through ffi,
instead they must be converted to the type declared with `prefix_ref= Foo_Ref`,
and then pass that instead.

To convert `Foo` to `Foo_Ref` you can use any of (non-exhaustive list):

- `PrefixTypeTrait::leak_into_prefix`:<br>
    Which does the conversion directly,but leaks the value.

- `prefix_type::WithMetadata::new`:<br>
    Use this if you need a compiletime constant.<br>
    First create a `StaticRef<WithMetadata<Self>>` constant using
    the [`staticref`] macro,
    then construct a `Foo_Ref` constant with `Foo_Ref(THE_STATICREF_CONSTANT.as_prefix())`.<br>
    There are two examples of this,
    [for modules](#module_construction),and [for vtables](#vtable_construction)


All the fields in the `DerivingType` can be accessed in `DerivingType_Ref` using
accessor methods named the same as the fields.

# Version compatibility

### Adding fields

To ensure that libraries stay abi compatible,
the first minor version of the library must use the `#[sabi(last_prefix_field)]` attribute on some
field, and every minor version after that must add fields at the end (never moving that attribute).
Changing the field that `#[sabi(last_prefix_field)]` is applied to is a breaking change.

Getter methods for fields after the one to which `#[sabi(last_prefix_field)]` was applied to
will return `Option<FieldType>` by default,because those fields might not exist
(the struct might come from a previous version of the library).
To override how to deal with nonexistent fields,
use the `#[sabi(missing_field())]` attribute,
applied to either the struct or the field.

### Alignment

To ensure that users can define empty vtables/modules that can be extended in
semver compatible versions,
this library forces the struct converted to ffi-safe form to have an alignment at
least that of usize.

You must ensure that newer versions don't change the alignment of the struct,
because that makes it ABI incompatible.

# Grammar Reference

For the grammar reference,you can look at the documentation for
[`#[derive(StableAbi)]`](../../derive.StableAbi.html).

# Examples

###  Example 1

Declaring a Prefix-type.

```

use abi_stable::{
    std_types::{RDuration, RStr},
    StableAbi,
};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = Module_Ref)))]
#[sabi(missing_field(panic))]
pub struct Module {
    pub lib_name: RStr<'static>,

    #[sabi(last_prefix_field)]
    pub elapsed: extern "C" fn() -> RDuration,

    pub description: RStr<'static>,
}

# fn main(){}


```

In this example:

- `#[sabi(kind(Prefix(prefix_ref= Module_Ref)))]` declares this type as being a prefix-type
    with an ffi-safe pointer called `Module_Ref` to which `Module` can be converted into.

- `#[sabi(missing_field(panic))]`
    makes the field accessors panic when attempting to
    access nonexistent fields instead of the default of returning an `Option<FieldType>`.

- `#[sabi(last_prefix_field)]`means that it is the last field in the struct
    that was defined in the first compatible version of the library
    (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    requiring new fields to always be added below preexisting ones.

<span id="module_construction"></span>
### Constructing a module

This example demonstrates how you can construct a module.

For constructing a vtable, you can look at [the next example](#vtable_construction)

```

use abi_stable::{
    extern_fn_panic_handling,
    prefix_type::{PrefixTypeTrait, WithMetadata},
    staticref,
    std_types::{RDuration, RStr},
    StableAbi,
};

fn main() {
    assert_eq!(MODULE_REF.lib_name().as_str(), "foo");

    assert_eq!(MODULE_REF.elapsed()(1000), RDuration::from_secs(1));

    assert_eq!(MODULE_REF.description().as_str(), "this is a module field");
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = Module_Ref)))]
#[sabi(missing_field(panic))]
pub struct Module<T> {
    pub lib_name: RStr<'static>,

    #[sabi(last_prefix_field)]
    pub elapsed: extern "C" fn(T) -> RDuration,

    pub description: RStr<'static>,
}

impl Module<u64> {
    // This macro declares a `StaticRef<WithMetadata<Module<u64>>>` constant.
    staticref!(const MODULE_VAL: WithMetadata<Module<u64>> = WithMetadata::new(
        Module{
            lib_name: RStr::from_str("foo"),
            elapsed,
            description: RStr::from_str("this is a module field"),
        },
    ));
}

const MODULE_REF: Module_Ref<u64> = Module_Ref(Module::MODULE_VAL.as_prefix());

extern "C" fn elapsed(milliseconds: u64) -> RDuration {
    extern_fn_panic_handling! {
        RDuration::from_millis(milliseconds)
    }
}

```

<span id="vtable_construction"></span>
### Constructing a vtable

This example demonstrates how you can construct a vtable.

```rust
use abi_stable::{
    extern_fn_panic_handling,
    marker_type::ErasedObject,
    prefix_type::{PrefixTypeTrait, WithMetadata},
    staticref, StableAbi,
};

fn main() {
    unsafe {
        let vtable = MakeVTable::<u64>::MAKE;
        assert_eq!(
            vtable.get_number()(&3u64 as *const u64 as *const ErasedObject),
            12,
        );
    }
    unsafe {
        let vtable = MakeVTable::<u8>::MAKE;
        assert_eq!(
            vtable.get_number()(&128u8 as *const u8 as *const ErasedObject),
            512,
        );
    }
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = VTable_Ref)))]
#[sabi(missing_field(panic))]
pub struct VTable {
    #[sabi(last_prefix_field)]
    pub get_number: unsafe extern "C" fn(*const ErasedObject) -> u64,
}

// A dummy struct, used purely for its associated constants.
struct MakeVTable<T>(T);

impl<T> MakeVTable<T>
where
    T: Copy + Into<u64>,
{
    unsafe extern "C" fn get_number(this: *const ErasedObject) -> u64 {
        extern_fn_panic_handling! {
            (*this.cast::<T>()).into() * 4
        }
    }

    // This macro declares a `StaticRef<WithMetadata<VTable>>` constant.
    staticref! {pub const VAL: WithMetadata<VTable> = WithMetadata::new(
        VTable{get_number: Self::get_number},
    )}

    pub const MAKE: VTable_Ref = VTable_Ref(Self::VAL.as_prefix());
}

```


<span id="example2"></span>
###  Example 2:Declaring a type with a VTable

Here is the implementation of a Box-like type,which uses a vtable that is a prefix type.

```

use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use abi_stable::{
    extern_fn_panic_handling,
    pointer_trait::{CallReferentDrop, TransmuteElement},
    prefix_type::{PrefixTypeTrait, WithMetadata},
    staticref, StableAbi,
};

/// An ffi-safe `Box<T>`
#[repr(C)]
#[derive(StableAbi)]
pub struct BoxLike<T> {
    data: *mut T,

    vtable: BoxVtable_Ref<T>,

    _marker: PhantomData<T>,
}

impl<T> BoxLike<T> {
    pub fn new(value: T) -> Self {
        let box_ = Box::new(value);

        Self {
            data: Box::into_raw(box_),
            vtable: BoxVtable::VTABLE,
            _marker: PhantomData,
        }
    }

    fn vtable(&self) -> BoxVtable_Ref<T> {
        self.vtable
    }

    /// Extracts the value this owns.
    pub fn into_inner(self) -> T {
        let this = ManuallyDrop::new(self);
        let vtable = this.vtable();
        unsafe {
            // Must copy this before calling `vtable.destructor()`
            // because otherwise it would be reading from a dangling pointer.
            let ret = this.data.read();
            vtable.destructor()(this.data, CallReferentDrop::No);
            ret
        }
    }
}

impl<T> Deref for BoxLike<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &(*self.data) }
    }
}

impl<T> DerefMut for BoxLike<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut (*self.data) }
    }
}

impl<T> Drop for BoxLike<T> {
    fn drop(&mut self) {
        let vtable = self.vtable();

        unsafe { vtable.destructor()(self.data, CallReferentDrop::Yes) }
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
impl<T> BoxVtable<T> {
    // This macro declares a `StaticRef<WithMetadata<BoxVtable<T>>>` constant.
    //
    // StaticRef represents a reference to data that lives forever,
    // but is not necessarily `'static` according to the type system,
    // eg: `BoxVtable<T>`.
    staticref!(const VTABLE_VAL: WithMetadata<Self> = WithMetadata::new(
        Self{
            destructor:destroy_box::<T>,
        },
    ));

    const VTABLE: BoxVtable_Ref<T> =
        { BoxVtable_Ref(Self::VTABLE_VAL.as_prefix()) };
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
    prefix_type::{PrefixTypeTrait, WithMetadata},
    sabi_extern_fn,
    std_types::RDuration,
    StableAbi,
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
    /// requiring new fields to always be added below preexisting ones.
    ///
    /// The `#[sabi(last_prefix_field)]` attribute would stay on this field until the library
    /// bumps its "major" version,
    /// at which point it would be moved to the last field at the time.
    ///
    #[sabi(last_prefix_field)]
    pub customer_for: extern "C" fn(Id) -> RDuration,

    // The default behavior for the getter is to return an Option<FieldType>,
    // if the field exists it returns Some(_),
    // otherwise it returns None.
    pub bike_count: extern "C" fn(Id) -> u32,

    // The getter for this field panics if the field doesn't exist.
    #[sabi(missing_field(panic))]
    pub visits: extern "C" fn(Id) -> u32,

    // The getter for this field returns `default_score()` if the field doesn't exist.
    #[sabi(missing_field(with = default_score))]
    pub score: extern "C" fn(Id) -> u32,

    // The getter for this field returns `Default::default()` if the field doesn't exist.
    #[sabi(missing_field(default))]
    pub visit_length: Option<extern "C" fn(Id) -> RDuration>,
}

fn default_score() -> extern "C" fn(Id) -> u32 {
    extern "C" fn default(_: Id) -> u32 {
        1000
    }

    default
}

type Id = u32;

#   static VARS:&[(RDuration,u32)]=&[
#       (RDuration::new(1_000,0),10),
#       (RDuration::new(1_000_000,0),1),
#   ];

#   #[sabi_extern_fn]
#   fn customer_for(id:Id)->RDuration{
#       VARS[id as usize].0
#   }

#   #[sabi_extern_fn]
#   fn bike_count(id:Id)->u32{
#       VARS[id as usize].1
#   }

#   #[sabi_extern_fn]
#   fn visits(id:Id)->u32{
#       VARS[id as usize].1
#   }

#   #[sabi_extern_fn]
#   fn score(id:Id)->u32{
#       VARS[id as usize].1
#   }

/*
    ...
    Elided function definitions
    ...
*/

# fn main(){

const _MODULE_WM_: &WithMetadata<PersonMod> = &WithMetadata::new(
    PersonMod {
        customer_for,
        bike_count,
        visits,
        score,
        visit_length: None,
    },
);

const MODULE: PersonMod_Ref = PersonMod_Ref(_MODULE_WM_.static_as_prefix());

// Getting the value for every field of `MODULE`.

let customer_for: extern "C" fn(Id) -> RDuration = MODULE.customer_for();

let bike_count: Option<extern "C" fn(Id) -> u32> = MODULE.bike_count();

let visits: extern "C" fn(Id) -> u32 = MODULE.visits();

let score: extern "C" fn(Id) -> u32 = MODULE.score();

let visit_length: Option<extern "C" fn(Id) -> RDuration> = MODULE.visit_length();

# }


```









[`staticref`]: ../../macro.staticref.html



*/
