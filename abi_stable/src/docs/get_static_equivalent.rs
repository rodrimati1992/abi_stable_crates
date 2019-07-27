/*!

The `GetStaticEquivalent` derive macro allows implementing the `GetStaticEquivalent_` trait,
allowing the type to be passed as a type parameter of a type deriving `StableAbi`,
that used the `#[sabi(not_stableabi(TypeParameter))]` attribute.

Be aware that if you use `#[sabi(not_stableabi(TypeParameter))]`,
the type parameter won't be compared for compatibility,
because it's not part of any field (type parameters are not compared).

# Container Attributes

These attributes are applied on the type declaration.

<h3 id="impl_InterfaceType"> `#[sabi(impl_InterfaceType(...))]`  </h3>

Implements the `InterfaceType` trait for a type,
defining the usable/required traits when creating a 
`DynTrait<_,ThisType>`/`NonExhaustive<_,_,ThisType>`.

Syntax:`#[sabi(impl_InterfaceType(Trait0,Trait1,...,TraitN))]`

If a trait is not specified,
it will not be required when constructing DynTrait/NonExhaustive,
and won't be usable afterwards.

<a href="../stable_abi_derive/index.html#InterfaceType_traits">
    The list of valid traits is here 
</a>

# Examples

<h3> Using an associated constant </h3>

This example demonstrates how one can have a type parameter,
and use the value of an associated constant as the identity of the type.

```
use std::marker::PhantomData;

use abi_stable::{
    abi_stability::check_layout_compatibility,
    marker_type::UnsafeIgnoredType,
    StableAbi,
    GetStaticEquivalent,
    tag,
};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    not_stableabi(T),
    bound="T:WithName",
    tag="tag!( <T as WithName>::NAME )",
)]
struct WithMarker<T>(UnsafeIgnoredType<T>);

impl<T> WithMarker<T>{
    const NEW:Self=WithMarker(UnsafeIgnoredType::NEW);
}


trait WithName{
    const NAME:&'static str;
}

#[derive(GetStaticEquivalent)]
struct Mark;
impl WithName for Mark{
    const NAME:&'static str="Mark";
}


#[derive(GetStaticEquivalent)]
struct John;
impl WithName for John{
    const NAME:&'static str="John";
}


#[derive(GetStaticEquivalent)]
struct JessiJames;
impl WithName for JessiJames{
    const NAME:&'static str="JessiJames";
}


# fn main(){

// This checks that the two types aren't considered compatible.
assert!(
    check_layout_compatibility(
        <WithMarker<Mark> as StableAbi>::ABI_INFO,
        <WithMarker<John> as StableAbi>::ABI_INFO,
    ).is_err()
);

// This checks that the two types aren't considered compatible.
assert!(
    check_layout_compatibility(
        <WithMarker<John> as StableAbi>::ABI_INFO,
        <WithMarker<JessiJames> as StableAbi>::ABI_INFO,
    ).is_err()
);

// This checks that the two types aren't considered compatible.
assert!(
    check_layout_compatibility(
        <WithMarker<JessiJames> as StableAbi>::ABI_INFO,
        <WithMarker<Mark> as StableAbi>::ABI_INFO,
    ).is_err()
);

# }


```

<h3> Using an associated type </h3>

This example demonstrates how one can have a type parameter,
and use its associated type as a field.

```rust
use abi_stable::{
    std_types::RVec,
    StableAbi,
};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    not_stableabi(I),
    bound="<I as IntoIterator>::Item : StableAbi",
)]
pub struct CollectedIterator<I>
where
    I:IntoIterator
{
    vec:RVec<I::Item>,
}


```


*/