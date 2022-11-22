//! Contains `DynTrait<_>`'s vtable,and related types/traits.
#![allow(missing_docs)]

use std::{
    fmt::{self, Debug, Write as FmtWrite},
    io,
    marker::PhantomData,
};

use super::{
    c_functions::*,
    iterator::{DoubleEndedIteratorFns, IteratorFns, MakeDoubleEndedIteratorFns, MakeIteratorFns},
    traits::{GetSerializeProxyType, IteratorItemOrDefault, SerializeType},
    type_info::TypeInfoFor,
    *,
};

use crate::{
    marker_type::{ErasedObject, NonOwningPhantom},
    pointer_trait::GetPointerKind,
    prefix_type::{panic_on_missing_fieldname, WithMetadata},
    sabi_types::{RMut, RRef, StaticRef},
    std_types::{RIoError, RNone, RSeekFrom, RSome},
    type_level::{
        downcasting::GetUTID,
        impl_enum::{Implemented, Unimplemented},
        trait_marker,
    },
    utils::Transmuter,
    InterfaceType, StableAbi,
};

/// Csontructs a vtable.
///
/// This is only exposed to allow users to construct
/// [`DynTrait`]s with a generic `I` type parameter.
pub trait MakeVTable<'borr, T, OrigPtr, CanDowncast> {
    type Helper0;

    const HELPER0: Self::Helper0;

    type Helper1;

    const HELPER1: Self::Helper1;

    const VTABLE_REF: Self;
}

macro_rules! declare_meta_vtable {
    (
        interface=$interf:ident;
        value=$value:ident;
        erased_pointer=$erased_ptr:ident;
        original_pointer=$orig_ptr:ident;
        can_downcast=$can_downcast:ident;

        auto_traits[
            $([
                impl $auto_trait:ident ($auto_trait_path:path)
                where [ $($phantom_where_clause:tt)* ]
                query_fn = $auto_trait_query:ident;
            ])*
        ]

        marker_traits[
            $([
                impl $marker_trait:ident($marker_trait_path:path)
                where [ $($marker_where_clause:tt)* ]
                query_fn = $marker_trait_query:ident;
            ])*
        ]

        $([
            $( #[$field_attr:meta] )*
            $field:ident : $field_ty:ty ;
            priv $priv_field:ident;
            option=$option_ty:ident,$some_constr:ident,$none_constr:ident;
            field_index=$field_index:ident;
            query_fn = $trait_query:ident;

            $(struct_bound=$struct_bound:expr;)*

            impl[$($impl_params:tt)*] VtableFieldValue<$selector:ident($trait_path:path)>
            where [ $($where_clause:tt)* ]
            { $field_value:expr }
        ])*
    ) => (

        /// This is the vtable for DynTrait<_>,
        ///
        #[repr(C,align(16))]
        #[derive(StableAbi)]
        #[sabi(
            // debug_print,
            with_field_indices,
            kind(Prefix(prefix_ref_docs = "\
                A pointer to the vtable of [`DynTrait`].\
                \n\n\
                This is only exposed to allow users to construct\
                [`DynTrait`]s with a generic `I` type parameter.\
            ")),
            missing_field(panic),
            prefix_bound(I: InterfaceType),
            bound(I: IteratorItemOrDefault<'borr>),
            bound(<I as IteratorItemOrDefault<'borr>>::Item: StableAbi),
            bound(I: GetSerializeProxyType<'borr>),
            bound(<I as GetSerializeProxyType<'borr>>::ProxyType: StableAbi),
            $($(bound=$struct_bound,)*)*
        )]
        pub struct VTable<'borr,$erased_ptr,$interf>{
            pub type_info:&'static TypeInfo,
            _marker:NonOwningPhantom<($erased_ptr,$interf,&'borr())>,
            pub drop_ptr:unsafe extern "C" fn(RMut<'_, $erased_ptr>),
            $(
                $( #[$field_attr] )*
                $priv_field:$option_ty<$field_ty>,
            )*
        }


        impl<'borr,$erased_ptr,$interf> VTable_Ref<'borr,$erased_ptr,$interf>{
            $(
                pub fn $field(&self)->$field_ty
                where
                    $interf:InterfaceType<$selector=Implemented<trait_marker::$selector>>,
                {
                    match self.$priv_field().into() {
                        Some(v)=>v,
                        None=>panic_on_missing_fieldname::<
                            VTable<'borr,$erased_ptr,$interf>,
                        >(
                            Self::$field_index,
                            self._prefix_type_layout(),
                        )
                    }
                }
            )*
            pub fn iter(
                &self
            )->IteratorFns< <I as IteratorItemOrDefault<'borr>>::Item >
            where
                $interf:InterfaceType<Iterator=Implemented<trait_marker::Iterator>>,
                $interf:IteratorItemOrDefault<'borr>,
            {
                unsafe{
                    std::mem::transmute::<
                        IteratorFns< () >,
                        IteratorFns< <I as IteratorItemOrDefault<'borr>>::Item >
                    >( self.erased_iter() )
                }
            }

            pub fn back_iter(
                &self
            )->DoubleEndedIteratorFns< <I as IteratorItemOrDefault<'borr>>::Item >
            where
                $interf:InterfaceType<
                    DoubleEndedIterator=Implemented<trait_marker::DoubleEndedIterator>
                >,
                $interf:IteratorItemOrDefault<'borr>,
            {
                unsafe{
                    std::mem::transmute::<
                        DoubleEndedIteratorFns< () >,
                        DoubleEndedIteratorFns< <I as IteratorItemOrDefault<'borr>>::Item >
                    >( self.erased_back_iter() )
                }
            }

            pub fn serialize<'s>(&self)->UnerasedSerializeFn<'s,I>
            where
                I:InterfaceType<Serialize=Implemented<trait_marker::Serialize>>,
                I:GetSerializeProxyType<'s>,
            {
                unsafe{
                    std::mem::transmute::<
                        unsafe extern "C" fn(RRef<'_, ErasedObject>)->RResult<ErasedObject,RBoxError>,
                        UnerasedSerializeFn<'s,I>,
                    >( self.erased_serialize() )
                }
            }
        }


        pub type UnerasedSerializeFn<'s,I>=
            unsafe extern "C" fn(
                RRef<'s, ErasedObject>
            )->RResult<<I as GetSerializeProxyType<'s>>::ProxyType,RBoxError>;


        /// Returns the type of a vtable field.
        pub type VTableFieldType<'borr,Selector,$value,$erased_ptr,$orig_ptr,$interf>=
            <Selector as VTableFieldType_<'borr,$value,$erased_ptr,$orig_ptr,$interf>>::Field;

        /// Returns the type of a vtable field.
        pub trait VTableFieldType_<'borr,$value,$erased_ptr,$orig_ptr,$interf>{
            type Field;
        }

        /// Returns the value of a vtable field in the current binary
        /// (this can be a different value in a dynamically_linked_library/executable).
        pub trait VTableFieldValue<'borr,Ty,$value,$erased_ptr,$orig_ptr,$interf>{
            const FIELD:Ty;
        }

        pub trait MarkerTrait<'borr,$value,$erased_ptr,$orig_ptr>{}


        $(
            impl<'borr,$value,$erased_ptr,$orig_ptr,$interf>
                VTableFieldType_<'borr,$value,$erased_ptr,$orig_ptr,$interf>
            for trait_marker::$selector
            where
                $interf:InterfaceType,
            {
                type Field=$field_ty;
            }


            impl<'borr,AnyFieldTy,$value,$erased_ptr,$orig_ptr,$interf>
                VTableFieldValue<
                    'borr,
                    $option_ty<AnyFieldTy>,
                    $value,
                    $erased_ptr,
                    $orig_ptr,
                    $interf
                >
            for Unimplemented<trait_marker::$selector>
            {
                const FIELD:$option_ty<AnyFieldTy>=$none_constr;
            }

            impl<'borr,$value,$erased_ptr,$orig_ptr,$interf,$($impl_params)*>
                VTableFieldValue<
                    'borr,
                    $option_ty<$field_ty>,
                    $value,
                    $erased_ptr,
                    $orig_ptr,
                    $interf
                >
            for Implemented<trait_marker::$selector>
            where
                $interf:InterfaceType,
                $($where_clause)*
            {
                const FIELD:$option_ty<$field_ty>=
                    $some_constr($field_value);
            }
        )*



        impl<'borr,Anything,$value,$erased_ptr,$orig_ptr>
            MarkerTrait<'borr,$value,$erased_ptr,$orig_ptr>
        for Unimplemented<Anything>
        {}

        $(
            impl<'borr,$value,$erased_ptr,$orig_ptr>
                MarkerTrait<'borr,$value,$erased_ptr,$orig_ptr>
            for Implemented<trait_marker::$auto_trait>
            where $($phantom_where_clause)*
            {}
        )*

        $(
            impl<'borr,$value,$erased_ptr,$orig_ptr>
                MarkerTrait<'borr,$value,$erased_ptr,$orig_ptr>
            for Implemented<trait_marker::$marker_trait>
            where $($marker_where_clause)*
            {}
        )*

        ///////////////////////////////////////////////////////////

        impl<'borr,$value,$erased_ptr,$orig_ptr,$interf,$can_downcast>
            MakeVTable<'borr,$value,$orig_ptr,$can_downcast>
        for VTable_Ref<'borr,$erased_ptr,$interf>
        where
            $interf:InterfaceType,
            $can_downcast: GetUTID<$value>,
            $(
                $interf::$auto_trait:
                    MarkerTrait<'borr,$value,$erased_ptr,$orig_ptr>,
            )*
            $(
                $interf::$marker_trait:
                    MarkerTrait<'borr,$value,$erased_ptr,$orig_ptr>,
            )*
            $(
                $interf::$selector: VTableFieldValue<
                    'borr,
                    $option_ty<$field_ty>,
                    $value,
                    $erased_ptr,
                    $orig_ptr,
                    $interf,
                >,
            )*
        {
            #[doc(hidden)]
            type Helper0 = WithMetadata<VTable<'borr,$erased_ptr,$interf>>;

            #[doc(hidden)]
            const HELPER0: Self::Helper0 = WithMetadata::new(VTable{
                type_info: <TypeInfoFor<$value, $interf, $can_downcast>>::INFO,
                drop_ptr:drop_pointer_impl::<$orig_ptr,$erased_ptr>,
                $(
                    $priv_field:
                        <$interf::$selector as
                            VTableFieldValue<
                                $option_ty<VTableFieldType<
                                    'borr,
                                    trait_marker::$selector,
                                    $value,
                                    $erased_ptr,
                                    $orig_ptr,
                                    $interf,
                                >>,
                                $value,
                                $erased_ptr,
                                $orig_ptr,
                                $interf,
                            >
                        >::FIELD,
                )*
                _marker:NonOwningPhantom::NEW,
            });

            #[doc(hidden)]
            type Helper1 = StaticRef<WithMetadata<VTable<'borr,ErasedPtr,I>>>;

            #[doc(hidden)]
            const HELPER1: Self::Helper1 = unsafe {
                // relying on static promotion, this will compile-error otherwise
                StaticRef::from_raw(
                    &<Self as MakeVTable<'borr,$value,$orig_ptr,$can_downcast>>::HELPER0
                )
            };

            const VTABLE_REF: Self = Self(
                <Self as MakeVTable<'borr,$value,$orig_ptr,$can_downcast>>::HELPER1.as_prefix()
            );

        }

        /// For constructing a [`RequiredTraits`] constant.
        #[allow(non_upper_case_globals)]
        pub trait MakeRequiredTraits: InterfaceType {
            #[doc(hidden)]
            // Used to prevent users from implementing this trait.
            const __MakeRequiredTraits_BLANKET_IMPL: PrivStruct<Self>;

            /// Describes which traits are required by `Self: `[`InterfaceType`],
            const MAKE: RequiredTraits = RequiredTraits::new::<Self>();
        }

        #[allow(non_upper_case_globals)]
        impl<I> MakeRequiredTraits for I
        where
            I:InterfaceType,
        {
            const __MakeRequiredTraits_BLANKET_IMPL:PrivStruct<Self>=
                PrivStruct(PhantomData);
        }


        impl<'borr,$erased_ptr,$interf> Debug for VTable_Ref<'borr,$erased_ptr,$interf>
        where
            $interf:InterfaceType,
        {
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
                f.debug_struct("VTable_Ref")
                    .field("type_info",&self.type_info())
                    .finish()
            }
        }

        declare_enabled_traits!{
            auto_traits[
                $(($auto_trait, $auto_trait_query, $auto_trait_path),)*
            ]

            regular_traits[
                $(($marker_trait, $marker_trait_query, $marker_trait_path),)*
                $(($selector, $trait_query, $trait_path),)*
                (Deserialize, contains_deserialize, serde::Deserialize),
            ]
        }
    )
}

declare_meta_vtable! {
    interface=I;
    value  =T;
    erased_pointer=ErasedPtr;
    original_pointer=OrigP;
    can_downcast = CanDowncast;

    auto_traits[
        [
            impl Send(std::marker::Send) where [OrigP:Send, T:Send]
            query_fn = contains_send;
        ]
        [
            impl Sync(std::marker::Sync) where [OrigP:Sync, T:Sync]
            query_fn = contains_sync;
        ]
        [
            impl Unpin(std::marker::Unpin) where [T: Unpin]
            query_fn = contains_unpin;
        ]
    ]

    marker_traits[
        [
            impl Error(std::error::Error) where [T:std::error::Error]
            query_fn = contains_error;
        ]
    ]

    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_clone())]
        clone_ptr:    unsafe extern "C" fn(RRef<'_, ErasedPtr>)->ErasedPtr;
        priv _clone_ptr;
        option=Option,Some,None;
        field_index=field_index_for__clone_ptr;
        query_fn = contains_clone;

        impl[] VtableFieldValue<Clone(std::clone::Clone)>
        where [OrigP:Clone]
        {
            clone_pointer_impl::<OrigP,ErasedPtr>
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_default())]
        default_ptr: unsafe extern "C" fn()->ErasedPtr ;
        priv _default_ptr;
        option=Option,Some,None;
        field_index=field_index_for__default_ptr;
        query_fn = contains_default;

        impl[] VtableFieldValue<Default(std::default::Default)>
        where [
            OrigP:GetPointerKind,
            OrigP:DefaultImpl<<OrigP as GetPointerKind>::Kind>,
        ]{
            default_pointer_impl::<OrigP,ErasedPtr>
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_display())]
        display:unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>;
        priv _display;
        option=Option,Some,None;
        field_index=field_index_for__display;
        query_fn = contains_display;

        impl[] VtableFieldValue<Display(std::fmt::Display)>
        where [T:Display]
        {
            display_impl::<T>
        }
    ]
    [
    #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_debug())]
        debug:unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>;
        priv _debug;
        option=Option,Some,None;
        field_index=field_index_for__debug;
        query_fn = contains_debug;

        impl[] VtableFieldValue<Debug(std::fmt::Debug)>
        where [T:Debug]
        {
            debug_impl::<T>
        }
    ]
    [
        #[sabi(unsafe_change_type=
            for<'s>
            unsafe extern "C" fn(
                RRef<'s, ErasedObject>
            )->RResult<<I as GetSerializeProxyType<'s>>::ProxyType,RBoxError>
        )]
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_serialize())]
        erased_serialize:unsafe extern "C" fn(RRef<'_, ErasedObject>)->RResult<ErasedObject,RBoxError>;
        priv priv_serialize;
        option=Option,Some,None;
        field_index=field_index_for_priv_serialize;
        query_fn = contains_serialize;

        impl[] VtableFieldValue<Serialize(serde::Serialize)>
        where [
            T:for<'s>SerializeType<'s,Interface=I>,
            I:for<'s>SerializeProxyType<'s>,
        ]{
            unsafe{
                Transmuter::<
                    unsafe extern "C" fn(
                        RRef<'_, ErasedObject>
                    )->RResult<<I as SerializeProxyType<'_>>::Proxy,RBoxError>,
                    unsafe extern "C" fn(RRef<'_, ErasedObject>)->RResult<ErasedObject,RBoxError>
                >{
                    from:serialize_impl::<T,I>
                }.to
            }
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_partial_eq())]
        partial_eq: unsafe extern "C" fn(RRef<'_, ErasedObject>,RRef<'_, ErasedObject>)->bool;
        priv _partial_eq;
        option=Option,Some,None;
        field_index=field_index_for__partial_eq;
        query_fn = contains_partial_eq;

        impl[] VtableFieldValue<PartialEq(std::cmp::PartialEq)>
        where [T:PartialEq,]
        {
            partial_eq_impl::<T>
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_cmp())]
        cmp:        unsafe extern "C" fn(RRef<'_, ErasedObject>,RRef<'_, ErasedObject>)->RCmpOrdering;
        priv _cmp;
        option=Option,Some,None;
        field_index=field_index_for__cmp;
        query_fn = contains_cmp;

        impl[] VtableFieldValue<Ord(std::cmp::Ord)>
        where [T:Ord,]
        {
            cmp_ord::<T>
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_partial_cmp())]
        partial_cmp:unsafe extern "C" fn(RRef<'_, ErasedObject>,RRef<'_, ErasedObject>)->ROption<RCmpOrdering>;
        priv _partial_cmp;
        option=Option,Some,None;
        field_index=field_index_for__partial_cmp;
        query_fn = contains_partial_cmp;

        impl[] VtableFieldValue<PartialOrd(std::cmp::PartialOrd)>
        where [T:PartialOrd,]
        {
            partial_cmp_ord::<T>
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_hash())]
        hash:unsafe extern "C" fn(RRef<'_, ErasedObject>,trait_objects::HasherObject<'_>);
        priv _hash;
        option=Option,Some,None;
        field_index=field_index_for__hash;
        query_fn = contains_hash;

        impl[] VtableFieldValue<Hash(std::hash::Hash)>
        where [T:Hash]
        {
            hash_Hash::<T>
        }
    ]
    [
        #[sabi(
            unsafe_change_type=
            ROption<IteratorFns< <I as IteratorItemOrDefault<'borr>>::Item >>
        )]
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_iterator())]
        erased_iter:IteratorFns< () >;
        priv _iter;
        option=ROption,RSome,RNone;
        field_index=field_index_for__iter;
        query_fn = contains_iterator;

        impl[] VtableFieldValue<Iterator(std::iter::Iterator)>
        where [
            T:Iterator,
            I:IteratorItemOrDefault<'borr,Item=<T as Iterator>::Item>,
        ]{
            MakeIteratorFns::<T>::NEW
        }
    ]
    [
        #[sabi(
            unsafe_change_type=
            ROption<DoubleEndedIteratorFns< <I as IteratorItemOrDefault<'borr>>::Item >>
        )]
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_double_ended_iterator())]
        erased_back_iter:DoubleEndedIteratorFns< () >;
        priv _back_iter;
        option=ROption,RSome,RNone;
        field_index=field_index_for__back_iter;
        query_fn = contains_double_ended_iterator;

        impl[] VtableFieldValue<DoubleEndedIterator(std::iter::DoubleEndedIterator)>
        where [
            T:DoubleEndedIterator,
            I:IteratorItemOrDefault<'borr,Item=<T as Iterator>::Item>,
        ]{
            MakeDoubleEndedIteratorFns::<T>::NEW
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_fmt_write())]
        fmt_write_str:unsafe extern "C" fn(RMut<'_, ErasedObject>,RStr<'_>)->RResult<(),()>;
        priv _fmt_write_str;
        option=Option,Some,None;
        field_index=field_index_for__fmt_write_str;
        query_fn = contains_fmt_write;

        impl[] VtableFieldValue<FmtWrite(std::fmt::Write)>
        where [ T:FmtWrite ]
        {
            write_str_fmt_write::<T>
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_io_write())]
        io_write:IoWriteFns;
        priv _io_write;
        option=ROption,RSome,RNone;
        field_index=field_index_for__io_write;
        query_fn = contains_io_write;

        impl[] VtableFieldValue<IoWrite(std::io::Write)>
        where [ T:io::Write ]
        {
            MakeIoWriteFns::<T>::NEW
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_io_read())]
        io_read:IoReadFns;
        priv _io_read;
        option=ROption,RSome,RNone;
        field_index=field_index_for__io_read;
        query_fn = contains_io_read;

        impl[] VtableFieldValue<IoRead(std::io::Read)>
        where [ T:io::Read ]
        {
            MakeIoReadFns::<T>::NEW
        }
    ]
    [
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_io_bufread())]
        io_bufread:IoBufReadFns;
        priv _io_bufread;
        option=ROption,RSome,RNone;
        field_index=field_index_for__io_bufread;
        query_fn = contains_io_bufread;

        impl[] VtableFieldValue<IoBufRead(std::io::BufRead)>
        where [
            T:io::BufRead,
            I:InterfaceType<IoRead= Implemented<trait_marker::IoRead>>
        ]{
            MakeIoBufReadFns::<T>::NEW
        }
    ]
    [
        #[sabi(last_prefix_field)]
        #[sabi(accessible_if= <I as MakeRequiredTraits>::MAKE.contains_io_seek())]
        io_seek:unsafe extern "C" fn(RMut<'_, ErasedObject>,RSeekFrom)->RResult<u64,RIoError>;
        priv _io_seek;
        option=Option,Some,None;
        field_index=field_index_for__io_seek;
        query_fn = contains_io_seek;

        impl[] VtableFieldValue<IoSeek(std::io::Seek)>
        where [ T:io::Seek ]
        {
            io_Seek_seek::<T>
        }
    ]
}

//////////////

/// Used to prevent MakeRequiredTraits being implemented outside this module,
/// since it is only constructed in the impl of MakeRequiredTraits in this module.
#[doc(hidden)]
pub struct PrivStruct<T>(PhantomData<T>);
