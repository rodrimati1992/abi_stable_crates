use super::*;

use crate::{
    erased_types::{FormattingMode, InterfaceType, MakeRequiredTraits},
    marker_type::NonOwningPhantom,
    std_types::{RResult, RString, UTypeId},
    type_level::{
        downcasting::GetUTID,
        impl_enum::{Implemented, Unimplemented},
        trait_marker,
    },
};

use std::marker::PhantomData;

//////////////////////////////////////////////////////////////////////////////

#[doc(hidden)]
pub struct Private<A: ?Sized, B: ?Sized, C: ?Sized, D: ?Sized, E: ?Sized>(
    PhantomData<(
        PhantomData<A>,
        PhantomData<B>,
        PhantomData<C>,
        PhantomData<D>,
        PhantomData<E>,
    )>,
);

/// Gets an `RObjectVtable_Ref<_Self,ErasedPtr,TO>`(the vtable for RObject itself),
/// which is stored as the first field of all generated trait object vtables.
///
/// This trait cannot be implemented outside of `abi_stable`,
/// it can only be used.
pub trait GetRObjectVTable<IA, _Self, ErasedPtr, OrigPtr>: Sized + InterfaceType {
    // Using privacy to make it impossible to implement this trait outside this module.
    #[doc(hidden)]
    const __HIDDEN_10341423423AB__: Private<Self, IA, _Self, ErasedPtr, OrigPtr>;

    const ROBJECT_VTABLE: RObjectVtable_Ref<_Self, ErasedPtr, Self>;
}

impl<IA, _Self, ErasedPtr, OrigPtr, I> GetRObjectVTable<IA, _Self, ErasedPtr, OrigPtr> for I
where
    I: AreTraitsImpld<IA, _Self, ErasedPtr, OrigPtr> + InterfaceType,
{
    const __HIDDEN_10341423423AB__: Private<Self, IA, _Self, ErasedPtr, OrigPtr> =
        Private(std::marker::PhantomData);

    const ROBJECT_VTABLE: RObjectVtable_Ref<_Self, ErasedPtr, Self> =
        { GetRObjectVTableHelper::<IA, _Self, ErasedPtr, OrigPtr, I>::TMP_VTABLE };
}

//////////////////////////////////////////////////////////////////////////////

#[doc(hidden)]
pub trait AreTraitsImpld<IA, _Self, ErasedPtr, OrigPtr>: Sized {
    const VTABLE_VAL: RObjectVtable<_Self, ErasedPtr, Self>;
}

impl<IA, _Self, ErasedPtr, OrigPtr, I> AreTraitsImpld<IA, _Self, ErasedPtr, OrigPtr> for I
where
    I: InterfaceType,
    I::Sync: RequiresSync<_Self, ErasedPtr, OrigPtr>,
    I::Send: RequiresSend<_Self, ErasedPtr, OrigPtr>,
    I::Clone: InitCloneField<_Self, ErasedPtr, OrigPtr>,
    I::Debug: InitDebugField<_Self, ErasedPtr, OrigPtr>,
    I::Display: InitDisplayField<_Self, ErasedPtr, OrigPtr>,
    IA: GetUTID<_Self>,
{
    const VTABLE_VAL: RObjectVtable<_Self, ErasedPtr, I> = RObjectVtable {
        _sabi_tys: NonOwningPhantom::NEW,
        _sabi_type_id: <IA as GetUTID<_Self>>::UID,
        _sabi_drop: c_functions::drop_pointer_impl::<OrigPtr, ErasedPtr>,
        _sabi_clone: <I::Clone as InitCloneField<_Self, ErasedPtr, OrigPtr>>::VALUE,
        _sabi_debug: <I::Debug as InitDebugField<_Self, ErasedPtr, OrigPtr>>::VALUE,
        _sabi_display: <I::Display as InitDisplayField<_Self, ErasedPtr, OrigPtr>>::VALUE,
    };
}

// A dummy type used to get around a compiler limitation WRT associated constants in traits.
#[doc(hidden)]
struct GetRObjectVTableHelper<IA, _Self, ErasedPtr, OrigPtr, I>(IA, _Self, ErasedPtr, OrigPtr, I);

impl<IA, _Self, ErasedPtr, OrigPtr, I> GetRObjectVTableHelper<IA, _Self, ErasedPtr, OrigPtr, I>
where
    I: AreTraitsImpld<IA, _Self, ErasedPtr, OrigPtr>,
{
    staticref! {
        const TMP_WM: WithMetadata<RObjectVtable<_Self,ErasedPtr,I>> =
            WithMetadata::new(I::VTABLE_VAL);
    }

    const TMP_VTABLE: RObjectVtable_Ref<_Self, ErasedPtr, I> =
        RObjectVtable_Ref(Self::TMP_WM.as_prefix());
}

/// The vtable for RObject,which all  `#[trait_object]` derived trait objects contain.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
#[sabi(missing_field(default))]
pub struct RObjectVtable<_Self, ErasedPtr, I> {
    pub _sabi_tys: NonOwningPhantom<(_Self, ErasedPtr, I)>,

    pub _sabi_type_id: extern "C" fn() -> MaybeCmp<UTypeId>,

    pub _sabi_drop: unsafe extern "C" fn(this: RMut<'_, ErasedPtr>),
    pub _sabi_clone: Option<unsafe extern "C" fn(this: RRef<'_, ErasedPtr>) -> ErasedPtr>,
    pub _sabi_debug: Option<
        unsafe extern "C" fn(
            RRef<'_, ErasedObject>,
            FormattingMode,
            &mut RString,
        ) -> RResult<(), ()>,
    >,
    #[sabi(last_prefix_field)]
    pub _sabi_display: Option<
        unsafe extern "C" fn(
            RRef<'_, ErasedObject>,
            FormattingMode,
            &mut RString,
        ) -> RResult<(), ()>,
    >,
}

/// The common prefix of all `#[trait_object]` derived vtables,
/// with `RObjectVtable_Ref` as its first field.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    bound(I: InterfaceType),
    extra_checks = <I as MakeRequiredTraits>::MAKE,
    kind(Prefix)
)]
pub(super) struct BaseVtable<_Self, ErasedPtr, I> {
    pub _sabi_tys: NonOwningPhantom<(_Self, ErasedPtr, I)>,

    #[sabi(last_prefix_field)]
    pub _sabi_vtable: RObjectVtable_Ref<_Self, ErasedPtr, I>,
}

use self::trait_bounds::*;
pub mod trait_bounds {
    use super::*;

    macro_rules! declare_conditional_marker {
        (
            type $selector:ident;
            trait $trait_name:ident[$self_:ident,$ErasedPtr:ident,$OrigPtr:ident]
            where [ $($where_preds:tt)* ]
        ) => (
            pub trait $trait_name<$self_,$ErasedPtr,$OrigPtr>{}

            impl<$self_,$ErasedPtr,$OrigPtr> $trait_name<$self_,$ErasedPtr,$OrigPtr>
            for Unimplemented<trait_marker::$selector>
            {}

            impl<$self_,$ErasedPtr,$OrigPtr> $trait_name<$self_,ErasedPtr,$OrigPtr>
            for Implemented<trait_marker::$selector>
            where
                $($where_preds)*
            {}
        )
    }

    macro_rules! declare_field_initalizer {
        (
            type $selector:ident;
            trait $trait_name:ident[$self_:ident,$ErasedPtr:ident,$OrigPtr:ident]
            where [ $($where_preds:tt)* ]
            type=$field_ty:ty,
            value=$field_value:expr,
        ) => (
            pub trait $trait_name<$self_,$ErasedPtr,$OrigPtr>{
                const VALUE:Option<$field_ty>;
            }

            impl<$self_,$ErasedPtr,$OrigPtr> $trait_name<$self_,$ErasedPtr,$OrigPtr>
            for Unimplemented<trait_marker::$selector>
            {
                const VALUE:Option<$field_ty>=None;
            }

            impl<$self_,$ErasedPtr,$OrigPtr> $trait_name<$self_,ErasedPtr,$OrigPtr>
            for Implemented<trait_marker::$selector>
            where
                $($where_preds)*
            {
                const VALUE:Option<$field_ty>=Some($field_value);
            }
        )
    }

    declare_conditional_marker! {
        type Send;
        trait RequiresSend[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Send,OrigPtr:Send ]
    }

    declare_conditional_marker! {
        type Sync;
        trait RequiresSync[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Sync,OrigPtr:Sync ]
    }

    declare_field_initalizer! {
        type Clone;
        trait InitCloneField[_Self,ErasedPtr,OrigPtr]
        where [ OrigPtr:Clone ]
        type=unsafe extern "C" fn(this:RRef<'_, ErasedPtr>)->ErasedPtr,
        value=c_functions::clone_pointer_impl::<OrigPtr,ErasedPtr>,
    }
    declare_field_initalizer! {
        type Debug;
        trait InitDebugField[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Debug ]
        type=unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>,
        value=c_functions::debug_impl::<_Self>,
    }
    declare_field_initalizer! {
        type Display;
        trait InitDisplayField[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Display ]
        type=unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>,
        value=c_functions::display_impl::<_Self>,
    }
}
