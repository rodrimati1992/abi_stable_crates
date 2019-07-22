pub mod doc_enums;

#[cfg(test)]
pub mod examples;

pub mod nonexhaustive;
pub mod vtable;
pub mod traits;
pub mod alt_c_functions;


pub use self::{
    nonexhaustive::{NonExhaustive,NonExhaustiveFor,NonExhaustiveWI,NonExhaustiveWS},
    vtable::{GetVTable,NonExhaustiveVtable,InterfaceBound},
    traits::{
        GetNonExhaustive,
        GetEnumInfo,
        EnumInfo,
        ValidDiscriminant,
        SerializeEnum,
        DeserializeEnum,
        GetSerializeEnumProxy,
        GetDeserializeEnumProxy,
    },
};



/////////////////////////////////////////////////////////////



pub fn assert_nonexhaustive<T>(type_:&'static str,storage_:&'static str)
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
        type_,
        type_size:std::mem::size_of::<T>(),
        type_alignment:std::mem::align_of::<T>(),
        storage_,
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