//! Contains types and traits for nonexhaustive enums.
//!
//! The most important type here is [NonExhaustive](./nonexhaustive/struct.NonExhaustive.html),
//! which allows passing an enum which used the
//! `#[derive(StableAbi)] #[sabi(kind(WithNonExhaustive(...)))]`
//! attributes through ffi.
//!

#[doc(hidden)]
pub mod doc_enums;

#[cfg(any(feature = "testing", feature = "nonexhaustive_examples"))]
pub mod examples;

pub(crate) mod alt_c_functions;
pub(crate) mod nonexhaustive;
pub(crate) mod traits;
pub(crate) mod vtable;

pub use self::{
    nonexhaustive::{
        NonExhaustive, NonExhaustiveFor, NonExhaustiveSharedOps, NonExhaustiveWI, NonExhaustiveWS,
        UnwrapEnumError,
    },
    traits::{
        DeserializeEnum, EnumInfo, GetEnumInfo, NonExhaustiveMarker, SerializeEnum,
        ValidDiscriminant,
    },
    vtable::GetVTable,
};

pub(crate) use self::traits::GetSerializeEnumProxy;

/////////////////////////////////////////////////////////////

/// Asserts that the size and alignment of an enum are valid for its default storage.
#[track_caller]
pub fn assert_correct_default_storage<E>()
where
    E: GetEnumInfo,
{
    assert_correct_storage::<E, E::DefaultStorage>(AssertCsArgs {
        enum_ty: std::any::type_name::<E>(),
        storage_ty: std::any::type_name::<E::DefaultStorage>(),
    })
}

/// Arguments for [`assert_correct_storage`]
pub struct AssertCsArgs {
    /// The stringified type name of the enum.
    pub enum_ty: &'static str,
    /// The stringified type name of the storage.
    pub storage_ty: &'static str,
}

impl AssertCsArgs {
    /// Constant where both types are unknown.
    pub const UNKNOWN: Self = Self {
        enum_ty: "<unknown>",
        storage_ty: "<unknown>",
    };
}

/// Asserts that the size and alignment of an enum aree valid for this storage.
///
/// To make this function a `const fn`,
/// the names of the `Enum` and `Storage` types must be passed separately.
#[track_caller]
pub const fn assert_correct_storage<Enum, Storage>(args: AssertCsArgs) {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct TypeAndStorageLayout {
        enum_: &'static str,
        enum_size: usize,
        enum_alignment: usize,
        storage_: &'static str,
        storage_size: usize,
        storage_alignment: usize,
    }

    #[track_caller]
    const fn inner(lay: TypeAndStorageLayout) {
        let msg = match (
            lay.enum_alignment <= lay.storage_alignment,
            lay.enum_size <= lay.storage_size,
        ) {
            (false, false) => {
                "The alignment and size of the storage is smaller than the contained type"
            }
            (false, true) => "The alignment of the storage is lower than the contained type",
            (true, false) => "The size of the storage is smaller than the contained type",
            (true, true) => return,
        };

        const_panic::concat_panic!(
            "\n",
            display: msg,
            ":\n",
            "\tenum_: ",
            lay.enum_,
            "\n",
            "\tenum_size: ",
            lay.enum_size,
            "\n",
            "\tenum_alignment: ",
            lay.enum_alignment,
            "\n",
            "\tstorage_: ",
            lay.storage_,
            "\n",
            "\tstorage_size: ",
            lay.storage_size,
            "\n",
            "\tstorage_alignment: ",
            lay.storage_alignment,
            "\n",
        )
    }

    inner(TypeAndStorageLayout {
        enum_: args.enum_ty,
        enum_size: std::mem::size_of::<Enum>(),
        enum_alignment: std::mem::align_of::<Enum>(),
        storage_: args.storage_ty,
        storage_size: std::mem::size_of::<Storage>(),
        storage_alignment: std::mem::align_of::<Storage>(),
    })
}
