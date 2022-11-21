//! The traits releated to nonexhaustive enums.

use std::{
    cmp::{Eq, Ord},
    fmt::{self, Debug},
};

use crate::{
    std_types::{RBoxError, RStr},
    type_layout::StartLen,
    type_level::{
        impl_enum::{Implemented, Unimplemented},
        trait_marker,
    },
    InterfaceType,
};

/// Queries the marker type which describes the layout of this enum,
/// for use in [`NonExhaustive`]'s [`StableAbi`] impl.
///
/// # Safety
///
/// `Self::Marker` must describe the layout of this enum,
/// with the size and alignment of `Storage`,
/// and using [`IsExhaustive::nonexhaustive`] to construct [`IsExhaustive`] in
/// the `enum`'s [`TypeLayout`].
///
/// [`StableAbi`]: trait@crate::StableAbi
/// [`TypeLayout`]: crate::type_layout::TypeLayout
/// [`IsExhaustive`]: crate::type_layout::IsExhaustive
/// [`IsExhaustive::nonexhaustive`]: crate::type_layout::IsExhaustive::nonexhaustive
/// [`NonExhaustive`]: crate::nonexhaustive_enum::NonExhaustive
///
pub unsafe trait NonExhaustiveMarker<Storage>: GetEnumInfo {
    /// A marker type which describes the layout of this enum
    /// in its [`StableAbi`] impl.
    type Marker;
}

/// Describes the discriminant of an enum,and its valid values.
///
/// # Safety
///
/// This must be an enum with a `#[repr(C)]` or `#[repr(SomeInteFgerType)]` attribute.
///
/// The `Discriminant` associated type must correspond to the type of
/// this enum's discriminant.
///
/// The `DISCRIMINANTS` associated constant must be the values of
/// this enum's discriminants.
pub unsafe trait GetEnumInfo: Sized {
    /// The type of the discriminant.
    type Discriminant: ValidDiscriminant;

    /// The default storage type,
    /// used to store this enum inside `NonExhaustive<>`,
    /// and allow the enum to grow in size in newer ABI compatible versions.
    type DefaultStorage;

    /// The default InterfaceType,
    /// used to determine the traits that are required when constructing a `NonExhaustive<>`,
    /// and are then usable afterwards.
    type DefaultInterface;

    /// Information about the enum.
    const ENUM_INFO: &'static EnumInfo;

    /// The values of the discriminants of each variant.
    ///
    const DISCRIMINANTS: &'static [Self::Discriminant];

    /// Whether `discriminant` is one of the valid discriminants for this enum in this context.
    fn is_valid_discriminant(discriminant: Self::Discriminant) -> bool;
}

pub use self::_enum_info::EnumInfo;
mod _enum_info {
    use super::*;

    /// Contains miscelaneous information about an enum.
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct EnumInfo {
        /// The name of a type,eg:`Vec` for a `Vec<u8>`.
        type_name: RStr<'static>,

        strings: RStr<'static>,

        /// The range inside of strings with the names of the variants of the enum,separated by ';'.
        variant_names_start_len: StartLen,
    }

    impl EnumInfo {
        #[doc(hidden)]
        pub const fn _for_derive(
            type_name: RStr<'static>,
            strings: RStr<'static>,
            variant_names_start_len: StartLen,
        ) -> Self {
            Self {
                type_name,
                strings,
                variant_names_start_len,
            }
        }

        /// The name of a type,eg:`Foo` for a `Foo<u8>`.
        pub fn type_name(&self) -> &'static str {
            self.type_name.as_str()
        }

        /// The names of the variants of the enum,separated by ';'.
        pub fn variant_names(&self) -> &'static str {
            &self.strings.as_str()[self.variant_names_start_len.to_range()]
        }
    }
}

impl EnumInfo {
    /// Gets an iterator over the names of the variants of the enum.
    pub fn variant_names_iter(
        &self,
    ) -> impl Iterator<Item = &'static str> + 'static + Debug + Clone {
        self.variant_names().split_terminator(';')
    }
}

impl Debug for EnumInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnumInfo")
            .field("type_name", &self.type_name())
            .field("variant_names", &IteratorAsList(self.variant_names_iter()))
            .finish()
    }
}

struct IteratorAsList<I>(I);

impl<I, T> Debug for IteratorAsList<I>
where
    I: Iterator<Item = T> + Clone,
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.clone()).finish()
    }
}

/////////////////////////////////////////////////////////////

/// Marker trait for types that abi_stable supports as enum discriminants.
///
/// This trait cannot be implemented outside of this module.
pub trait ValidDiscriminant: Sealed + Copy + Eq + Ord + Debug + Send + Sync + 'static {}

mod sealed {
    pub trait Sealed {}
}
use self::sealed::Sealed;

macro_rules! impl_valid_discriminant {
    (
        $($ty:ty),* $(,)*
    ) => (
        $(
            impl ValidDiscriminant for $ty{}
            impl Sealed for $ty{}
        )*
    )
}

impl_valid_discriminant! {u8,i8,u16,i16,u32,i32,u64,i64,usize,isize}

///////////////////////////////////////////////////////////////////////////////

/// Describes how some enum is serialized.
///
/// This is generally implemented by the interface of an enum
/// (`Enum_Interface` for `Enum`),which also implements [`InterfaceType`]).
///
/// # Example
///
/// ```rust
/// use abi_stable::{
///     external_types::RawValueBox,
///     nonexhaustive_enum::{NonExhaustive, SerializeEnum},
///     std_types::{RBoxError, RString},
///     StableAbi,
/// };
///
/// let ne = NonExhaustive::new(Foo::C{name: "world".into()});
/// assert_eq!(serde_json::to_string(&ne).unwrap(), r#"{"C":{"name":"world"}}"#);
///
///
/// #[repr(u8)]
/// #[derive(StableAbi, Debug, PartialEq, Eq, serde::Serialize)]
/// #[sabi(kind(WithNonExhaustive(
///     size = 64,
///     traits(Debug, PartialEq, Eq, Serialize)
/// )))]
/// pub enum Foo {
///     A,
///     B(i8),
///     C {
///         name: RString
///     },
/// }
///
/// impl SerializeEnum<Foo> for Foo_Interface {
///     /// A type that `Foo` is converted into to be serialized.
///     type Proxy = RawValueBox;
///
///     fn serialize_enum(this: &Foo) -> Result<RawValueBox, RBoxError> {
///         match serde_json::value::to_raw_value(&this) {
///             Ok(v) => Ok(v.into()),
///             Err(e) => Err(RBoxError::new(e)),
///         }
///     }
/// }
/// ```
///
/// [`InterfaceType`]: ../trait.InterfaceType.html
pub trait SerializeEnum<Enum>: InterfaceType {
    /// The intermediate type the `Enum` is converted into,to serialize it.
    type Proxy;

    /// Serializes an enum into its proxy type.
    fn serialize_enum(this: &Enum) -> Result<Self::Proxy, RBoxError>;
}

#[doc(hidden)]
pub trait GetSerializeEnumProxy<E>: InterfaceType {
    type ProxyType;
}

impl<I, E, PT> GetSerializeEnumProxy<E> for I
where
    I: InterfaceType,
    I: GetSerializeEnumProxyHelper<E, <I as InterfaceType>::Serialize, ProxyType = PT>,
{
    type ProxyType = PT;
}

#[doc(hidden)]
pub trait GetSerializeEnumProxyHelper<E, IS>: InterfaceType {
    type ProxyType;
}

impl<I, E> GetSerializeEnumProxyHelper<E, Implemented<trait_marker::Serialize>> for I
where
    I: InterfaceType,
    I: SerializeEnum<E>,
{
    type ProxyType = <I as SerializeEnum<E>>::Proxy;
}

impl<I, E> GetSerializeEnumProxyHelper<E, Unimplemented<trait_marker::Serialize>> for I
where
    I: InterfaceType,
{
    type ProxyType = ();
}

///////////////////////////////////////

/// Describes how a nonexhaustive enum is deserialized.
///
/// Generally this delegates to a library function,
/// so that the implementation can be delegated
/// to the `implementation crate`.
///
/// This is generally implemented by the interface of an enum
/// (`Enum_Interface` for `Enum`),which also implements [`InterfaceType`]).
///
/// The `NE` type parameter is expected to be [`NonExhaustive`].
///
/// # Example
///
/// ```rust
/// use abi_stable::{
///     nonexhaustive_enum::{DeserializeEnum, NonExhaustive, NonExhaustiveFor},
///     external_types::RawValueRef,
///     std_types::{RBoxError, RResult, ROk, RErr, RStr, RString},
///     rstr, StableAbi,
/// };
///
/// let input = r#"{"C": {"name": "hello"}}"#;
/// let ne = serde_json::from_str::<NonExhaustiveFor<Foo>>(input).unwrap();
/// assert_eq!(ne, Foo::C{name: "hello".into()});
///
///
/// #[repr(u8)]
/// #[derive(StableAbi, Debug, PartialEq, Eq, serde::Deserialize)]
/// #[sabi(kind(WithNonExhaustive(
///     size = 64,
///     traits(Debug, PartialEq, Eq, Deserialize)
/// )))]
/// pub enum Foo {
///     A,
///     B(i8),
///     C {
///         name: RString
///     },
/// }
///
/// impl<'borr> DeserializeEnum<'borr, NonExhaustiveFor<Foo>> for Foo_Interface {
///     /// The intermediate type the `NE` is converted from,to deserialize it.
///     type Proxy = RawValueRef<'borr>;
///
///     /// Deserializes an enum from its proxy type.
///     fn deserialize_enum(s: Self::Proxy) -> Result<NonExhaustiveFor<Foo>, RBoxError> {
///         deserialize_foo(s.get_rstr()).into_result()
///     }
/// }
///
/// /////////////
/// // everything below could be defined in an implementation crate
/// //
/// // This allows the library that defines the enum to add variants,
/// // and deserialize the variants that it added,
/// // regardless of whether the dependent crates know about those variants.
///
/// extern "C" fn deserialize_foo(s: RStr<'_>) -> RResult<NonExhaustiveFor<Foo>, RBoxError> {
///     abi_stable::extern_fn_panic_handling!{
///         match serde_json::from_str::<Foo>(s.into()) {
///             Ok(x) => ROk(NonExhaustive::new(x)),
///             Err(e) => RErr(RBoxError::new(e)),
///         }
///     }
/// }
///
/// ```
///
/// [`InterfaceType`]: crate::InterfaceType
/// [`NonExhaustive`]: crate::nonexhaustive_enum::NonExhaustive
pub trait DeserializeEnum<'borr, NE>: InterfaceType {
    /// The intermediate type the `NonExhaustive` is converted from,to deserialize it.
    type Proxy;

    /// Deserializes an enum from its proxy type.
    fn deserialize_enum(s: Self::Proxy) -> Result<NE, RBoxError>;
}
