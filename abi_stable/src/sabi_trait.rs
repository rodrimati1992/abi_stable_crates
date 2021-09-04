/*!
Contains items related to the `#[sabi_trait]` attribute.
*/

#[doc(hidden)]
pub mod reexports{

    pub use std::{
        ops::{Deref as __DerefTrait,DerefMut as __DerefMutTrait},
    };

    pub use crate::{
        marker_type::ErasedObject as __ErasedObject,
        pointer_trait::GetPointerKind as __GetPointerKind,
    };


        

    pub mod __sabi_re{
        pub use abi_stable::{
            erased_types::{
                DynTrait,
                GetVtable,
                VTableDT,
                traits::InterfaceFor,
            },
            marker_type::{
                UnsafeIgnoredType,
                SyncSend,UnsyncUnsend,UnsyncSend,SyncUnsend,
                NonOwningPhantom,
            },
            pointer_trait::{AsPtr, AsMutPtr, CanTransmuteElement,TransmuteElement,OwnedPointer},
            prefix_type::{PrefixRef, PrefixTypeTrait, WithMetadata},
            traits::IntoInner,
            sabi_types::{RRef,RMut,MovePtr},
            sabi_trait::{
                robject::{
                    RObject,
                },
                vtable::{
                    RObjectVtable_Ref, RObjectVtable, GetRObjectVTable,
                    VTableTO_DT,VTableTO_RO,VTableTO,
                },
            },
            std_types::RBox,
            utils::take_manuallydrop,
            extern_fn_panic_handling,
        };

        pub use core_extensions::{
            utils::transmute_ignore_size,
            TypeIdentity,
        };

        pub use std::{
            marker::PhantomData,
            mem::{ManuallyDrop, transmute},
            ops::Deref,
            ptr,
        };
    }
}

/// A prelude for modules using `#[sabi_trait]` generated traits/trait objects.
pub mod prelude{
    pub use crate::type_level::downcasting::{TD_CanDowncast,TD_Opaque};
}

pub use crate::type_level::downcasting::{TD_CanDowncast,TD_Opaque};

#[cfg(any(test,feature="sabi_trait_examples"))]
pub mod examples;

pub mod doc_examples;

mod robject;

#[doc(hidden)]
pub mod vtable;

#[cfg(test)]
pub mod tests;

#[cfg(all(test,not(feature="only_new_tests")))]
pub mod test_supertraits;

use std::{
    fmt::{Debug,Display},
    marker::PhantomData,
};

use self::reexports::__sabi_re::*;

pub use self::{
    vtable::{VTableTO_DT,VTableTO_RO,VTableTO},
    robject::{RObject, UneraseError, ReborrowBounds},
};

use crate::{
    erased_types::{c_functions,InterfaceType},
    marker_type::ErasedObject,
    sabi_types::MaybeCmp,
};
