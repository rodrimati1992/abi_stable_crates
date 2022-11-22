//! Contains items related to the [`#[sabi_trait]`](macro@crate::sabi_trait) attribute.

#[doc(hidden)]
pub mod reexports {

    pub use std::ops::{Deref as __DerefTrait, DerefMut as __DerefMutTrait};

    pub use crate::{
        marker_type::ErasedObject as __ErasedObject,
        pointer_trait::GetPointerKind as __GetPointerKind,
    };

    pub mod __sabi_re {
        pub use abi_stable::{
            erased_types::{
                DynTrait, MakeVTable as MakeDynTraitVTable, VTable_Ref as DynTraitVTable_Ref,
            },
            extern_fn_panic_handling,
            marker_type::{
                NonOwningPhantom, SyncSend, SyncUnsend, UnsafeIgnoredType, UnsyncSend, UnsyncUnsend,
            },
            pointer_trait::{AsMutPtr, AsPtr, CanTransmuteElement, OwnedPointer, TransmuteElement},
            prefix_type::{PrefixRef, PrefixTypeTrait, WithMetadata},
            sabi_trait::{
                robject::RObject,
                vtable::{GetRObjectVTable, RObjectVtable, RObjectVtable_Ref},
            },
            sabi_types::{MovePtr, RMut, RRef},
            std_types::RBox,
            traits::IntoInner,
            utils::take_manuallydrop,
        };

        pub use core_extensions::{utils::transmute_ignore_size, TypeIdentity};

        pub use std::{
            marker::PhantomData,
            mem::{transmute, ManuallyDrop},
            ops::Deref,
            ptr,
        };
    }
}

/// A prelude for modules using `#[sabi_trait]` generated traits/trait objects.
pub mod prelude {
    pub use crate::type_level::downcasting::{TD_CanDowncast, TD_Opaque};
}

pub use crate::type_level::downcasting::{TD_CanDowncast, TD_Opaque};

#[cfg(any(test, feature = "sabi_trait_examples"))]
pub mod examples;

pub mod doc_examples;

mod robject;

#[doc(hidden)]
pub mod vtable;

#[cfg(test)]
pub mod tests;

#[cfg(all(test, not(feature = "only_new_tests")))]
pub mod test_supertraits;

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use self::reexports::__sabi_re::*;

pub use self::robject::{RObject, ReborrowBounds, UneraseError};

use crate::{erased_types::c_functions, marker_type::ErasedObject, sabi_types::MaybeCmp};
