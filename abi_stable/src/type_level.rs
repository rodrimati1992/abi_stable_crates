/*!
Types used to represent values at compile-time,eg:True/False.
*/

/**
Type-level booleans.

This is a re-export from `core_extensions::type_level_bool`,
so as to allow glob imports (`abi_stable::type_level_bool::*`)
without worrying about importing too many items.
*/
pub mod bools{
    #[doc(inline)]
    pub use core_extensions::type_level_bool::{True,False,Boolean};
}


// Uncomment if I have a use for type-level `Option`s
// pub mod option;


pub mod unerasability{
    use crate::{
        sabi_types::{MaybeCmp,ReturnValueEquality},
        std_types::utypeid::{UTypeId,no_utypeid,some_utypeid},
    };


    /// Indicates that a type implements `Any`.
    pub struct TU_Unerasable;

    /// Indicates that a type does not implement `Any`.
    pub struct TU_Opaque;
    




    /// Gets a function optionally returning the UTypeId of `T`.
    /// Whether the function returns `MaybeCmp::Just(typeid)` is determined by `Self`.
    pub trait GetUTID<T>{
        const UID:ReturnValueEquality<MaybeCmp<UTypeId>>;
    }


    impl<T> GetUTID<T> for TU_Unerasable
    where T:'static
    {
        const UID:ReturnValueEquality<MaybeCmp<UTypeId>>=ReturnValueEquality{
            function:some_utypeid::<T>
        };
    }

    impl<T> GetUTID<T> for TU_Opaque{
        const UID:ReturnValueEquality<MaybeCmp<UTypeId>>=ReturnValueEquality{
            function:no_utypeid
        };
    }
}