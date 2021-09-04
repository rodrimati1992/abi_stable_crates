/*!
Shared docs for inherent items of `sabi_trait` trait objects.

`<trait>` here is a generic way to refer to the name of "a trait that's
annotated with the [`sabi_trait`] attribute macro".

[The trait used in method examples
](../../sabi_trait/doc_examples/trait.Action.html):
```rust
#[abi_stable::sabi_trait]
pub trait Action: Debug {
    /// Gets the current value of `self`.
    fn get(&self) -> usize;
    
    /// Adds `val` into `self`, returning the new value.
    fn add_mut(&mut self, val: usize) -> usize;

    /// Adds `val` into `self`, returning the new value.
    #[sabi(last_prefix_field)]
    fn add_into(self, val: usize) -> usize;
}
# fn main(){}
```


# Methods

These are the common methods for the `<trait>_TO` ffi-safe trait object type
generated by the [`sabi_trait`] attribute.

-[`from_ptr`](#from_ptr-method)

-[`from_value`](#from_value-method)

-[`from_const`](#from_const-method)

-[`from_sabi`](#from_sabi-method)

-[`sabi_reborrow_mut`](#sabi_reborrow_mut-method)

-[`sabi_reborrow`](#sabi_reborrow-method)




## `from_ptr` method

```text
impl<'lt, ErasedPtr, …> Trait_TO<'lt, ErasedPtr, …> {
    pub fn from_ptr<Ptr, Downcasting>(
        pointer: Ptr,
        can_it_downcast: Downcasting
    ) -> Self 
```

Constructs `<trait>_TO` from a pointer to a type that implements `<trait>`

The `can_it_downcast` parameter describes whether the trait object can be 
converted back into the original type or not.<br>
Its possible values are [`TD_CanDowncast`] and [`TD_Opaque`].

[Method docs for `Action_TO::from_ptr`
](../../sabi_trait/doc_examples/struct.Action_TO.html#method.from_ptr)

**Example**:
```rust
use abi_stable::{
    sabi_trait::doc_examples::Action_TO,
    std_types::{RArc, RBox},
    type_level::downcasting::TD_CanDowncast,
    RRef, RMut,
};

// From an RBox
{
    // The type annotation is purely for the reader.
    let mut object: Action_TO<'static, RBox<()>> =
        Action_TO::from_ptr(RBox::new(10_usize), TD_CanDowncast);

    assert_eq!(object.get(), 10);

    assert_eq!(object.add_mut(3), 13);
    assert_eq!(object.get(), 13);

    // consumes `object`, now it can't be used anymore.
    assert_eq!(object.add_into(7), 20);
}

// From a reference
{
    // `Action_TO`s constructed from `&` are `Action_TO<'_, RRef<'_, ()>>`
    // since `&T` can't soundly be transmuted back and forth into `&()`    
    let object: Action_TO<'static, RRef<'static, ()>> =
        Action_TO::from_ptr(&20_usize, TD_CanDowncast);

    assert_eq!(object.get(), 20);
}

// From a mutable reference
{
    let mut val = 30_usize;

    // `Action_TO`s constructed from `&mut` are `Action_TO<'_, RMut<'_, ()>>`
    // since `&mut T` can't soundly be transmuted back and forth into `&mut ()`    
    let mut object: Action_TO<'static, RMut<'_, ()>> =
        Action_TO::from_ptr(&mut val, TD_CanDowncast);

    assert_eq!(object.get(), 30);

    assert_eq!(object.add_mut(3), 33);
    assert_eq!(object.get(), 33);

    drop(object);

    assert_eq!(val, 33);
}

// From an RArc
{
    let object: Action_TO<'static, RArc<()>> =
        Action_TO::from_ptr(RArc::new(40), TD_CanDowncast);

    assert_eq!(object.get(), 40);
}

```

## `from_value` method

```text
impl<'lt, …> Trait_TO<'lt, RBox<()>, …> {
    pub fn from_value<_OrigPtr, Downcasting>(
        pointer: _OrigPtr,
        can_it_downcast: Downcasting
    ) -> Self 
```

Constructs `<trait>_TO` from a type that implements `<trait>`,
wrapping that value in an [`RBox`].

The `can_it_downcast` parameter describes whether the trait object can be 
converted back into the original type or not.<br>
Its possible values are [`TD_CanDowncast`] and [`TD_Opaque`].

[Method docs for `Action_TO::from_value`
](../../sabi_trait/doc_examples/struct.Action_TO.html#method.from_value)

**Example**:
```rust
use abi_stable::{
    sabi_trait::doc_examples::Action_TO,
    std_types::RBox,
    type_level::downcasting::TD_CanDowncast,
};

// The type annotation is purely for the reader.
let mut object: Action_TO<'static, RBox<()>> =
    Action_TO::from_value(100_usize, TD_CanDowncast);

assert_eq!(object.get(), 100);

assert_eq!(object.add_mut(3), 103);
assert_eq!(object.get(), 103);

// consumes `object`, now it can't be used anymore.
assert_eq!(object.add_into(7), 110);

```


## `from_const` method

```text
impl<'lt, 'sub, …> Trait_TO<'lt, RRef<'sub, ()>, …> {
    pub const fn from_const<T, Downcasting>(
        pointer: &'sub T,
        can_it_downcast: Downcasting,
        vtable_for: …,
    ) -> Self 
```

Const-constructs `<trait>_TO` from a constant that
implements `<trait>`,

The `can_it_downcast` parameter describes whether the trait object can be 
converted back into the original type or not.<br>
Its possible values are [`TD_CanDowncast`] and [`TD_Opaque`].

You can construct the `vtable_for` parameter with `<trait>_MV:VTABLE`

[Method docs for `Action_TO::from_const`
](../../sabi_trait/doc_examples/struct.Action_TO.html#method.from_const)

**Example:**
```rust
use abi_stable::{
    sabi_trait::doc_examples::Action_trait::{Action_CTO, Action_TO, Action_MV},
    std_types::RBox,
    type_level::downcasting::TD_CanDowncast,
};

const TO: Action_CTO<'_, '_> = 
    Action_TO::from_const(
        &200,
        TD_CanDowncast,
        Action_MV::VTABLE,
    );

assert_eq!(TO.get(), 200);

```


## `from_sabi` method

```text
impl<'lt, ErasedPtr, …> Trait_TO<'lt, ErasedPtr, …> {
    pub fn from_sabi(obj: Trait_Backend<'lt, ErasedPtr, …>) -> Self 
```

Constructs `<trait>_TO` from the backend trait object type,
either [`RObject`] or [`DynTrait`].

[Method docs for `Action_TO::from_sabi`
](../../sabi_trait/doc_examples/struct.Action_TO.html#method.from_sabi)

This allows calling methods on the `.obj` field of `<trait>_TO`
that consume the backend and return it back.

For [`Action_TO`] specifically, [`RObject`] is the backend.

**Example:**
```rust
use abi_stable::{
    pointer_trait::{CanTransmuteElement, OwnedPointer},
    sabi_trait::{
        doc_examples::Action_trait::{Action_Interface, Action_TO, Action_MV},
        RObject,
    },
    std_types::RBox,
    type_level::downcasting::TD_CanDowncast,
};

let mut object: Action_TO<'static, RBox<()>> = 
    Action_TO::from_value(700, TD_CanDowncast);


object = try_unerase::<_, u32>(object).unwrap_err();

object = try_unerase::<_, String>(object).unwrap_err();

assert_eq!(*try_unerase::<_, usize>(object).unwrap(), 700);



fn try_unerase<P, T>(
    object: Action_TO<'static, P>,
) -> Result<P::TransmutedPtr, Action_TO<'static, P>>
where
    T: 'static,
    // This bound is required to call `into_unerased` on the `obj: RObject<…>` field
    P: OwnedPointer<PtrTarget = ()> + CanTransmuteElement<T>,
{
    object.obj
        .into_unerased()
        .map_err(|e| Action_TO::from_sabi(e.into_inner()))
}

```

## `sabi_reborrow` method

```text
impl<'lt, ErasedPtr, …> Trait_TO<'lt, ErasedPtr, …> {
    pub fn sabi_reborrow<'r>(&'r self) -> Trait_TO<'lt, RRef<'r, ()>, …> {
```

Reborrows a `&'r <trait>_TO<'lt, …>` into a `<trait>_TO<'lt, RRef<'r, ()>, …>`.

[Method docs for `Action_TO::sabi_reborrow`
](../../sabi_trait/doc_examples/struct.Action_TO.html#method.sabi_reborrow)

This allows passing the trait object to functions that take a
`<trait>_TO<'b, RRef<'a, ()>, …>`,
and functions generic over the traits that `<trait>_TO` implements.

This method is only available for traits that either:
- require neither Send nor Sync,
- require `Send + Sync`.

**Example**:
```rust
use abi_stable::{
    sabi_trait::doc_examples::Action_TO,
    std_types::RBox,
    type_level::downcasting::TD_CanDowncast,
    RRef,
};

let mut object: Action_TO<'static, RBox<()>> =
    Action_TO::from_value(300_usize, TD_CanDowncast);

assert_eq!(to_debug_string(object.sabi_reborrow()), "300");

assert_eq!(object.add_mut(7), 307);
assert_eq!(get_usize(object.sabi_reborrow()), 307);

assert_eq!(object.add_mut(14), 321);
// last use of `object`, so we can move it into the function
assert_eq!(to_debug_string(object), "321");


fn to_debug_string<T>(x: T) -> String
where
    T: std::fmt::Debug
{
    format!("{:?}", x)
}

fn get_usize(x: Action_TO<'_, RRef<'_, ()>>) -> usize {
    x.get()
}
```

## `sabi_reborrow_mut` method

```text
impl<'lt, ErasedPtr, …> Trait_TO<'lt, ErasedPtr, …> {
    pub fn sabi_reborrow_mut<'r>(&'r mut self) -> Trait_TO<'lt, RMut<'r, ()>, …> {
```

Reborrows a `&'r mut <trait>_TO<'b, …>` into a `<trait>_TO<'b, RMut<'r, ()>, …>`.

[Method docs for `Action_TO::sabi_reborrow_mut`
](../../sabi_trait/doc_examples/struct.Action_TO.html#method.sabi_reborrow_mut)

This allows passing the trait object to functions that take a
`<trait>_TO<'b, RMut<'a, ()>, …>`,
and functions generic over the traits that `<trait>_TO` implements.

This method is only available for traits that either:
- require neither Send nor Sync,
- require `Send + Sync`.

**Example**:
```rust
use abi_stable::{
    pointer_trait::AsMutPtr,
    sabi_trait::doc_examples::Action_TO,
    std_types::RBox,
    type_level::downcasting::TD_CanDowncast,
};

let mut object: Action_TO<'static, RBox<()>> =
    Action_TO::from_value(400_usize, TD_CanDowncast);

assert_eq!(add_mut(object.sabi_reborrow_mut(), 6), 406);

assert_eq!(add_mut(object.sabi_reborrow_mut(), 10), 416);

// last use of `object`, so we can move it into the function
assert_eq!(add_mut(object, 20), 436);


fn add_mut<P>(mut x: Action_TO<'_, P>, how_much: usize) -> usize 
where
    // Needed for calling mutable methods on `Action_TO`
    P: AsMutPtr<PtrTarget = ()>
{
    x.add_mut(how_much)
}

```















[`sabi_trait`]: ../../attr.sabi_trait.html

[`RObject`]: ../../sabi_trait/struct.RObject.html

[`DynTrait`]: ../struct.DynTrait.html

[`RBox`]: ../std_types/struct.RObject.html

[`Action_TO`]: ../../sabi_trait/doc_examples/struct.Action_TO.html

[`TD_CanDowncast`]: ../../type_level/downcasting/struct.TD_CanDowncast.html

[`TD_Opaque`]: ../../type_level/downcasting/struct.TD_Opaque.html

*/