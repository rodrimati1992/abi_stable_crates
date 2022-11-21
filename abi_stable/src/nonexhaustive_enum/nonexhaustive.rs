//! Contains `NonExhaustive<>` and related items.

use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::Deref,
};

use crate::{
    abi_stability::StableAbi,
    erased_types::{c_functions, trait_objects::HasherObject, InterfaceType, MakeRequiredTraits},
    inline_storage::ScratchSpace,
    marker_type::ErasedObject,
    nonexhaustive_enum::{
        assert_correct_storage, vtable::NonExhaustiveVtable_Ref, AssertCsArgs, DeserializeEnum,
        EnumInfo, GetEnumInfo, GetVTable, NonExhaustiveMarker, SerializeEnum, ValidDiscriminant,
    },
    pointer_trait::{CanTransmuteElement, TransmuteElement},
    sabi_types::{RMut, RRef},
    std_types::RBoxError,
    traits::IntoReprRust,
    type_level::{impl_enum::Implemented, trait_marker},
};

use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};

// #[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod tests;

/// A generic type for all ffi-safe non-exhaustive enums.
///
/// This type allows adding variants to enums it wraps in ABI compatible versions of a library.
///
/// # Generic parameters
///
/// ###  `E`
///
/// This is the enum that this was constructed from,
/// and can be unwrapped back into if it's one of the valid variants in this context.
///
/// ###  `S`
///
/// The storage type,used to store the enum opaquely.
///
/// This has to be at least the size and alignment of the wrapped enum.
///
/// This is necessary because:
///
/// - The compiler assumes that an enum cannot be a variant outside the ones it sees.
///
/// - To give some flexibility to grow the enum in semver compatible versions of a library.
///
/// ###  `I`
///
/// The interface of the enum(it implements [`InterfaceType`](crate::InterfaceType)),
/// determining which traits are required when constructing `NonExhaustive<>`
/// and which are available afterwards.
///
/// # Examples
///
/// ### Error type
///
/// Say that we define an error type for a library.
///
///
/// Version 1.0.
/// ```
/// use abi_stable::{
///     nonexhaustive_enum::{NonExhaustive, NonExhaustiveFor},
///     sabi_trait,
///     std_types::RString,
///     StableAbi,
/// };
///
/// #[repr(u8)]
/// #[derive(StableAbi, Debug, Clone, PartialEq)]
/// #[sabi(kind(WithNonExhaustive(
///     size = [usize;8],
///     traits(Debug, Clone, PartialEq),
/// )))]
/// #[non_exhaustive]
/// pub enum Error {
///     CouldNotFindItem {
///         name: RString,
///     },
///     OutOfStock {
///         id: usize,
///         name: RString,
///     },
/// }
///
/// fn returns_could_not_find_item(name: RString) -> NonExhaustiveFor<Error> {
///     let e = Error::CouldNotFindItem { name };
///     NonExhaustive::new(e)
/// }
///
/// fn returns_out_of_stock(id: usize, name: RString) -> NonExhaustiveFor<Error> {
///     let e = Error::OutOfStock { id, name };
///     NonExhaustive::new(e)
/// }
///
/// ```
///
/// Then in 1.1 we add another error variant,returned only by new library functions.
///
/// ```
/// use abi_stable::{
///     nonexhaustive_enum::{NonExhaustive, NonExhaustiveFor},
///     sabi_trait,
///     std_types::RString,
///     StableAbi,
/// };
///
/// #[repr(u8)]
/// #[derive(StableAbi, Debug, Clone, PartialEq)]
/// #[sabi(kind(WithNonExhaustive(
///     size = [usize;8],
///     traits(Debug, Clone, PartialEq),
/// )))]
/// #[non_exhaustive]
/// pub enum Error {
///     CouldNotFindItem {
///         name: RString,
///     },
///     OutOfStock {
///         id: usize,
///         name: RString,
///     },
///     InvalidItemId {
///         id: usize,
///     },
/// }
///
/// fn returns_invalid_item_id() -> NonExhaustiveFor<Error> {
///     NonExhaustive::new(Error::InvalidItemId { id: 100 })
/// }
///
/// ```
///
/// If a library user attempted to unwrap `Error::InvalidItemId`
/// (using NonExhaustive::as_enum/as_enum_mut/into_enum)
/// with the 1.0 version of `Error` they would get an `Err(..)` back.
///
///
/// ### Static enums
///
/// This example demonstrates putting a nonexhaustive enum in a static.
///
/// ```rust
/// use abi_stable::{
///     nonexhaustive_enum::{NonExhaustive, NonExhaustiveFor},
///     std_types::RString,
///     rstr, StableAbi,
/// };
///
/// static AA: NonExhaustiveFor<Foo> = NonExhaustive::new(Foo::A);
///
/// static BB: NonExhaustiveFor<Foo> = NonExhaustive::new(Foo::B(2));
///
/// let cc = NonExhaustive::new(Foo::C {name: "hello".into()});
///
/// assert_eq!(AA, Foo::A);
/// assert_eq!(BB, Foo::B(2));
/// assert_eq!(cc, Foo::C {name: RString::from("hello")});
///
///
/// #[repr(u8)]
/// #[derive(StableAbi, Debug, PartialEq, Eq)]
/// #[sabi(kind(WithNonExhaustive(
///     size = 64,
///     traits(Debug, PartialEq, Eq)
/// )))]
/// pub enum Foo {
///     A,
///     B(i8),
///     C { name: RString },
/// }
///
/// ```
///
///
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    //debug_print,
    not_stableabi(E,S,I),
    bound(NonExhaustiveVtable_Ref<E,S,I>:StableAbi),
    bound(E: NonExhaustiveMarker<S>),
    bound(I: InterfaceType),
    extra_checks = <I as MakeRequiredTraits>::MAKE,
    phantom_type_param = <E as NonExhaustiveMarker<S>>::Marker,
)]
pub struct NonExhaustive<E, S, I> {
    // This is an opaque field since we only care about its size and alignment
    #[sabi(unsafe_opaque_field)]
    fill: ScratchSpace<E, S>,
    vtable: NonExhaustiveVtable_Ref<E, S, I>,
    _marker: PhantomData<()>,
}

/// The type of a `NonExhaustive` wrapping the enum `E`,
/// using `E`'s  default storage and interface.
pub type NonExhaustiveFor<E> =
    NonExhaustive<E, <E as GetEnumInfo>::DefaultStorage, <E as GetEnumInfo>::DefaultInterface>;

/// The type of a `NonExhaustive<>` wrapping the enum E,
/// using the `E`'s  default storage and a custom interface.
pub type NonExhaustiveWI<E, I> = NonExhaustive<E, <E as GetEnumInfo>::DefaultStorage, I>;

/// The type of a `NonExhaustive<>` wrapping the enum E,
/// using a custom storage and the `E`'s default interface.
pub type NonExhaustiveWS<E, S> = NonExhaustive<E, S, <E as GetEnumInfo>::DefaultInterface>;

impl<E, S, I> NonExhaustive<E, S, I> {
    /// Constructs a `NonExhaustive<>` from `value` using its default interface and storage.
    ///
    /// # Panic
    ///
    /// This panics if the storage has an alignment or size smaller than that of `E`.
    #[inline]
    pub const fn new(value: E) -> Self
    where
        E: GetVTable<S, I> + GetEnumInfo<DefaultStorage = S, DefaultInterface = I>,
    {
        NonExhaustive::with_storage_and_interface(value)
    }

    /// Constructs a `NonExhaustive<>` from `value` using its default storage
    /// and a custom interface.
    ///
    /// # Panic
    ///
    /// This panics if the storage has an alignment or size smaller than that of `E`.
    #[inline]
    pub const fn with_interface(value: E) -> Self
    where
        E: GetVTable<S, I> + GetEnumInfo<DefaultStorage = S>,
    {
        NonExhaustive::with_storage_and_interface(value)
    }

    /// Constructs a `NonExhaustive<>` from `value` using its default interface
    /// and a custom storage.
    ///
    /// # Panic
    ///
    /// This panics if the storage has an alignment or size smaller than that of `E`.
    #[inline]
    pub const fn with_storage(value: E) -> Self
    where
        E: GetVTable<S, I> + GetEnumInfo<DefaultInterface = I>,
    {
        NonExhaustive::with_storage_and_interface(value)
    }

    /// Constructs a `NonExhaustive<>` from `value` using both a custom interface and storage.
    ///
    /// # Panic
    ///
    /// This panics if the storage has an alignment or size smaller than that of `E`.
    #[inline]
    pub const fn with_storage_and_interface(value: E) -> Self
    where
        E: GetVTable<S, I>,
    {
        unsafe { NonExhaustive::with_vtable(value, E::VTABLE) }
    }

    #[track_caller]
    pub(super) const unsafe fn with_vtable(
        value: E,
        vtable: NonExhaustiveVtable_Ref<E, S, I>,
    ) -> Self {
        // `ScratchSpace::new` is what asserts that the enum is
        // the correct size and alignment
        Self {
            fill: ScratchSpace::<E, S>::new(value),
            vtable,
            _marker: PhantomData,
        }
    }
}

impl<E, S, I> NonExhaustive<E, S, I>
where
    E: GetEnumInfo,
{
    /// wraps a reference to this `NonExhaustive<>` into a reference to the original enum.
    ///
    /// # Errors
    ///
    /// This returns an error if the wrapped enum is of a variant that is
    /// not valid in this context.
    ///
    /// # Example
    ///
    /// This shows how some `NonExhaustive<enum>` can be unwrapped, and others cannot.<br>
    /// That enum comes from a newer version of the library than this knows.
    ///
    /// ```
    /// use abi_stable::nonexhaustive_enum::doc_enums::example_2::{
    ///     new_a, new_b, new_c, Foo,
    /// };
    ///
    /// assert_eq!(new_a().as_enum().ok(), Some(&Foo::A));
    /// assert_eq!(new_b(10).as_enum().ok(), Some(&Foo::B(10)));
    /// assert_eq!(new_b(77).as_enum().ok(), Some(&Foo::B(77)));
    /// assert_eq!(new_c().as_enum().ok(), None);
    ///
    ///
    /// ```
    ///
    pub fn as_enum(&self) -> Result<&E, UnwrapEnumError<&Self>> {
        let discriminant = self.get_discriminant();
        if E::is_valid_discriminant(discriminant) {
            unsafe { Ok(&*(&self.fill as *const ScratchSpace<E, S> as *const E)) }
        } else {
            Err(UnwrapEnumError::new(self))
        }
    }

    /// Unwraps a mutable reference to this `NonExhaustive<>` into a
    /// mutable reference to the original enum.
    ///
    /// # Errors
    ///
    /// This returns an error if the wrapped enum is of a variant that is
    /// not valid in this context.
    ///
    /// # Example
    ///
    /// This shows how some `NonExhaustive<enum>` can be unwrapped, and others cannot.<br>
    /// That enum comes from a newer version of the library than this knows.
    ///
    /// ```
    /// use abi_stable::nonexhaustive_enum::doc_enums::example_1::{
    ///     new_a, new_b, new_c, Foo,
    /// };
    ///
    /// assert_eq!(new_a().as_enum_mut().ok(), Some(&mut Foo::A));
    /// assert_eq!(new_b(10).as_enum_mut().ok(), None);
    /// assert_eq!(new_b(77).as_enum_mut().ok(), None);
    /// assert_eq!(new_c().as_enum_mut().ok(), None);
    ///
    /// ```
    pub fn as_enum_mut(&mut self) -> Result<&mut E, UnwrapEnumError<&mut Self>>
    where
        E: GetVTable<S, I>,
    {
        let discriminant = self.get_discriminant();
        if E::is_valid_discriminant(discriminant) {
            // Must update the vtable every time as_enum_mut is called,
            // because if the enum is replaced with a variant with a discriminant
            // outside the valid range for the functions in the vtable,
            // it would be undefined behavior to call those functions.
            self.vtable = E::VTABLE;
            unsafe { Ok(&mut *(&mut self.fill as *mut ScratchSpace<E, S> as *mut E)) }
        } else {
            Err(UnwrapEnumError::new(self))
        }
    }

    /// Unwraps this `NonExhaustive<>` into the original enum.
    ///
    /// # Errors
    ///
    /// This returns an error if the wrapped enum is of a variant that is
    /// not valid in this context.
    ///
    /// # Example
    ///
    /// This shows how some `NonExhaustive<enum>` can be unwrapped, and others cannot.<br>
    /// That enum comes from a newer version of the library than this knows.
    ///
    /// ```
    /// use abi_stable::nonexhaustive_enum::doc_enums::example_2::{
    ///     new_a, new_b, new_c, Foo,
    /// };
    ///
    /// assert_eq!(new_a().into_enum().ok(), Some(Foo::A));
    /// assert_eq!(new_b(10).into_enum().ok(), Some(Foo::B(10)));
    /// assert_eq!(new_b(77).into_enum().ok(), Some(Foo::B(77)));
    /// assert_eq!(new_c().into_enum().ok(), None);
    ///
    /// ```
    pub fn into_enum(self) -> Result<E, UnwrapEnumError<Self>> {
        let discriminant = self.get_discriminant();
        if E::is_valid_discriminant(discriminant) {
            let this = ManuallyDrop::new(self);
            unsafe { Ok((&this.fill as *const ScratchSpace<E, S> as *const E).read()) }
        } else {
            Err(UnwrapEnumError::new(self))
        }
    }

    /// Returns whether the discriminant of this enum is valid in this context.
    ///
    /// The only way for it to be invalid is if the dynamic library is a
    /// newer version than this knows.
    #[inline]
    pub fn is_valid_discriminant(&self) -> bool {
        E::is_valid_discriminant(self.get_discriminant())
    }

    /// Gets the value of the discriminant of the enum.
    #[inline]
    pub const fn get_discriminant(&self) -> E::Discriminant {
        unsafe { *(&self.fill as *const ScratchSpace<E, S> as *const E::Discriminant) }
    }
}

impl<E, S, I> NonExhaustive<E, S, I> {
    /// Transmute this `NonExhaustive<E,S,I>` into `NonExhaustive<F,S,I>`,
    /// changing the type of the enum it wraps.
    ///
    /// # Safety
    ///
    /// This has the same safety requirements that `std::mem::transmute` has.
    ///
    /// # Panics
    ///
    /// This panics if the storage has an alignment or size smaller than that of `F`.
    ///
    ///
    pub const unsafe fn transmute_enum<F>(self) -> NonExhaustive<F, S, I> {
        assert_correct_storage::<F, S>(AssertCsArgs::UNKNOWN);
        unsafe { const_transmute!(NonExhaustive<E, S, I>, NonExhaustive<F, S, I>, self) }
    }

    /// Transmute this `&NonExhaustive<E,S,I>` into `&NonExhaustive<F,S,I>`,
    /// changing the type of the enum it wraps.
    ///
    /// # Safety
    ///
    /// This has the same safety requirements that `std::mem::transmute` has.
    ///
    /// # Panics
    ///
    /// This panics if the storage has an alignment or size smaller than that of `F`.
    pub const unsafe fn transmute_enum_ref<F>(&self) -> &NonExhaustive<F, S, I> {
        assert_correct_storage::<F, S>(AssertCsArgs::UNKNOWN);
        unsafe { &*(self as *const Self as *const _) }
    }

    /// Transmute this `&mut NonExhaustive<E,S,I>` into `&mut NonExhaustive<F,S,I>`,
    /// changing the type of the enum it wraps.
    ///
    /// # Safety
    ///
    /// This has the same safety requirements that `std::mem::transmute` has.
    ///
    /// # Panics
    ///
    /// This panics if the storage has an alignment or size smaller than that of `F`.
    pub unsafe fn transmute_enum_mut<F>(&mut self) -> &mut NonExhaustive<F, S, I> {
        assert_correct_storage::<F, S>(AssertCsArgs::UNKNOWN);
        unsafe { &mut *(self as *mut Self as *mut _) }
    }

    /// Transmute this pointer to a `NonExhaustive<E,S,I>` into
    /// a pointer (of the same kind) to a `NonExhaustive<F,S,I>`,
    /// changing the type of the enum it wraps.
    ///
    /// # Safety
    ///
    /// This has the same safety requirements that
    /// `abi_stable::pointer_traits::TransmuteElement::transmute_element` has.
    ///
    /// # Panics
    ///
    /// This panics if the storage has an alignment or size smaller than that of `F`.
    ///
    pub unsafe fn transmute_enum_ptr<P, F>(this: P) -> P::TransmutedPtr
    where
        P: Deref<Target = Self>,
        P: CanTransmuteElement<NonExhaustive<F, S, I>>,
    {
        assert_correct_storage::<F, S>(AssertCsArgs::UNKNOWN);
        unsafe { this.transmute_element::<NonExhaustive<F, S, I>>() }
    }

    /// Gets a reference to the vtable of this `NonExhaustive<>`.
    pub(crate) const fn vtable(&self) -> NonExhaustiveVtable_Ref<E, S, I> {
        self.vtable
    }

    const fn sabi_erased_ref(&self) -> RRef<'_, ErasedObject> {
        unsafe { RRef::from_raw(&self.fill as *const ScratchSpace<E, S> as *const ErasedObject) }
    }

    const fn as_erased_ref(&self) -> RRef<'_, ErasedObject> {
        unsafe { RRef::from_raw(self as *const Self as *const ErasedObject) }
    }

    fn sabi_erased_mut(&mut self) -> RMut<'_, ErasedObject> {
        unsafe { RMut::from_raw(&mut self.fill as *mut ScratchSpace<E, S> as *mut ErasedObject) }
    }
}

impl<E, S, I> Clone for NonExhaustive<E, S, I>
where
    I: InterfaceType<Clone = Implemented<trait_marker::Clone>>,
{
    fn clone(&self) -> Self {
        unsafe { self.vtable().clone_()(self.sabi_erased_ref(), self.vtable) }
    }
}

impl<E, S, I> Display for NonExhaustive<E, S, I>
where
    I: InterfaceType<Display = Implemented<trait_marker::Display>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            c_functions::adapt_std_fmt::<ErasedObject>(
                self.sabi_erased_ref(),
                self.vtable().display(),
                f,
            )
        }
    }
}

impl<E, S, I> Debug for NonExhaustive<E, S, I>
where
    I: InterfaceType<Debug = Implemented<trait_marker::Debug>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            c_functions::adapt_std_fmt::<ErasedObject>(
                self.sabi_erased_ref(),
                self.vtable().debug(),
                f,
            )
        }
    }
}

impl<E, S, I> Eq for NonExhaustive<E, S, I>
where
    Self: PartialEq,
    I: InterfaceType<Eq = Implemented<trait_marker::Eq>>,
{
}

impl<E, S, I1, I2> PartialEq<NonExhaustive<E, S, I2>> for NonExhaustive<E, S, I1>
where
    I1: InterfaceType<PartialEq = Implemented<trait_marker::PartialEq>>,
{
    fn eq(&self, other: &NonExhaustive<E, S, I2>) -> bool {
        unsafe { self.vtable().partial_eq()(self.sabi_erased_ref(), other.as_erased_ref()) }
    }
}

impl<E, S, I> Ord for NonExhaustive<E, S, I>
where
    I: InterfaceType<Ord = Implemented<trait_marker::Ord>>,
    Self: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { self.vtable().cmp()(self.sabi_erased_ref(), other.as_erased_ref()).into() }
    }
}

impl<E, S, I1, I2> PartialOrd<NonExhaustive<E, S, I2>> for NonExhaustive<E, S, I1>
where
    I1: InterfaceType<PartialOrd = Implemented<trait_marker::PartialOrd>>,
    Self: PartialEq<NonExhaustive<E, S, I2>>,
{
    fn partial_cmp(&self, other: &NonExhaustive<E, S, I2>) -> Option<Ordering> {
        unsafe {
            self.vtable().partial_cmp()(self.sabi_erased_ref(), other.as_erased_ref())
                .map(IntoReprRust::into_rust)
                .into()
        }
    }
}

/////////////////////

impl<E, S, I> PartialOrd<E> for NonExhaustive<E, S, I>
where
    E: GetEnumInfo + PartialOrd,
    I: InterfaceType<PartialOrd = Implemented<trait_marker::PartialOrd>>,
    Self: PartialEq<E>,
{
    fn partial_cmp(&self, other: &E) -> Option<Ordering> {
        match self.as_enum() {
            Ok(this) => this.partial_cmp(other),
            Err(_) => Some(Ordering::Greater),
        }
    }
}

impl<E, S, I> PartialEq<E> for NonExhaustive<E, S, I>
where
    E: GetEnumInfo + PartialEq,
    I: InterfaceType<PartialEq = Implemented<trait_marker::PartialEq>>,
{
    fn eq(&self, other: &E) -> bool {
        match self.as_enum() {
            Ok(this) => this == other,
            Err(_) => false,
        }
    }
}

/////////////////////

impl<E, S, I> NonExhaustive<E, S, I>
where
    E: GetEnumInfo,
{
    /// It serializes a `NonExhaustive<_>` into a proxy.
    pub fn serialize_into_proxy(&self) -> Result<I::Proxy, RBoxError>
    where
        I: InterfaceType<Serialize = Implemented<trait_marker::Serialize>>,
        I: SerializeEnum<E>,
    {
        unsafe { self.vtable().serialize()(self.as_erased_ref()).into_result() }
    }

    /// Deserializes a `NonExhaustive<_>` from a proxy.
    pub fn deserialize_from_proxy<'borr>(proxy: I::Proxy) -> Result<Self, RBoxError>
    where
        I: InterfaceType<Deserialize = Implemented<trait_marker::Deserialize>>,
        I: DeserializeEnum<'borr, Self>,
        I::Proxy: 'borr,
    {
        I::deserialize_enum(proxy)
    }
}

/// First it serializes a `NonExhaustive<_>` into a proxy,then it serializes that proxy.
impl<E, S, I> Serialize for NonExhaustive<E, S, I>
where
    I: InterfaceType<Serialize = Implemented<trait_marker::Serialize>>,
    I: SerializeEnum<E>,
    I::Proxy: Serialize,
{
    fn serialize<Z>(&self, serializer: Z) -> Result<Z::Ok, Z::Error>
    where
        Z: Serializer,
    {
        unsafe {
            self.vtable().serialize()(self.as_erased_ref())
                .into_result()
                .map_err(ser::Error::custom)?
                .serialize(serializer)
        }
    }
}

/// First it Deserializes a string,then it deserializes into a
/// `NonExhaustive<_>`,by using `<I as DeserializeEnum>::deserialize_enum`.
impl<'de, E, S, I> Deserialize<'de> for NonExhaustive<E, S, I>
where
    E: 'de + GetVTable<S, I>,
    S: 'de,
    I: 'de + InterfaceType<Deserialize = Implemented<trait_marker::Deserialize>>,
    I: DeserializeEnum<'de, Self>,
    <I as DeserializeEnum<'de, Self>>::Proxy: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s =
            <<I as DeserializeEnum<'de, Self>>::Proxy as Deserialize>::deserialize(deserializer)?;

        I::deserialize_enum(s).map_err(de::Error::custom)
    }
}

/////////////////////

impl<E, S, I> Hash for NonExhaustive<E, S, I>
where
    I: InterfaceType<Hash = Implemented<trait_marker::Hash>>,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        unsafe { self.vtable().hash()(self.sabi_erased_ref(), HasherObject::new(state)) }
    }
}

impl<E, S, I> std::error::Error for NonExhaustive<E, S, I> where
    I: InterfaceType<
        Debug = Implemented<trait_marker::Debug>,
        Display = Implemented<trait_marker::Display>,
        Error = Implemented<trait_marker::Error>,
    >
{
}

/////////////////////

impl<E, S, I> Drop for NonExhaustive<E, S, I> {
    fn drop(&mut self) {
        let drop = self.vtable()._sabi_drop();

        unsafe {
            drop(self.sabi_erased_mut());
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

/// Used to abstract over the reference-ness of [`NonExhaustive`] inside [`UnwrapEnumError`].
pub trait NonExhaustiveSharedOps {
    /// The type of the discriminant of the wrapped enum.
    type Discriminant: ValidDiscriminant;

    /// Gets the discriminant of the wrapped enum.
    fn get_discriminant_(&self) -> Self::Discriminant;

    /// Gets miscelaneous information about the wrapped enum
    fn enum_info_(&self) -> &'static EnumInfo;
}

/// A struct storing the discriminant and `EnumInfo` of some enum.
struct DiscrAndEnumInfo<E> {
    discr: E,
    enum_info: &'static EnumInfo,
}

impl<E> NonExhaustiveSharedOps for DiscrAndEnumInfo<E>
where
    E: ValidDiscriminant,
{
    type Discriminant = E;
    fn get_discriminant_(&self) -> E {
        self.discr
    }
    fn enum_info_(&self) -> &'static EnumInfo {
        self.enum_info
    }
}

macro_rules! impl_neso {
    (
        impl[$E:ident,$S:ident,$I:ident]
    ) => {
        type Discriminant = $E::Discriminant;

        fn get_discriminant_(&self) -> $E::Discriminant {
            self.get_discriminant()
        }

        fn enum_info_(&self) -> &'static EnumInfo {
            self.vtable().enum_info()
        }
    };
}

impl<E, S, I> NonExhaustiveSharedOps for NonExhaustive<E, S, I>
where
    E: GetEnumInfo,
{
    impl_neso! { impl[E,S,I] }
}

impl<'a, E, S, I> NonExhaustiveSharedOps for &'a NonExhaustive<E, S, I>
where
    E: GetEnumInfo,
{
    impl_neso! { impl[E,S,I] }
}

impl<'a, E, S, I> NonExhaustiveSharedOps for &'a mut NonExhaustive<E, S, I>
where
    E: GetEnumInfo,
{
    impl_neso! { impl[E,S,I] }
}

///////////////////////////////////////////////////////////////////////////////

/// An error for a situation where a `NonExhaustive<>` could not be unwrapped into the enum
/// because the discriminant wasn't valid in this context
/// (likely because it is from a newer version of the library).
#[must_use]
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StableAbi)]
#[non_exhaustive]
pub struct UnwrapEnumError<N> {
    /// This field is either a `NonExhaustive<>` or a `DiscrAndEnumInfo<>`
    pub non_exhaustive: N,
}

#[allow(clippy::missing_const_for_fn)]
impl<N> UnwrapEnumError<N> {
    /// Gets the `non_exhaustive` field.
    #[must_use]
    pub fn into_inner(self) -> N {
        self.non_exhaustive
    }

    /// Converts this into a boxed error.
    pub fn into_boxed(self) -> RBoxError
    where
        N: NonExhaustiveSharedOps,
    {
        let x = DiscrAndEnumInfo {
            discr: self.non_exhaustive.get_discriminant_(),
            enum_info: self.non_exhaustive.enum_info_(),
        };
        let x = UnwrapEnumError::new(x);
        RBoxError::new(x)
    }
}

impl<N> UnwrapEnumError<N> {
    #[inline]
    const fn new(non_exhaustive: N) -> Self {
        Self { non_exhaustive }
    }
}

impl<N> Display for UnwrapEnumError<N>
where
    N: NonExhaustiveSharedOps,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Could not unwrap NonExhaustive into '{}'.\n\
             Because its discriminant was {:?} .",
            self.non_exhaustive.enum_info_().type_name(),
            self.non_exhaustive.get_discriminant_(),
        )
    }
}

impl<N> Debug for UnwrapEnumError<N>
where
    N: NonExhaustiveSharedOps,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnwrapEnumError")
            .field("non_exhaustive", &"<opaque>")
            .field("discriminant", &self.non_exhaustive.get_discriminant_())
            .field("enum_info", &self.non_exhaustive.enum_info_())
            .finish()
    }
}

impl<N> From<UnwrapEnumError<N>> for RBoxError
where
    N: NonExhaustiveSharedOps,
{
    fn from(uee: UnwrapEnumError<N>) -> RBoxError {
        uee.into_boxed()
    }
}

impl<N> std::error::Error for UnwrapEnumError<N> where N: NonExhaustiveSharedOps {}
