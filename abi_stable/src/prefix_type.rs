/*!
Types,traits,and functions used by prefix-types.

*/

use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use crate::{
    pointer_trait::ImmutableRef,
    marker_type::NotCopyNotClone,
    sabi_types::StaticRef,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

mod accessible_fields;
mod layout;
mod pt_metadata;
mod prefix_ref;

pub use self::{
    accessible_fields::{
        BoolArray, FieldAccessibility, FieldConditionality, IsAccessible, IsConditional,
    },
    layout::PTStructLayout,
    prefix_ref::PrefixRef,
};

pub(crate) use self::pt_metadata::PrefixTypeMetadata;

/// For types deriving `StableAbi` with `#[sabi(kind(Prefix(..)))]`.
pub unsafe trait PrefixTypeTrait: Sized {
    /// The metadata of the prefix-type (a FieldAccessibility and a PTStructLayout),
    /// for passing to `WithMetadata::new`,
    /// with `WithMetadata::new(PrefixTypeTrait::METADATA,value)`
    const METADATA: PrefixMetadata<Self, Self::PrefixFields> = PrefixMetadata {
        field_accessibility: Self::PT_FIELD_ACCESSIBILITY,
        type_layout: Self::PT_LAYOUT,
        _marker: PhantomData,
    };

    /// Describes the layout of the struct,exclusively for use in error messages.
    const PT_LAYOUT: &'static PTStructLayout;

    /// A bit array,where the bit at the field index represents whether that
    /// field is accessible.
    const PT_FIELD_ACCESSIBILITY: FieldAccessibility;

    /// 
    fn from_prefix_ref(this: PrefixRef<Self::PrefixFields>)->Self::PrefixRef;
    
    fn to_prefix_ref(this: Self::PrefixRef)->PrefixRef<Self::PrefixFields>;

    /// Convers `Self` to `&'a Self::Prefix`,leaking it in the process.
    fn leak_into_prefix(self)->Self::PrefixRef{
        let x=WithMetadata::new(Self::METADATA, self);
        let x=StaticRef::leak_value(x);
        let x=PrefixRef::from_staticref(x);
        Self::from_prefix_ref(x)
    }

    type PrefixFields;

    type PrefixRef: ImmutableRef<Target = WithMetadata_<Self::PrefixFields, Self::PrefixFields>>;
}

////////////////////////////////////////////////////////////////////////////////

pub type WithMetadata<T, P = <T as PrefixTypeTrait>::PrefixFields> = 
    WithMetadata_<T, P>;


/// Wraps a type so that it can be converted to its prefix.
#[repr(C)]
pub struct WithMetadata_<T, P> {
    pub metadata: PrefixMetadata<T, P>,
    // Forces value to be aligned to at least a usize.
    _alignment: [usize; 0],
    /// The wrapped value.
    pub value: T,
    unbounds: NotCopyNotClone,
}

impl<T, P> WithMetadata_<T, P> {
    /// Constructs Self with `WithMetadata::new(PrefixTypeTrait::METADATA,value)`
    ///
    /// This takes in the `metadata:PrefixMetadata<T>` parameter as a
    /// workaround for `const fn` not allowing trait bounds,
    /// which in case is `PrefixTypeTrait`.
    #[inline]
    pub const fn new(metadata: PrefixMetadata<T, P>, value: T) -> Self {
        Self {
            metadata,
            _alignment: [],
            value,
            unbounds: NotCopyNotClone,
        }
    }

    /// 
    /// 
    /// # Safety 
    /// 
    /// TODO
    #[inline]
    pub const unsafe fn raw_as_prefix(this: *const Self) -> PrefixRef<P> {
        PrefixRef::from_raw(this)
    }

    /// 
    /// 
    /// # Safety 
    /// 
    /// TODO
    #[inline]
    pub const unsafe fn as_prefix(&self) -> PrefixRef<P> {
        PrefixRef::from_raw(self)
    }

    #[inline]
    pub const fn static_as_prefix(&'static self) -> PrefixRef<P> {
        PrefixRef::from_ref(self) 
    }
}

impl<T, P> StaticRef<WithMetadata_<T, P>>{
    pub const fn as_prefix(self) -> PrefixRef<P> {
        PrefixRef::from_staticref(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// The prefix-type metadata for `T` (with a FieldAccessibility and a PTStructLayout).
/// This is only constructed in PrefixTypeTrait::METADATA.
///
/// This is used as a workaround for `const fn` not allowing trait bounds.
#[repr(C)]
pub struct PrefixMetadata<T, P> {
    /// A bit array,where the bit at field index represents whether a field is accessible.
    field_accessibility: FieldAccessibility,
    /// Yhe basic layout of the prefix type.
    type_layout: &'static PTStructLayout,
    _marker: PhantomData<(T, P)>,
}

impl<T, P> Copy for PrefixMetadata<T, P> {}
impl<T, P> Clone for PrefixMetadata<T, P> {
    fn clone(&self)->Self{
        *self
    }
}

impl<T, P> PrefixMetadata<T, P> {
    #[inline]
    pub const fn field_accessibility(self) -> FieldAccessibility {
        self.field_accessibility
    }
    #[inline]
    pub const fn type_layout(self) -> &'static PTStructLayout {
        self.type_layout
    }
}

impl<T, P> Debug for PrefixMetadata<T, P>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>)-> fmt::Result {
        f.debug_struct("PrefixMetadata")
         .field("field_accessibility", &self.field_accessibility)
         .field("type_layout", &self.type_layout)
         .finish()        
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
    pub fn inner(
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
pub fn panic_on_missing_field_val(
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
