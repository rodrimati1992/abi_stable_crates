//! Types,traits,and functions used by prefix-types.

use std::marker::PhantomData;

use crate::{
    inline_storage::alignment::AlignToUsize, marker_type::NotCopyNotClone,
    pointer_trait::ImmutableRef, sabi_types::StaticRef, utils::Transmuter,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use repr_offset::offset_calc::next_field_offset;

mod accessible_fields;
mod layout;
mod prefix_ref;
mod pt_metadata;

#[cfg(test)]
mod tests;

pub use self::{
    accessible_fields::{FieldAccessibility, FieldConditionality, IsAccessible, IsConditional},
    layout::PTStructLayout,
    prefix_ref::PrefixRef,
};

#[doc(hidden)]
pub use self::pt_metadata::__PrefixTypeMetadata;

/// For types deriving `StableAbi` with
/// [`#[sabi(kind(Prefix(..)))]`](derive@crate::StableAbi#sabi_kind_prefix_attr).
///
/// # Safety
///
/// This trait must be implemented by the `StableAbi` derive macro.
pub unsafe trait PrefixTypeTrait: Sized {
    /// Describes the layout of the struct,exclusively for use in error messages.
    const PT_LAYOUT: &'static PTStructLayout;

    /// A bit array,where each nth bit represents whether the nth field is accessible.
    const PT_FIELD_ACCESSIBILITY: FieldAccessibility;

    /// Converts `Self` to `Self::PrefixRef`,leaking it in the process.
    ///
    /// # Warning
    ///
    /// You must be careful when calling this function,
    /// since this leak is ignored by [miri](https://github.com/rust-lang/miri) .
    ///
    fn leak_into_prefix(self) -> Self::PrefixRef {
        let x = WithMetadata::new(self);
        let x = StaticRef::leak_value(x);
        let x = PrefixRef::from_staticref(x);
        <Self::PrefixRef as PrefixRefTrait>::from_prefix_ref(x)
    }

    /// A struct that contains all the fields up to the field annotated with
    /// `#[sabi(last_prefix_field)]` inclusive.
    ///
    /// Those structs are usually named with a `_Prefix` suffix.
    type PrefixFields;

    /// A pointer to `Self::PrefixFields`,
    /// generally wraps a `PrefixRef<Self::PrefixFields>`.
    ///
    /// Those pointer types are usually named with a `_Ref` suffix.
    type PrefixRef: PrefixRefTrait<
        PtrTarget = WithMetadata_<Self::PrefixFields, Self::PrefixFields>,
        PrefixFields = Self::PrefixFields,
    >;
}

////////////////////////////////////////////////////////////////////////////////

/// Marker trait for pointers to prefix field structs.
///
/// Generally prefix field structs are named with a `_Prefix` suffix,
/// and have all the fields of some other struct up to the
/// one with a `#[sabi(last_prefix_field)]` attribute.
///
/// # Safety
///
/// `Self` must either be `PrefixRef<Self::PrefixFields>`,
/// or a `#[repr(transparent)]` wrapper around one.
pub unsafe trait PrefixRefTrait:
    Sized + ImmutableRef<PtrTarget = WithMetadata_<Self::PrefixFields, Self::PrefixFields>>
{
    /// A struct that contains all the fields of some other struct
    /// up to the field annotated with
    /// `#[sabi(last_prefix_field)]` inclusive.
    ///
    /// Those structs are usually named with a `_Prefix` suffix.
    // The `GetWithMetadata<ForSelf = Self::Target>` bound
    // is a hacky way to encode this type equality bound:
    // `Self::Target == WithMetadata_<Self::PrefixFields, Self::PrefixFields>`
    // (except that the compiler doesn't unify both types)
    type PrefixFields;

    /// Converts a `PrefixRef` to `Self`
    #[inline]
    fn from_prefix_ref(this: PrefixRef<Self::PrefixFields>) -> Self {
        unsafe { Transmuter { from: this }.to }
    }

    /// Converts `Self` to a `PrefixRef`
    #[inline]
    fn to_prefix_ref(self) -> PrefixRef<Self::PrefixFields> {
        unsafe { Transmuter { from: self }.to }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Alias for [`WithMetadata_`]
/// that defaults to passing `<T as PrefixTypeTrait>::PrefixFields`
/// as the second type parameter.
///
/// [`WithMetadata_`] can't have that defaulted type parameter,
/// because `T: `[`PrefixTypeTrait`] is an overly restrictive bound in some cases.
///
///
/// [`WithMetadata_`]: ./struct.WithMetadata_.html
pub type WithMetadata<T, P = <T as PrefixTypeTrait>::PrefixFields> = WithMetadata_<T, P>;

/// Wraps a type along with its prefix-type-related metadata,
/// so that it can be converted to its prefix.
///
/// # Example
///
/// This example demonstrates how you can construct a `WithMetadata` and
/// convert it to a prefix type pointer (`Module_Ref` in this case).
///
/// You can look at the [`PrefixRef` docs](./struct.PrefixRef.html#example) for
/// a more detailed example.
///
/// ```rust
/// use abi_stable::{
///     for_examples::{Module, Module_Ref},
///     prefix_type::{PrefixRef, PrefixTypeTrait, WithMetadata},
///     std_types::{RSome, RStr},
///     staticref,
/// };
///
/// const WITH_META: &WithMetadata<Module> = &WithMetadata::new(
///     Module {
///         first: RSome(66),
///         second: RStr::from_str("lore"),
///         third: 333,
///     },
/// );
///
/// const MOD: Module_Ref = Module_Ref(WITH_META.static_as_prefix());
///
/// assert_eq!(MOD.first(), RSome(66));
/// assert_eq!(MOD.second().as_str(), "lore");
///
/// ```
///
#[repr(C)]
pub struct WithMetadata_<T, P> {
    // __VALUE_OFFSET must be updated if fields are added before `value`
    field_accessibility: FieldAccessibility,
    type_layout: &'static PTStructLayout,

    /// The wrapped value.
    pub value: AlignToUsize<T>,
    unbounds: NotCopyNotClone,
    _prefix: PhantomData<P>,
}

#[doc(hidden)]
impl<T, P> WithMetadata_<T, P> {
    // The offset of the `value` field
    pub const __VALUE_OFFSET: usize = {
        let tl_offset = next_field_offset::<Self, FieldAccessibility, &'static PTStructLayout>(0);

        next_field_offset::<Self, &'static PTStructLayout, AlignToUsize<T>>(tl_offset)
    };
}

impl<T, P> WithMetadata_<T, P> {
    /// Constructs this with `WithMetadata::new(value)`
    #[inline]
    pub const fn new(value: T) -> Self
    where
        T: PrefixTypeTrait<PrefixFields = P>,
    {
        Self {
            field_accessibility: T::PT_FIELD_ACCESSIBILITY,
            type_layout: T::PT_LAYOUT,
            value: AlignToUsize(value),
            unbounds: NotCopyNotClone,
            _prefix: PhantomData,
        }
    }

    /// A bit array that describes the accessibility of each field in `T`.
    #[inline]
    pub const fn field_accessibility(&self) -> FieldAccessibility {
        self.field_accessibility
    }

    /// The basic layout of the prefix type, for error messages.
    #[inline]
    pub const fn type_layout(&self) -> &'static PTStructLayout {
        self.type_layout
    }

    /// Constructs a `PrefixRef` from `this`.
    ///
    /// # Safety
    ///
    /// You must enture that this `WithMetadata` lives for the entire program's lifetime.
    #[inline]
    pub const unsafe fn raw_as_prefix(this: *const Self) -> PrefixRef<P> {
        unsafe { PrefixRef::from_raw(this) }
    }

    /// Constructs a `PrefixRef` from `self`.
    ///
    /// # Safety
    ///
    /// You must ensure that `self` lives for the entire program's lifetime.
    ///
    /// # Alternative
    ///
    /// For a safe equivalent of this, you can use [`StaticRef::as_prefix`].
    ///
    /// [`StaticRef::as_prefix`]: ../sabi_types/struct.StaticRef.html#method.as_prefix
    #[inline]
    pub const unsafe fn as_prefix(&self) -> PrefixRef<P> {
        unsafe { PrefixRef::from_raw(self) }
    }

    /// Constructs a `PrefixRef` from `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     for_examples::{Module, Module_Ref},
    ///     prefix_type::{PrefixRef, PrefixTypeTrait, WithMetadata},
    ///     std_types::{RSome, RStr},
    /// };
    ///
    /// const WITH_META: &WithMetadata<Module> = &WithMetadata::new(
    ///     Module {
    ///         first: RSome(13),
    ///         second: RStr::from_str("foo"),
    ///         third: 100,
    ///     },
    /// );
    ///
    /// const MOD: Module_Ref = Module_Ref(WITH_META.static_as_prefix());
    ///
    /// assert_eq!(MOD.first(), RSome(13));
    /// assert_eq!(MOD.second().as_str(), "foo");
    ///
    /// ```
    #[inline]
    pub const fn static_as_prefix(&'static self) -> PrefixRef<P> {
        PrefixRef::from_ref(self)
    }
}

impl<T, P> StaticRef<WithMetadata_<T, P>> {
    /// Constructs a `PrefixRef<P>` from self.
    ///
    /// This is most useful when you have a generic type that isn't `'static`
    /// for type system reasons, but lives for the entire program.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     for_examples::{PhantModule, PhantModule_Ref},
    ///     prefix_type::{PrefixRef, PrefixTypeTrait, WithMetadata},
    ///     std_types::{RNone, RStr},
    ///     staticref,
    /// };
    ///
    /// struct Foo<T>(T);
    ///
    /// impl<T: Copy> Foo<T> {
    ///     // The `staticref` invocation here declares a
    ///     // `StaticRef<WithMetadata<PhantModule<T>>>` constant.
    ///     staticref!(const WITH_META: WithMetadata<PhantModule<T>> = WithMetadata::new(
    ///         PhantModule {
    ///             first: RNone,
    ///             second: RStr::from_str("hello"),
    ///             third: 100,
    ///             phantom: std::marker::PhantomData,
    ///         },
    ///     ));
    /// }
    ///
    /// const MOD: PhantModule_Ref<()> = PhantModule_Ref(Foo::WITH_META.as_prefix());
    ///
    /// assert_eq!(MOD.first(), RNone);
    /// assert_eq!(MOD.second().as_str(), "hello");
    ///
    /// ```
    pub const fn as_prefix(self) -> PrefixRef<P> {
        PrefixRef::from_staticref(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Used to panic with an error message informing the user that a field
/// is expected to be on the `T` type when it's not.
#[cold]
#[inline(never)]
pub fn panic_on_missing_field_ty<T>(field_index: usize, actual_layout: &'static PTStructLayout) -> !
where
    T: PrefixTypeTrait,
{
    #[inline(never)]
    fn inner(
        field_index: usize,
        expected_layout: &'static PTStructLayout,
        actual_layout: &'static PTStructLayout,
    ) -> ! {
        let field = expected_layout
            .get_field_name(field_index)
            .unwrap_or("<unavailable>");
        panic_on_missing_field_val(field_index, field, expected_layout, actual_layout)
    }

    inner(field_index, T::PT_LAYOUT, actual_layout)
}

/// Used to panic with an error message informing the user that a field
/// is expected to be on the `T` type when it's not.
#[cold]
#[inline(never)]
pub fn panic_on_missing_fieldname<T>(field_index: u8, actual_layout: &'static PTStructLayout) -> !
where
    T: PrefixTypeTrait,
{
    #[inline(never)]
    fn inner(
        field_index: usize,
        expected_layout: &'static PTStructLayout,
        actual_layout: &'static PTStructLayout,
    ) -> ! {
        let fieldname = expected_layout
            .get_field_name(field_index)
            .unwrap_or("<unavaiable>");
        panic_on_missing_field_val(field_index, fieldname, expected_layout, actual_layout)
    }

    inner(field_index as usize, T::PT_LAYOUT, actual_layout)
}

/// Used to panic with an error message informing the user that a field
/// is expected to be on `expected` when it's not.
#[inline(never)]
fn panic_on_missing_field_val(
    field_index: usize,
    field_name: &'static str,
    expected: &'static PTStructLayout,
    actual: &'static PTStructLayout,
) -> ! {
    panic!(
        "\n
Attempting to access nonexistent field:
    index:{index} 
    named:{field_named}

Inside of:{struct_name}{struct_generics}

Package:'{package}' 

Expected:
    Version:{expected_package_version} (or compatible version number)
    Field count:{expected_field_count}

Found:
    Version:{actual_package_version}
    Field count:{actual_field_count}

\n",
        index = field_index,
        field_named = field_name,
        struct_name = expected.mono_layout.name(),
        struct_generics = expected.generics.as_str(),
        package = expected.mono_layout.item_info().package(),
        expected_package_version = expected.mono_layout.item_info().version(),
        expected_field_count = expected.get_field_names().count(),
        actual_package_version = actual.mono_layout.item_info().version(),
        actual_field_count = actual.get_field_names().count(),
    );
}
