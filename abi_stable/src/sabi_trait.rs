pub mod reexports{

    pub use std::{
        marker::{PhantomData as __Phantom},
        ops::{Deref as __DerefTrait,DerefMut as __DerefMutTrait},
    };


    pub mod __sabi_re{
        pub use abi_stable::{
            pointer_trait::{TransmuteElement,OwnedPointer},
            prefix_type::{PrefixTypeTrait,WithMetadata},
            traits::IntoInner,
            sabi_types::{StaticRef,MovePtr},
            utils::{transmute_reference,transmute_mut_reference,take_manuallydrop},
        };

        pub use core_extensions::utils::transmute_ignore_size;

        pub use std::{
            mem::ManuallyDrop,
            ptr,
        };
    }
}