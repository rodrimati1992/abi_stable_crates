//! Types used to represent values at compile-time, eg: True/False.

/// Type-level booleans.
///
/// This is a re-export from `core_extensions::type_level_bool`,
/// so as to allow glob imports (`abi_stable::type_level::bools::*`)
/// without worrying about importing too many items.
pub mod bools {
    #[doc(no_inline)]
    pub use core_extensions::type_level_bool::{Boolean, False, True};
}

// Uncomment if I have a use for type-level `Option`s
// pub mod option;

/// Type-level enum representing whether a
/// `DynTrait`/`RObject`/`#[sabi_trait]`-generated trait object
/// can be converted back into the concrete type they were constructed with.
pub mod downcasting;

/// Marker types representing traits.
pub mod trait_marker {
    ///
    pub struct Send;

    ///
    pub struct Sync;

    ///
    pub struct Clone;

    ///
    pub struct Default;

    ///
    pub struct Display;

    ///
    pub struct Debug;

    ///
    pub struct Eq;

    ///
    pub struct PartialEq;

    ///
    pub struct Ord;

    ///
    pub struct PartialOrd;

    ///
    pub struct Hash;

    /// Represents the [`serde::Deserialize`] trait.
    pub struct Deserialize;

    /// Represents the [`serde::Serialize`] trait.
    pub struct Serialize;

    ///
    pub struct Iterator;

    ///
    pub struct DoubleEndedIterator;

    /// Represents the [`std::fmt::Write`] trait.
    pub struct FmtWrite;

    /// Represents the [`std::io::Write`] trait.
    pub struct IoWrite;

    /// Represents the [`std::io::Seek`] trait.
    pub struct IoSeek;

    /// Represents the [`std::io::Read`] trait.
    pub struct IoRead;

    /// Represents the [`std::io::BufRead`] trait.
    pub struct IoBufRead;

    /// Represents the [`std::error::Error`] trait.
    pub struct Error;

    /// Represents the [`std::marker::Unpin`] trait.
    pub struct Unpin;

    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    pub struct define_this_in_the_impl_InterfaceType_macro;
}

/// Type-level-enum representing whether a trait is implemented or not implemented.
pub mod impl_enum {

    use crate::marker_type::NonOwningPhantom;

    use core_extensions::type_level_bool::{False, True};

    mod sealed {
        pub trait Sealed {}
    }
    use self::sealed::Sealed;

    /// Trait for [`Implemented`] and [`Unimplemented`]
    pub trait Implementability: Sealed {
        /// Whether the trait represented by the type parameter must be implemented.
        const IS_IMPLD: bool;
    }

    /// Converts a type to either `Unimplemented` or `Implemented`.
    ///
    /// The `T` type parameter represents the (un)required trait.
    ///
    pub trait ImplFrom_<T: ?Sized> {
        /// Either `Unimplemented` or `Implemented`.
        type Impl: ?Sized + Implementability;
    }

    impl<T: ?Sized> ImplFrom_<T> for False {
        type Impl = Unimplemented<T>;
    }

    impl<T: ?Sized> ImplFrom_<T> for True {
        type Impl = Implemented<T>;
    }

    impl<T: ?Sized> ImplFrom_<T> for Unimplemented<T> {
        type Impl = Unimplemented<T>;
    }

    impl<T: ?Sized> ImplFrom_<T> for Implemented<T> {
        type Impl = Implemented<T>;
    }

    /// Converts `B` to either `Unimplemented<T>` or `Implemented<T>`.
    ///
    /// The `T` type parameter represents the (un)required trait.
    pub type ImplFrom<B, T> = <B as ImplFrom_<T>>::Impl;

    /// Describes that a trait must be implemented.
    ///
    /// The `T` type parameter represents the required trait.
    pub struct Implemented<T: ?Sized>(NonOwningPhantom<T>);

    impl<T: ?Sized> Sealed for Implemented<T> {}
    impl<T: ?Sized> Implementability for Implemented<T> {
        const IS_IMPLD: bool = true;
    }

    /// Describes that a trait does not need to be implemented.
    ///
    /// The `T` type parameter represents the trait.
    pub struct Unimplemented<T: ?Sized>(NonOwningPhantom<T>);

    impl<T: ?Sized> Sealed for Unimplemented<T> {}
    impl<T: ?Sized> Implementability for Unimplemented<T> {
        const IS_IMPLD: bool = false;
    }
}
