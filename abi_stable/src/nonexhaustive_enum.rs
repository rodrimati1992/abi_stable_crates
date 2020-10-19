/*!
Contains types and traits for nonexhaustive enums.

The most important type here is [NonExhaustive](./nonexhaustive/struct.NonExhaustive.html),
which allows passing an enum which used the
`#[derive(StableAbi)] #[sabi(kind(WithNonExhaustive(...)))]`
attributes through ffi.

*/

#[doc(hidden)]
pub mod doc_enums;

#[cfg(any(feature = "testing",feature="nonexhaustive_examples"))]
pub mod examples;

pub(crate) mod nonexhaustive;
pub(crate) mod vtable;
pub(crate) mod traits;
pub(crate) mod alt_c_functions;


pub(crate) use self::vtable::NonExhaustiveVtable_Ref;


pub use self::{
    nonexhaustive::{
        DiscrAndEnumInfo,
        NonExhaustive,NonExhaustiveFor,NonExhaustiveWI,NonExhaustiveWS,
        NonExhaustiveSharedOps,
        UnwrapEnumError,
    },
    vtable::GetVTable,
    traits::{
        GetNonExhaustive,
        GetEnumInfo,
        EnumInfo,
        ValidDiscriminant,
        SerializeEnum,
        DeserializeEnum,
    },
};


pub(crate) use self::{
    traits::{
        GetSerializeEnumProxy,
    },
};



/////////////////////////////////////////////////////////////

/// Asserts that the size and alignment of an enum are valid for its default storage.
pub fn assert_nonexhaustive<T>()
where
    T:GetEnumInfo,
    T:GetVTable<
        <T as GetEnumInfo>::DefaultStorage,
        <T as GetEnumInfo>::DefaultInterface,
    >
{
    #[derive(Debug)]
    struct TypeAndStorageLayout{
        type_:&'static str,
        type_size:usize,
        type_alignment:usize,
        storage_:&'static str,
        storage_size:usize,
        storage_alignment:usize,
    }

    let lay=TypeAndStorageLayout{
        type_: std::any::type_name::<T>(),
        type_size:std::mem::size_of::<T>(),
        type_alignment:std::mem::align_of::<T>(),
        storage_: std::any::type_name::<<T as GetEnumInfo>::DefaultStorage>(),
        storage_size:std::mem::size_of::<T::DefaultStorage>(),
        storage_alignment:std::mem::align_of::<T::DefaultStorage>(),
    };

    assert!(
        NonExhaustiveFor::<T>::check_alignment(),
        "The alignment of the storage is different than the enum:\n{:#?}",
        lay
    );

    assert!(
        NonExhaustiveFor::<T>::check_size(),
        "The size of the storage is smaller than the enum:\n{:#?}",
        lay
    );
}