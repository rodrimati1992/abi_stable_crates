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

/// Type-level enum representing whether a 
/// `DynTrait`/`RObject`/`#[sabi_trait]`-generated trait object
/// can be converted back into the concrete type they were constructed with.
pub mod unerasability{
    use crate::{
        sabi_types::{MaybeCmp,ReturnValueEquality},
        std_types::utypeid::{UTypeId,no_utypeid,some_utypeid},
    };


    /// Indicates that a type implements `Any`.
    #[allow(non_camel_case_types)]
    pub struct TU_Unerasable;

    /// Indicates that a type does not implement `Any`.
    #[allow(non_camel_case_types)]
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


/// Marker types representing traits.
pub mod trait_marker{
    pub struct Send;
    pub struct Sync;
    pub struct Clone;
    pub struct Default;
    pub struct Display;
    pub struct Debug;
    pub struct Serialize;
    pub struct Eq;
    pub struct PartialEq;
    pub struct Ord;
    pub struct PartialOrd;
    pub struct Hash;
    pub struct Deserialize;
    pub struct Iterator;
    pub struct DoubleEndedIterator;
    pub struct FmtWrite;
    pub struct IoWrite;
    pub struct IoSeek;
    pub struct IoRead;
    pub struct IoBufRead;
    pub struct Error;
    
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    pub struct define_this_in_the_impl_InterfaceType_macro;
}


/// Type-level-enum representing whether a trait is implemented or not implemented.
pub mod impl_enum{

    use std::marker::PhantomData;

    use core_extensions::type_level_bool::{True,False};

    mod sealed{
        pub trait Sealed{}
    }
    use self::sealed::Sealed;

    /// Queries whether this type is `Implemented<T>`
    pub trait IsImplemented:Sealed{
        const VALUE:bool;
    }

    

    /// Converts a type to either Unimplemented<T> or Implemented<T>.
    pub trait ImplFrom_<T:?Sized>{
        type Impl:?Sized;
        const IMPL:Self::Impl;
    }

    impl<T:?Sized> ImplFrom_<T> for False {
        type Impl=Unimplemented<T>;
        const IMPL:Unimplemented<T>=Unimplemented::NEW;
    }

    impl<T:?Sized> ImplFrom_<T> for True {
        type Impl=Implemented<T>;
        const IMPL:Implemented<T>=Implemented::NEW;
    }

    impl<T:?Sized> ImplFrom_<T> for Unimplemented<T> {
        type Impl=Unimplemented<T>;
        const IMPL:Unimplemented<T>=Unimplemented::NEW;
    }

    impl<T:?Sized> ImplFrom_<T> for Implemented<T> {
        type Impl=Implemented<T>;
        const IMPL:Implemented<T>=Implemented::NEW;
    }

    /// Converts False to Unimplemented<T> and True to Implemented<T>.
    pub type ImplFrom<B,T>=
        <B as ImplFrom_<T>>::Impl;



    /// Describes a trait being implemented.
    pub struct Implemented<T:?Sized>(PhantomData<fn()->T>);

    impl<T:?Sized> Implemented<T>{
        pub const NEW:Implemented<T>=Implemented(PhantomData);
    }

    impl<T> Sealed for Implemented<T>{}
    impl<T> IsImplemented for Implemented<T>{
        const VALUE:bool=true;
    }



    /// Describes a trait being unimplemented.
    pub struct Unimplemented<T:?Sized>(PhantomData<fn()->T>);

    impl<T:?Sized> Unimplemented<T>{
        pub const NEW:Unimplemented<T>=Unimplemented(PhantomData);
    }

    impl<T> Sealed for Unimplemented<T>{}
    impl<T> IsImplemented for Unimplemented<T>{
        const VALUE:bool=false;
    }
}