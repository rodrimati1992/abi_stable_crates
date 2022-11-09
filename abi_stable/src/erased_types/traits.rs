//! Traits for types wrapped in `DynTrait<_>`

use crate::{
    marker_type::NonOwningPhantom,
    sabi_types::{Constructor, VersionStrings},
    std_types::{RBoxError, RStr},
};

use super::TypeInfo;

#[allow(unused_imports)]
use crate::type_level::{
    bools::{False, True},
    impl_enum::{Implemented, Unimplemented},
    trait_marker,
};

/// An `implementation type`,
/// with [an associated `InterfaceType`](ImplType::Interface)
/// which describes the traits that
/// must be implemented when constructing a [`DynTrait`] from `Self`,
/// using the [`DynTrait::from_value`] and [`DynTrait::from_ptr`] constructors,
/// so as to pass an opaque type across ffi.
///
/// To initialize `INFO` you can use the [`impl_get_type_info`] macro.
///
/// # Uniqueness
///
/// Users of this trait can't enforce that they are the only ones with the same interface,
/// therefore they should handle the `Err(..)`s returned
/// from the `DynTrait::*downcast*` functions whenever
/// they convert back and forth between `Self` and [`Self::Interface`](#associatedtype.Interface).
///
///
/// [`DynTrait`]: crate::DynTrait
/// [`DynTrait::from_value`]: crate::DynTrait::from_value
/// [`DynTrait::from_ptr`]: crate::DynTrait::from_ptr
/// [`impl_get_type_info`]: crate::impl_get_type_info
pub trait ImplType: Sized {
    /// Describes the traits that must be implemented when constructing a
    /// `DynTrait` from `Self`.
    type Interface: InterfaceType;

    /// Information about the type for debugging purposes.
    ///
    /// You can use the `impl_get_type_info` macro to initialize this.
    const INFO: &'static TypeInfo;
}

macro_rules! declare_InterfaceType {
    (

        $(#[$attrs:meta])*

        assoc_types[
            $(
                $(#[$assoc_attrs:meta])*
                type $trait_:ident ;
            )*
        ]
    ) => (
        $(#[$attrs])*
        pub trait InterfaceType: Sized {
            $(
                $(#[$assoc_attrs])*
                type $trait_;
            )*

            #[doc(hidden)]
            type define_this_in_the_impl_InterfaceType_macro;
        }


    )
}

declare_InterfaceType! {
    /// Defines the usable/required traits when creating a
    /// [`DynTrait<Pointer<()>, ThisInterfaceType>`](crate::DynTrait).
    ///
    /// This trait can only be implemented using the
    /// [`#[derive(StableAbi)]`](derive@crate::StableAbi)
    /// derive with the
    /// [`#[sabi(impl_InterfaceType(...))]`](derive@crate::StableAbi#sabiimpl_interfacetype)
    /// helper attribute,
    /// defaulting associated types to `Unimplemented<_>`.
    ///
    /// The value of every associated type can be:
    ///
    /// - [`Implemented<_>`](crate::type_level::impl_enum::Implemented):
    /// the trait would be required by, and be usable in `DynTrait`.
    ///
    /// - [`Unimplemented<_>`](crate::type_level::impl_enum::Unimplemented):
    /// the trait would not be required by, and not be usable in `DynTrait`.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// use abi_stable::{erased_types::InterfaceType, type_level::bools::*, StableAbi};
    ///
    /// #[repr(C)]
    /// #[derive(StableAbi)]
    /// #[sabi(impl_InterfaceType(Clone, Debug))]
    /// pub struct FooInterface;
    ///
    /// /*
    /// The `#[sabi(impl_InterfaceType(Clone, Debug))]` helper attribute
    /// (as part of #[derive(StableAbi)]) above is roughly equivalent to this impl:
    ///
    /// impl InterfaceType for FooInterface {
    ///     type Clone = Implemented<trait_marker::Clone>;
    ///
    ///     type Debug = Implemented<trait_marker::Debug>;
    ///
    ///     /////////////////////////////////////
    ///     //// defaulted associated types
    ///     /////////////////////////////////////
    ///
    ///     // Changing this to require/unrequire in minor versions, is an abi breaking change.
    ///     // type Send = Unimplemented<trait_marker::Send>;
    ///
    ///     // Changing this to require/unrequire in minor versions, is an abi breaking change.
    ///     // type Sync = Unimplemented<trait_marker::Sync>;
    ///
    ///     // Changing this to require/unrequire in minor versions, is an abi breaking change.
    ///     // type Unpin = Unimplemented<trait_marker::Unpin>;
    ///
    ///     // type Iterator = Unimplemented<trait_marker::Iterator>;
    ///
    ///     // type DoubleEndedIterator = Unimplemented<trait_marker::DoubleEndedIterator>;
    ///
    ///     // type Default = Unimplemented<trait_marker::Default>;
    ///
    ///     // type Display = Unimplemented<trait_marker::Display>;
    ///
    ///     // type Serialize = Unimplemented<trait_marker::Serialize>;
    ///
    ///     // type Eq = Unimplemented<trait_marker::Eq>;
    ///
    ///     // type PartialEq = Unimplemented<trait_marker::PartialEq>;
    ///
    ///     // type Ord = Unimplemented<trait_marker::Ord>;
    ///
    ///     // type PartialOrd = Unimplemented<trait_marker::PartialOrd>;
    ///
    ///     // type Hash = Unimplemented<trait_marker::Hash>;
    ///
    ///     // type Deserialize = Unimplemented<trait_marker::Deserialize>;
    ///
    ///     // type FmtWrite = Unimplemented<trait_marker::FmtWrite>;
    ///
    ///     // type IoWrite = Unimplemented<trait_marker::IoWrite>;
    ///
    ///     // type IoSeek = Unimplemented<trait_marker::IoSeek>;
    ///
    ///     // type IoRead = Unimplemented<trait_marker::IoRead>;
    ///
    ///     // type IoBufRead = Unimplemented<trait_marker::IoBufRead>;
    ///
    ///     // type Error = Unimplemented<trait_marker::Error>;
    /// }
    /// */
    ///
    /// # fn main(){}
    ///
    ///
    /// ```
    ///
    ///
    ///
    ///
    assoc_types[
        /// Changing this to require/unrequire in minor versions, is an abi breaking change.
        type Send;

        /// Changing this to require/unrequire in minor versions, is an abi breaking change.
        type Sync;

        /// Changing this to require/unrequire in minor versions, is an abi breaking change.
        type Unpin;

        ///
        type Clone;

        ///
        type Default;

        ///
        type Display;

        ///
        type Debug;

        ///
        type Serialize;

        ///
        type Eq;

        ///
        type PartialEq;

        ///
        type Ord;

        ///
        type PartialOrd;

        ///
        type Hash;

        ///
        type Deserialize;

        ///
        type Iterator;

        ///
        type DoubleEndedIterator;

        /// For the `std::fmt::Write` trait
        type FmtWrite;

        /// For the `std::io::Write` trait
        type IoWrite;

        /// For the `std::io::Seek` trait
        type IoSeek;

        /// For the `std::io::Read` trait
        type IoRead;

        /// For the `std::io::BufRead` trait
        type IoBufRead;

        /// For the `std::error::Error` trait
        type Error;
    ]


}

///////////////////////////////////////////////////////////////////////////////

/// Describes how a type is serialized by [`DynTrait`].
///
/// [`DynTrait`]: ../struct.DynTrait.html
pub trait SerializeImplType<'s> {
    /// An [`InterfaceType`] implementor which determines the
    /// intermediate type through which this is serialized.
    ///
    /// [`InterfaceType`]: ./trait.InterfaceType.html
    type Interface: SerializeProxyType<'s>;

    /// Performs the serialization into the proxy.
    fn serialize_impl(
        &'s self,
    ) -> Result<<Self::Interface as SerializeProxyType<'s>>::Proxy, RBoxError>;
}

/// Determines the intermediate type a [`SerializeImplType`] implementor is converted into,
/// and is then serialized.
///
/// [`SerializeImplType`]: ./trait.SerializeImplType.html
pub trait SerializeProxyType<'borr>: InterfaceType {
    /// The intermediate type.
    type Proxy: 'borr;
}

#[doc(hidden)]
pub trait GetSerializeProxyType<'borr>: InterfaceType {
    type ProxyType;
}

impl<'borr, I, PT> GetSerializeProxyType<'borr> for I
where
    I: InterfaceType,
    I: GetSerializeProxyTypeHelper<'borr, <I as InterfaceType>::Serialize, ProxyType = PT>,
{
    type ProxyType = PT;
}

#[doc(hidden)]
pub trait GetSerializeProxyTypeHelper<'borr, IS>: InterfaceType {
    type ProxyType;
}

impl<'borr, I> GetSerializeProxyTypeHelper<'borr, Implemented<trait_marker::Serialize>> for I
where
    I: SerializeProxyType<'borr>,
{
    type ProxyType = <I as SerializeProxyType<'borr>>::Proxy;
}

impl<'borr, I> GetSerializeProxyTypeHelper<'borr, Unimplemented<trait_marker::Serialize>> for I
where
    I: InterfaceType,
{
    type ProxyType = ();
}

///////////////////////////////////////

/// Describes how `D` is deserialized, using a proxy to do so.
///
/// Generally this delegates to a library function,
/// so that the implementation can be delegated
/// to the `implementation crate`.
///
pub trait DeserializeDyn<'borr, D> {
    /// The type that is deserialized and then converted into `D`,
    /// with `DeserializeDyn::deserialize_dyn`.
    type Proxy;

    /// Converts the proxy type into `D`.
    fn deserialize_dyn(s: Self::Proxy) -> Result<D, RBoxError>;
}

#[doc(hidden)]
pub trait GetDeserializeDynProxy<'borr, D>: InterfaceType {
    type ProxyType;
}

impl<'borr, I, D, PT> GetDeserializeDynProxy<'borr, D> for I
where
    I: InterfaceType,
    I: GetDeserializeDynProxyHelper<'borr, D, <I as InterfaceType>::Deserialize, ProxyType = PT>,
{
    type ProxyType = PT;
}

#[doc(hidden)]
pub trait GetDeserializeDynProxyHelper<'borr, D, IS>: InterfaceType {
    type ProxyType;
}

impl<'borr, I, D> GetDeserializeDynProxyHelper<'borr, D, Implemented<trait_marker::Deserialize>>
    for I
where
    I: InterfaceType,
    I: DeserializeDyn<'borr, D>,
{
    type ProxyType = <I as DeserializeDyn<'borr, D>>::Proxy;
}

impl<'borr, I, D> GetDeserializeDynProxyHelper<'borr, D, Unimplemented<trait_marker::Deserialize>>
    for I
where
    I: InterfaceType,
{
    type ProxyType = ();
}

/////////////////////////////////////////////////////////////////////

/// The way to specify the expected `Iterator::Item` type for an `InterfaceType`.
///
/// This is a separate trait to allow iterators that yield borrowed elements.
pub trait IteratorItem<'a>: InterfaceType {
    /// The iterator item type.
    type Item;
}

/// Gets the expected `Iterator::Item` type for an `InterfaceType`,
/// defaulting to `()` if it doesn't require `Iterator` to be implemented.
///
/// Used by `DynTrait`'s vtable to give its iterator methods a defaulted return type.
pub trait IteratorItemOrDefault<'borr>: InterfaceType {
    /// The iterator item type.
    type Item;
}

impl<'borr, I, Item> IteratorItemOrDefault<'borr> for I
where
    I: InterfaceType,
    I: IteratorItemOrDefaultHelper<'borr, <I as InterfaceType>::Iterator, Item = Item>,
{
    type Item = Item;
}

#[doc(hidden)]
pub trait IteratorItemOrDefaultHelper<'borr, ImplIsRequired> {
    type Item;
}

impl<'borr, I, Item> IteratorItemOrDefaultHelper<'borr, Implemented<trait_marker::Iterator>> for I
where
    I: IteratorItem<'borr, Item = Item>,
{
    type Item = Item;
}

impl<'borr, I> IteratorItemOrDefaultHelper<'borr, Unimplemented<trait_marker::Iterator>> for I {
    type Item = ();
}

//////////////////////////////////////////////////////////////////

pub use self::interface_for::InterfaceFor;

#[doc(hidden)]
pub mod interface_for {
    use super::*;

    use crate::type_level::downcasting::GetUTID;

    /// Helper struct to get an `ImplType` implementation for any type.
    pub struct InterfaceFor<T, Interface, Downcasting>(
        NonOwningPhantom<(T, Interface, Downcasting)>,
    );

    impl<T, Interface, Downcasting> ImplType for InterfaceFor<T, Interface, Downcasting>
    where
        Interface: InterfaceType,
        Downcasting: GetUTID<T>,
    {
        type Interface = Interface;

        /// The `&'static TypeInfo` constant, used when unerasing `DynTrait`s into a type.
        const INFO: &'static TypeInfo = &TypeInfo {
            size: std::mem::size_of::<T>(),
            alignment: std::mem::align_of::<T>(),
            _uid: Constructor(<Downcasting as GetUTID<T>>::UID),
            type_name: Constructor(crate::utils::get_type_name::<T>),
            module: RStr::from_str("<unavailable>"),
            package: RStr::from_str("<unavailable>"),
            package_version: VersionStrings::new("99.99.99"),
            _private_field: (),
        };
    }
}

/////////////////////////////////////////////////////////////////////

crate::impl_InterfaceType! {
    impl crate::erased_types::InterfaceType for () {
        type Send= True;
        type Sync= True;
    }
}
