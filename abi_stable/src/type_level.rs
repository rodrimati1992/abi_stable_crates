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
        sabi_types::{Constructor,MaybeCmp},
        std_types::utypeid::{UTypeId,no_utypeid,some_utypeid},
    };


    /// Indicates that a type implements `Any`.
    ///
    /// A trait object wrapping that type can be unerased back into taht type.
    #[allow(non_camel_case_types)]
    #[derive(Copy,Clone)]
    pub struct TU_Unerasable;

    /// Indicates that a type does not implement `Any`,
    ///
    /// A trait object wrapping that type can't be unerased.
    #[allow(non_camel_case_types)]
    #[derive(Copy,Clone)]
    pub struct TU_Opaque;
    




    /// Gets a function optionally returning the `UTypeId` of `T`.
    ///
    /// Whether the function returns `MaybeCmp::Just(typeid)` is determined by implementors:
    /// 
    /// - `TU_Unerasable`: the function always returns `MaybeCmp::Just(typeid)`.
    /// 
    /// - `TU_Opaque`: the function always returns `MaybeCmp::Nothing`.
    pub trait GetUTID<T> {
        /// A struct wrapping the function.
        const UID:Constructor<MaybeCmp<UTypeId>>;
    }


    impl<T> GetUTID<T> for TU_Unerasable
    where T:'static
    {
        const UID:Constructor<MaybeCmp<UTypeId>>=Constructor( some_utypeid::<T> );
    }

    impl<T> GetUTID<T> for TU_Opaque{
        const UID:Constructor<MaybeCmp<UTypeId>>=Constructor( no_utypeid );
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
    pub struct Eq;
    pub struct PartialEq;
    pub struct Ord;
    pub struct PartialOrd;
    pub struct Hash;

    /// Represents the `serde::Deserialize` trait.
    pub struct Deserialize;

    /// Represents the `serde::Serialize` trait.
    pub struct Serialize;
    
    pub struct Iterator;
    pub struct DoubleEndedIterator;
    
    /// Represents the `std::fmt::Write` trait.
    pub struct FmtWrite;
    
    /// Represents the `std::io::Write` trait.
    pub struct IoWrite;
    
    /// Represents the `std::io::Seek` trait.
    pub struct IoSeek;
    
    /// Represents the `std::io::Read` trait.
    pub struct IoRead;

    /// Represents the `std::io::BufRead` trait.
    pub struct IoBufRead;
    
    /// Represents the `std::error::Error` trait.
    pub struct Error;
    
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    pub struct define_this_in_the_impl_InterfaceType_macro;
}


/// Type-level-enum representing whether a trait is implemented or not implemented.
pub mod impl_enum{

    use crate::marker_type::NonOwningPhantom;

    use core_extensions::type_level_bool::{True,False};

    mod sealed{
        pub trait Sealed{}
    }
    use self::sealed::Sealed;

    /// Queries whether this type is `Implemented<T>`
    pub trait IsImplemented:Sealed{
        /// Whether the trait represented by the type parameter must be implemented.
        const VALUE:bool;
    }
    

    /// Converts a type to either `Unimplemented` or `Implemented`.
    ///
    /// The `T` type parameter represents the (un)required trait.
    ///
    pub trait ImplFrom_<T:?Sized>{
        /// Either `Unimplemented` or `Implemented`.
        type Impl:?Sized;
        
        /// Either `Unimplemented` or `Implemented`.
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

    /// Converts `B` to either `Unimplemented<T>` or `Implemented<T>`.
    ///
    /// The `T` type parameter represents the (un)required trait.
    pub type ImplFrom<B,T>=
        <B as ImplFrom_<T>>::Impl;



    /// Describes that a trait must be implemented.
    ///
    /// The `T` type parameter represents the required trait.
    pub struct Implemented<T:?Sized>(NonOwningPhantom<T>);

    impl<T:?Sized> Implemented<T>{
        /// Constructs an `Implemented`.
        pub const NEW:Implemented<T>=Implemented(NonOwningPhantom::NEW);
    }

    impl<T> Sealed for Implemented<T>{}
    impl<T> IsImplemented for Implemented<T>{
        const VALUE:bool=true;
    }



    /// Describes that a trait does not need to be implemented.
    ///
    /// The `T` type parameter represents the trait.
    pub struct Unimplemented<T:?Sized>(NonOwningPhantom<T>);

    impl<T:?Sized> Unimplemented<T>{
        /// Constructs an `Unimplemented`.
        pub const NEW:Unimplemented<T>=Unimplemented(NonOwningPhantom::NEW);
    }

    impl<T> Sealed for Unimplemented<T>{}
    impl<T> IsImplemented for Unimplemented<T>{
        const VALUE:bool=false;
    }
}