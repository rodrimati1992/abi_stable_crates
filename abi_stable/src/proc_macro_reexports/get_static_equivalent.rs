/**

The `GetStaticEquivalent` macro derives the [`GetStaticEquivalent_`] trait.

Implementing [`GetStaticEquivalent_`] allows the type to be passed as a 
type argument of a type deriving `StableAbi`,
that used the 
[`#[sabi(not_stableabi(TypeParameter))]`](derive@crate::StableAbi#sabinot_stableabitypeparameter)
helper attribute.

# Container Attributes

These helper attributes are applied on the type declaration.

<span id = "impl_InterfaceType"></span>
### `#[sabi(impl_InterfaceType(...))]`

Implements the `InterfaceType` trait for a type,
defining the usable/required traits when creating a 
`DynTrait<_, ThisType>`/`NonExhaustive<_, _, ThisType>`.

Syntax: `#[sabi(impl_InterfaceType(Trait0, Trait1, ..., TraitN))]`

If a trait is not specified,
it will not be required when constructing DynTrait/NonExhaustive,
and won't be usable afterwards.

<a href = "./derive.StableAbi.html#InterfaceType_traits">
    The list of valid traits is here 
</a>

# Examples

###  Using an associated constant 

This example demonstrates how one can have a type parameter,
and use the value of an associated constant as the identity of the type.

*/
#[cfg_attr(not(feature = "no_fn_promotion"), doc = "```rust")]
#[cfg_attr(feature = "no_fn_promotion", doc = "```ignore")]
/**
use std::marker::PhantomData;

use abi_stable::{
    abi_stability::check_layout_compatibility, marker_type::UnsafeIgnoredType, tag,
    GetStaticEquivalent, StableAbi,
};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    not_stableabi(T),
    bound(T: WithName),
    tag = tag!( <T as WithName>::NAME )
)]
struct WithMarker<T>(UnsafeIgnoredType<T>);

impl<T> WithMarker<T> {
    const NEW: Self = WithMarker(UnsafeIgnoredType::NEW);
}

trait WithName {
    const NAME: &'static str;
}

#[derive(GetStaticEquivalent)]
struct Mark;
impl WithName for Mark {
    const NAME: &'static str = "Mark";
}

#[derive(GetStaticEquivalent)]
struct John;
impl WithName for John {
    const NAME: &'static str = "John";
}

#[derive(GetStaticEquivalent)]
struct JessiJames;
impl WithName for JessiJames {
    const NAME: &'static str = "JessiJames";
}

# fn main(){

// This checks that the two types aren't considered compatible.
assert!(check_layout_compatibility(
    <WithMarker<Mark> as StableAbi>::LAYOUT,
    <WithMarker<John> as StableAbi>::LAYOUT,
)
.is_err());

// This checks that the two types aren't considered compatible.
assert!(check_layout_compatibility(
    <WithMarker<John> as StableAbi>::LAYOUT,
    <WithMarker<JessiJames> as StableAbi>::LAYOUT,
)
.is_err());

// This checks that the two types aren't considered compatible.
assert!(check_layout_compatibility(
    <WithMarker<JessiJames> as StableAbi>::LAYOUT,
    <WithMarker<Mark> as StableAbi>::LAYOUT,
)
.is_err());

# }


```

###  Using an associated type 

This example demonstrates how one can have a type parameter,
and use its associated type as a field.

```rust
use abi_stable::{std_types::RVec, StableAbi};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(not_stableabi(I), bound(<I as IntoIterator>::Item : StableAbi))]
pub struct CollectedIterator<I>
where
    I: IntoIterator,
{
    vec: RVec<I::Item>,
}


```


[`GetStaticEquivalent_`]: abi_stable::abi_stability::get_static_equivalent::GetStaticEquivalent_

*/

#[doc(inline)]
pub use abi_stable_derive::GetStaticEquivalent;