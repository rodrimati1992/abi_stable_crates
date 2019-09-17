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
                traits::InterfaceFor,
            },
            marker_type::{UnsafeIgnoredType,SyncSend,UnsyncUnsend,UnsyncSend,SyncUnsend},
            pointer_trait::{TransmuteElement,OwnedPointer},
            prefix_type::{PrefixTypeTrait,WithMetadata},
            traits::IntoInner,
            sabi_types::{StaticRef,MovePtr},
            sabi_trait::{
                robject::{
                    RObject,
                },
                vtable::{GetVTable,RObjectVtable,GetRObjectVTable},
                for_generated_code::{sabi_from_ref,sabi_from_mut},
            },
            std_types::RBox,
            utils::{transmute_reference,transmute_mut_reference,take_manuallydrop},
        };

        pub use core_extensions::{
            utils::transmute_ignore_size,
            TypeIdentity,
        };

        pub use std::{
            marker::PhantomData,
            mem::ManuallyDrop,
            ptr,
        };
    }
}

/// A prelude for modules using `#[sabi_trait]` generated traits/trait objects.
pub mod prelude{
    pub use crate::type_level::unerasability::{TU_Unerasable,TU_Opaque};
}

#[doc(hidden)]
pub mod for_generated_code;
#[cfg(any(
    all(test,not(feature="only_new_tests")),
    feature="sabi_trait_examples"
))]

#[cfg(any(test,feature="sabi_trait_examples"))]
pub mod examples;

/**
Contains `RObject` and related items.
*/
pub mod robject;

#[doc(hidden)]
pub mod vtable;

#[cfg(test)]
// #[cfg(all(test,not(feature="only_new_tests")))]
pub mod tests;

#[cfg(all(test,not(feature="only_new_tests")))]
pub mod test_supertraits;

use std::{
    fmt::{Debug,Display},
    marker::PhantomData,
};

use self::{
    reexports::{
        *,
        __sabi_re::*,
    },
    vtable::BaseVtable,
};

use crate::{
    erased_types::{c_functions,InterfaceType},
    marker_type::ErasedObject,
    sabi_types::MaybeCmp,
    std_types::Tuple2,
};