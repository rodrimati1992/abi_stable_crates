pub mod reexports{

    pub use std::{
        ops::{Deref as __DerefTrait,DerefMut as __DerefMutTrait},
    };

    pub use crate::marker_type::ErasedObject as __ErasedObject;


    pub mod __sabi_re{
        pub use abi_stable::{
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

pub mod prelude{
    pub use super::{
        markers::{YesImplAny,NoImplAny},
    };
}

pub mod for_generated_code;
pub mod robject;
pub mod vtable;

use std::{
    fmt::Debug,
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
    abi_stability::Tag,
    erased_types::{c_functions,InterfaceType},
    marker_type::ErasedObject,
    type_level::bools::{True,False,Boolean},
    sabi_types::MaybeCmp,
    std_types::Tuple2,
};


pub mod markers{
    use super::*;

    use crate::{
        std_types::utypeid::{UTypeId,no_utypeid,some_utypeid},
        return_value_equality::ReturnValueEquality,
    };


    /// Indicates that a type does not implement `Any`.
    pub struct YesImplAny;

    /// Indicates that a type implements `Any`.
    pub struct NoImplAny;
    




    /// Gets a function optionally returning the UTypeId of `T`.
    /// Whether the function returns `MaybeCmp::Just(typeid)` is determined by `Self`.
    pub trait GetUTID<T>{
        const UID:ReturnValueEquality<MaybeCmp<UTypeId>>;
    }


    impl<T> GetUTID<T> for YesImplAny
    where T:'static
    {
        const UID:ReturnValueEquality<MaybeCmp<UTypeId>>=ReturnValueEquality{
            function:some_utypeid::<T>
        };
    }

    impl<T> GetUTID<T> for NoImplAny{
        const UID:ReturnValueEquality<MaybeCmp<UTypeId>>=ReturnValueEquality{
            function:no_utypeid
        };
    }

}
