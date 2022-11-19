use std::{
    cmp::{Ord, PartialEq, PartialOrd},
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};

use crate::{
    erased_types::{c_functions, trait_objects, FormattingMode, InterfaceType},
    inline_storage::InlineStorage,
    marker_type::{ErasedObject, UnsafeIgnoredType},
    nonexhaustive_enum::{
        alt_c_functions, EnumInfo, GetEnumInfo, GetSerializeEnumProxy, NonExhaustive, SerializeEnum,
    },
    prefix_type::{panic_on_missing_fieldname, WithMetadata},
    sabi_types::{RMut, RRef},
    std_types::{RBoxError, RCmpOrdering, ROption, RResult, RString},
    type_level::{
        impl_enum::{Implemented, Unimplemented},
        trait_marker,
    },
    utils::Transmuter,
    StableAbi,
};

#[doc(hidden)]
pub struct Private<T: ?Sized, S: ?Sized, I: ?Sized>(
    PhantomData<(PhantomData<T>, PhantomData<S>, PhantomData<I>)>,
);

/// Gets the vtable of `NonExhaustive<Self,S,I>`.
///
/// This trait is only exposed for use in bounds,
/// and cannot be implemented outside of `abi_stable`.
pub trait GetVTable<S, I>: Sized {
    // Using privacy to make it impossible to implement this trait outside this module.
    #[doc(hidden)]
    const __HIDDEN_10341423423__: Private<Self, S, I>;

    #[doc(hidden)]
    const VTABLE_VAL: NonExhaustiveVtable<Self, S, I>;

    staticref! {
        #[doc(hidden)]
        const VTABLE_WM: WithMetadata<NonExhaustiveVtable<Self,S,I>> =
            WithMetadata::new(Self::VTABLE_VAL)
    }

    /// The vtable
    const VTABLE: NonExhaustiveVtable_Ref<Self, S, I> =
        NonExhaustiveVtable_Ref(Self::VTABLE_WM.as_prefix());
}

/// The vtable for NonExhaustive.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    bound(I: GetSerializeEnumProxy<E>),
    bound(<I as GetSerializeEnumProxy<E>>::ProxyType: StableAbi),
    not_stableabi(E,S,I),
    missing_field(default),
    kind(Prefix(prefix_ref_docs = "\
        A reference to the vtable of a non-exhaustive enum,
    ")),
    with_field_indices,
    //debug_print,
)]
pub struct NonExhaustiveVtable<E, S, I> {
    pub(crate) _sabi_tys: UnsafeIgnoredType<(E, S, I)>,

    /// The `EnumInfo` for the enum.
    pub enum_info: &'static EnumInfo,

    pub(crate) _sabi_drop: unsafe extern "C" fn(this: RMut<'_, ErasedObject>),

    #[sabi(unsafe_opaque_field)]
    pub(crate) _sabi_clone: Option<
        unsafe extern "C" fn(
            RRef<'_, ErasedObject>,
            NonExhaustiveVtable_Ref<E, S, I>,
        ) -> NonExhaustive<E, S, I>,
    >,

    pub(crate) _sabi_debug: Option<
        unsafe extern "C" fn(
            RRef<'_, ErasedObject>,
            FormattingMode,
            &mut RString,
        ) -> RResult<(), ()>,
    >,
    pub(crate) _sabi_display: Option<
        unsafe extern "C" fn(
            RRef<'_, ErasedObject>,
            FormattingMode,
            &mut RString,
        ) -> RResult<(), ()>,
    >,
    #[sabi(unsafe_change_type =
        unsafe extern "C" fn(
            RRef<'_, ErasedObject>
        )->RResult< <I as GetSerializeEnumProxy<E>>::ProxyType, RBoxError>
    )]
    pub(crate) erased_sabi_serialize:
        Option<unsafe extern "C" fn(RRef<'_, ErasedObject>) -> RResult<ErasedObject, RBoxError>>,
    pub(crate) _sabi_partial_eq:
        Option<unsafe extern "C" fn(RRef<'_, ErasedObject>, RRef<'_, ErasedObject>) -> bool>,
    pub(crate) _sabi_cmp: Option<
        unsafe extern "C" fn(RRef<'_, ErasedObject>, RRef<'_, ErasedObject>) -> RCmpOrdering,
    >,
    pub(crate) _sabi_partial_cmp: Option<
        unsafe extern "C" fn(
            RRef<'_, ErasedObject>,
            RRef<'_, ErasedObject>,
        ) -> ROption<RCmpOrdering>,
    >,
    #[sabi(last_prefix_field)]
    pub(crate) _sabi_hash:
        Option<unsafe extern "C" fn(RRef<'_, ErasedObject>, trait_objects::HasherObject<'_>)>,
}

unsafe impl<E, S, I> Sync for NonExhaustiveVtable<E, S, I> {}
unsafe impl<E, S, I> Send for NonExhaustiveVtable<E, S, I> {}

impl<E, S, I> GetVTable<S, I> for E
where
    S: InlineStorage,
    I: InterfaceType,
    E: GetEnumInfo,
    I::Sync: RequiresSync<E, S, I>,
    I::Send: RequiresSend<E, S, I>,
    I::Clone: InitCloneField<E, S, I>,
    I::Debug: InitDebugField<E, S, I>,
    I::Display: InitDisplayField<E, S, I>,
    I::Serialize: InitSerializeField<E, S, I>,
    I::PartialEq: InitPartialEqField<E, S, I>,
    I::PartialOrd: InitPartialOrdField<E, S, I>,
    I::Ord: InitOrdField<E, S, I>,
    I::Hash: InitHashField<E, S, I>,
{
    const __HIDDEN_10341423423__: Private<Self, S, I> = Private(PhantomData);

    #[doc(hidden)]
    const VTABLE_VAL: NonExhaustiveVtable<E, S, I> = NonExhaustiveVtable {
        _sabi_tys: UnsafeIgnoredType::DEFAULT,
        enum_info: E::ENUM_INFO,
        _sabi_drop: alt_c_functions::drop_impl::<E>,
        _sabi_clone: <I::Clone as InitCloneField<E, S, I>>::VALUE,
        _sabi_debug: <I::Debug as InitDebugField<E, S, I>>::VALUE,
        _sabi_display: <I::Display as InitDisplayField<E, S, I>>::VALUE,
        erased_sabi_serialize: <I::Serialize as InitSerializeField<E, S, I>>::VALUE,
        _sabi_partial_eq: <I::PartialEq as InitPartialEqField<E, S, I>>::VALUE,
        _sabi_partial_cmp: <I::PartialOrd as InitPartialOrdField<E, S, I>>::VALUE,
        _sabi_cmp: <I::Ord as InitOrdField<E, S, I>>::VALUE,
        _sabi_hash: <I::Hash as InitHashField<E, S, I>>::VALUE,
    };
}

type UnerasedSerializeFn<E, I> =
    unsafe extern "C" fn(
        RRef<'_, ErasedObject>,
    ) -> RResult<<I as GetSerializeEnumProxy<E>>::ProxyType, RBoxError>;

impl<E, S, I> NonExhaustiveVtable_Ref<E, S, I> {
    pub(crate) fn serialize(self) -> UnerasedSerializeFn<E, I>
    where
        I: InterfaceType<Serialize = Implemented<trait_marker::Serialize>>,
        I: GetSerializeEnumProxy<E>,
    {
        unsafe {
            std::mem::transmute::<
                unsafe extern "C" fn(RRef<'_, ErasedObject>) -> RResult<ErasedObject, RBoxError>,
                UnerasedSerializeFn<E, I>,
            >(self.priv_serialize())
        }
    }
}

use self::trait_bounds::*;
pub mod trait_bounds {
    use super::*;

    macro_rules! declare_conditional_marker {
        (
            type $selector:ident;
            trait $trait_name:ident[$self_:ident,$Filler:ident,$OrigPtr:ident]
            where [ $($where_preds:tt)* ]
        ) => (
            pub trait $trait_name<$self_,$Filler,$OrigPtr>{}

            impl<$self_,$Filler,$OrigPtr> $trait_name<$self_,$Filler,$OrigPtr>
            for Unimplemented<trait_marker::$selector>
            {}

            impl<$self_,$Filler,$OrigPtr> $trait_name<$self_,$Filler,$OrigPtr>
            for Implemented<trait_marker::$selector>
            where
                $($where_preds)*
            {}
        )
    }

    macro_rules! declare_field_initalizer {
        (
            type $selector:ident;
            trait $trait_name:ident[$enum_:ident,$filler:ident,$interf:ident]
            $( where_for_both[ $($where_preds_both:tt)* ] )?
            where [ $($where_preds:tt)* ]
            $priv_field:ident,$field:ident : $field_ty:ty;
            field_index=$field_index:ident;
            value=$field_value:expr,
        ) => (
            pub trait $trait_name<$enum_,$filler,$interf>
            where
                $($($where_preds_both)*)?
            {
                const VALUE:Option<$field_ty>;
            }

            impl<$enum_,$filler,$interf> $trait_name<$enum_,$filler,$interf>
            for Unimplemented<trait_marker::$selector>
            where
                $($($where_preds_both)*)?
            {
                const VALUE:Option<$field_ty>=None;
            }

            impl<$enum_,$filler,$interf> $trait_name<$enum_,$filler,$interf>
            for Implemented<trait_marker::$selector>
            where
                $($($where_preds_both)*)?
                $($where_preds)*
            {
                const VALUE:Option<$field_ty>=Some($field_value);
            }

            impl<E,S,$interf> NonExhaustiveVtable_Ref<E,S,$interf>{
                #[doc = concat!(
                    "Fallibly accesses the `",
                    stringify!($field),
                    "` field, panicking if it doesn't exist."
                )]

                pub fn $field(self) -> $field_ty
                where
                    $interf:InterfaceType<$selector=Implemented<trait_marker::$selector>>,
                {
                    match self.$priv_field().into() {
                        Some(v)=>v,
                        None=>panic_on_missing_fieldname::<
                            NonExhaustiveVtable<E,S,$interf>,
                        >(
                            Self::$field_index,
                            self._prefix_type_layout(),
                        )
                    }
                }
            }
        )
    }

    declare_conditional_marker! {
        type Send;
        trait RequiresSend[E,S,I]
        where [ E:Send ]
    }

    declare_conditional_marker! {
        type Sync;
        trait RequiresSync[E,S,I]
        where [ E:Sync ]
    }

    declare_field_initalizer! {
        type Clone;
        trait InitCloneField[E,S,I]
        where_for_both[ E:GetEnumInfo, ]
        where [ E:Clone ]
        _sabi_clone,clone_:
            unsafe extern "C" fn(
                RRef<'_, ErasedObject>,
                NonExhaustiveVtable_Ref<E,S,I>
            )->NonExhaustive<E,S,I>;
        field_index=field_index_for__sabi_clone;
        value=alt_c_functions::clone_impl::<E,S,I>,
    }
    declare_field_initalizer! {
        type Debug;
        trait InitDebugField[E,S,I]
        where [ E:Debug ]
        _sabi_debug,debug:
            unsafe extern "C" fn(
                RRef<'_, ErasedObject>,
                FormattingMode,
                &mut RString,
            )->RResult<(),()>;
        field_index=field_index_for__sabi_debug;
        value=c_functions::debug_impl::<E>,
    }
    declare_field_initalizer! {
        type Display;
        trait InitDisplayField[E,S,I]
        where [ E:Display ]
        _sabi_display,display:
            unsafe extern "C" fn(
                RRef<'_, ErasedObject>,
                FormattingMode,
                &mut RString,
            )->RResult<(),()>;
        field_index=field_index_for__sabi_display;
        value=c_functions::display_impl::<E>,
    }
    declare_field_initalizer! {
        type Serialize;
        trait InitSerializeField[E,S,I]
        where [ I:SerializeEnum<E> ]
        erased_sabi_serialize,priv_serialize:
            unsafe extern "C" fn(RRef<'_, ErasedObject>)->RResult<ErasedObject,RBoxError>;
        field_index=field_index_for_erased_sabi_serialize;
        value=unsafe{
            Transmuter::<
                unsafe extern "C" fn(
                    RRef<'_, ErasedObject>
                )->RResult<<I as SerializeEnum<E>>::Proxy,RBoxError>,
                unsafe extern "C" fn(RRef<'_, ErasedObject>)->RResult<ErasedObject,RBoxError>
            >{
                from:alt_c_functions::serialize_impl::<E,I>
            }.to
        },
    }
    declare_field_initalizer! {
        type PartialEq;
        trait InitPartialEqField[E,S,I]
        where_for_both[ E:GetEnumInfo, ]
        where [ E:PartialEq ]
        _sabi_partial_eq,partial_eq: unsafe extern "C" fn(RRef<'_, ErasedObject>,RRef<'_, ErasedObject>)->bool;
        field_index=field_index_for__sabi_partial_eq;
        value=alt_c_functions::partial_eq_impl::<E,S,I>,
    }
    declare_field_initalizer! {
        type PartialOrd;
        trait InitPartialOrdField[E,S,I]
        where_for_both[ E:GetEnumInfo, ]
        where [ E:PartialOrd ]
        _sabi_partial_cmp,partial_cmp:
            unsafe extern "C" fn(RRef<'_, ErasedObject>,RRef<'_, ErasedObject>)->ROption<RCmpOrdering>;
        field_index=field_index_for__sabi_partial_cmp;
        value=alt_c_functions::partial_cmp_ord::<E,S,I>,
    }
    declare_field_initalizer! {
        type Ord;
        trait InitOrdField[E,S,I]
        where_for_both[ E:GetEnumInfo, ]
        where [ E:Ord ]
        _sabi_cmp,cmp:
            unsafe extern "C" fn(
                RRef<'_, ErasedObject>,
                RRef<'_, ErasedObject>,
            )->RCmpOrdering;
        field_index=field_index_for__sabi_cmp;
        value=alt_c_functions::cmp_ord::<E,S,I>,
    }
    declare_field_initalizer! {
        type Hash;
        trait InitHashField[E,S,I]
        where [ E:Hash ]
        _sabi_hash,hash: unsafe extern "C" fn(RRef<'_, ErasedObject>,trait_objects::HasherObject<'_>);
        field_index=field_index_for__sabi_hash;
        value=c_functions::hash_Hash::<E>,
    }
}
