/*!

Contains `DynTrait<_>`'s vtable,and related types/traits.

*/
use std::{
    fmt::{self, Debug,Write as FmtWrite},
    io,
    marker::PhantomData,
};

use super::{
    *,
    c_functions::*,
    iterator::{
        IteratorFns,MakeIteratorFns,
        DoubleEndedIteratorFns,MakeDoubleEndedIteratorFns,
    },
    traits::{
        IteratorItemOrDefault,InterfaceFor,
        SerializeImplType,GetSerializeProxyType,
    },
};

use crate::{
    StableAbi,
    marker_type::{ErasedObject,NonOwningPhantom},
    prefix_type::{PrefixTypeTrait,WithMetadata,panic_on_missing_fieldname},
    pointer_trait::{AsPtr, GetPointerKind,CanTransmuteElement},
    sabi_types::{RRef, RMut},
    std_types::{RSome,RNone,RIoError,RSeekFrom},
    type_level::{
        impl_enum::{Implemented,Unimplemented,Implementability},
        trait_marker,
    },
    utils::Transmuter,
};


use core_extensions::TypeIdentity;





/// Returns the vtable used by DynTrait to do dynamic dispatch.
pub trait GetVtable<'borr,This,ErasedPtr,OrigPtr,I:InterfaceBound> {
    
    #[doc(hidden)]
    const TMP_VTABLE:VTable<'borr,ErasedPtr,I>;


    staticref!{
        #[doc(hidden)]
        const _WM_VTABLE: WithMetadata<VTable<'borr,ErasedPtr,I>> = 
            WithMetadata::new(PrefixTypeTrait::METADATA, Self::TMP_VTABLE)
    }

    #[doc(hidden)]
    const _GET_INNER_VTABLE:VTable_Ref<'borr,ErasedPtr,I>=unsafe{
        VTable_Ref(Self::_WM_VTABLE.as_prefix())
    };

}


/// A helper type for constructing a `DynTrait` at compile-time,
/// by passing `VTableDT::GET` to `DynTrait::from_const`.
#[repr(transparent)]
pub struct VTableDT<'borr,T,ErasedPtr,OrigPtr,I,Downcasting>{
    pub(super) vtable:VTable_Ref<'borr,ErasedPtr,I>,
    _for:NonOwningPhantom<(T,OrigPtr,Downcasting)>
}

impl<'borr,T,ErasedPtr,OrigPtr,I,Downcasting> Copy 
    for VTableDT<'borr,T,ErasedPtr,OrigPtr,I,Downcasting>
{}

impl<'borr,T,ErasedPtr,OrigPtr,I,Downcasting> Clone
    for VTableDT<'borr,T,ErasedPtr,OrigPtr,I,Downcasting>
{
    fn clone(&self)->Self{
        *self
    }
}

impl<'borr,T,ErasedPtr,OrigPtr,I,Downcasting> 
    VTableDT<'borr,T,ErasedPtr,OrigPtr,I,Downcasting>
where
    OrigPtr: CanTransmuteElement<(), PtrTarget = T, TransmutedPtr=ErasedPtr>,
    ErasedPtr: AsPtr<PtrTarget=()>,
    I: InterfaceBound,
    InterfaceFor<T,I,Downcasting>: GetVtable<'borr,T,ErasedPtr,OrigPtr,I>,
{    
    /// Constructs a `VTableDT`.
    pub const GET:Self=Self{
        vtable:<
            InterfaceFor<T,I,Downcasting> as 
            GetVtable<'borr,T,ErasedPtr,OrigPtr,I>
        >::_GET_INNER_VTABLE,
        _for:NonOwningPhantom::NEW,
    };
}


macro_rules! declare_meta_vtable {
    (
        interface=$interf:ident;
        value=$value:ident;
        erased_pointer=$erased_ptr:ident;
        original_pointer=$orig_ptr:ident;

        auto_traits[
            $([
                impl $auto_trait:ident where [ $($phantom_where_clause:tt)* ]
            ])*
        ]

        marker_traits[
            $([
                impl $marker_trait:ident where [ $($marker_where_clause:tt)* ]
            ])*
        ]

        $([
            $( #[$field_attr:meta] )*
            $field:ident : $field_ty:ty ;
            priv $priv_field:ident;
            option=$option_ty:ident,$some_constr:ident,$none_constr:ident;
            field_index=$field_index:ident;

            $(struct_bound=$struct_bound:expr;)*
            
            impl[$($impl_params:tt)*] VtableFieldValue<$selector:ident>
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
            kind(Prefix),
            missing_field(panic),
            prefix_bound="I:InterfaceBound",
            bound="I:IteratorItemOrDefault<'borr>",
            bound="<I as IteratorItemOrDefault<'borr>>::Item:StableAbi",
            bound="I: GetSerializeProxyType<'borr>",
            bound="<I as GetSerializeProxyType<'borr>>::ProxyType:StableAbi",
            // bound="I:for<'s> GetSerializeProxyType<'s>",
            // bound="for<'s> <I as GetSerializeProxyType<'s>>::ProxyType:StableAbi",
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
                    $interf:InterfaceBound<$selector=Implemented<trait_marker::$selector>>,
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
                $interf:InterfaceBound<Iterator=Implemented<trait_marker::Iterator>>,
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
                $interf:InterfaceBound<
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
                I:InterfaceBound<Serialize=Implemented<trait_marker::Serialize>>,
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
        pub trait VTableFieldValue<'borr,Ty,IsImpld,$value,$erased_ptr,$orig_ptr,$interf>{
            const FIELD:Ty;
        }

        pub trait MarkerTrait<'borr,IsImpld,$value,$erased_ptr,$orig_ptr>{}


        $(
            impl<'borr,$value,$erased_ptr,$orig_ptr,$interf> 
                VTableFieldType_<'borr,$value,$erased_ptr,$orig_ptr,$interf> 
            for trait_selector::$selector 
            where 
                $interf:InterfaceBound,
            {
                type Field=$field_ty;
            }

            
            impl<'borr,AnyFieldTy,$value,$erased_ptr,$orig_ptr,$interf>
                VTableFieldValue<
                    'borr,
                    $option_ty<AnyFieldTy>,
                    Unimplemented<trait_marker::$selector>,
                    $value,
                    $erased_ptr,
                    $orig_ptr,
                    $interf
                >
            for trait_selector::$selector
            {
                const FIELD:$option_ty<AnyFieldTy>=$none_constr;
            }

            impl<'borr,FieldTy,$value,$erased_ptr,$orig_ptr,$interf,$($impl_params)*>
                VTableFieldValue<
                    'borr,
                    $option_ty<FieldTy>,
                    Implemented<trait_marker::$selector>,
                    $value,
                    $erased_ptr,
                    $orig_ptr,
                    $interf
                >
            for trait_selector::$selector
            where 
                $interf:InterfaceBound,
                $field_ty:TypeIdentity<Type=FieldTy>,
                FieldTy:Copy,
                $($where_clause)*
            {
                const FIELD:$option_ty<FieldTy>=
                    $some_constr(type_identity!($field_ty=>FieldTy;$field_value));
            }
        )*



        impl<'borr,Anything,$value,X,$erased_ptr,$orig_ptr> 
            MarkerTrait<'borr,Unimplemented<X>,$value,$erased_ptr,$orig_ptr> 
        for Anything
        {}

        $(
            impl<'borr,$value,X,$erased_ptr,$orig_ptr> 
                MarkerTrait<'borr,Implemented<X>,$value,$erased_ptr,$orig_ptr> 
            for trait_selector::$auto_trait
            where $($phantom_where_clause)*
            {}
        )*

        $(
            impl<'borr,$value,X,$erased_ptr,$orig_ptr> 
                MarkerTrait<'borr,Implemented<X>,$value,$erased_ptr,$orig_ptr> 
            for trait_selector::$marker_trait
            where $($marker_where_clause)*
            {}
        )*

        ///////////////////////////////////////////////////////////

        /// Contains marker types representing traits of the same name.
        pub mod trait_selector{
            $(
                /// Marker type representing the trait of the same name.
                pub struct $auto_trait;
            )*
            $(
                /// Marker type representing the trait of the same name.
                pub struct $marker_trait;
            )*
            $(
                /// Marker type representing the trait of the same name.
                pub struct $selector;
            )*
        }


        impl<'borr,This,$value,$erased_ptr,$orig_ptr,$interf> 
            GetVtable<'borr,$value,$erased_ptr,$orig_ptr,$interf>
        for This
        where
            This:ImplType<Interface=$interf>,
            $interf:InterfaceBound,
            $(
                trait_selector::$auto_trait:
                    MarkerTrait<'borr,$interf::$auto_trait,$value,$erased_ptr,$orig_ptr>,
            )*
            $(
                trait_selector::$marker_trait:
                    MarkerTrait<'borr,$interf::$marker_trait,$value,$erased_ptr,$orig_ptr>,
            )*
            $(
                trait_selector::$selector:VTableFieldValue<
                    'borr,
                    $option_ty<$field_ty>,
                    $interf::$selector,
                    $value,
                    $erased_ptr,
                    $orig_ptr,
                    $interf,
                >,
            )*
        {
            const TMP_VTABLE:VTable<'borr,$erased_ptr,$interf>=VTable{
                type_info:This::INFO,
                drop_ptr:drop_pointer_impl::<$orig_ptr,$erased_ptr>,
                $(
                    $priv_field:
                        <trait_selector::$selector as
                            VTableFieldValue<
                                $option_ty<VTableFieldType<
                                    'borr,
                                    trait_selector::$selector,
                                    $value,
                                    $erased_ptr,
                                    $orig_ptr,
                                    $interf,
                                >>,
                                $interf::$selector,
                                $value,
                                $erased_ptr,
                                $orig_ptr,
                                $interf,
                            >
                        >::FIELD,
                )*
                _marker:NonOwningPhantom::NEW,
            };

        }



        /// Associated constant equivalents of the associated types in `InterfaceType`.
        #[allow(non_upper_case_globals)]
        pub trait InterfaceBound:InterfaceType {
            #[doc(hidden)]
            // Used to prevent users from implementing this trait.
            const __InterfaceBound_BLANKET_IMPL:PrivStruct<Self>;

            /// Describes which traits are implemented,
            /// stored in the `TypeLayout` for `DynTrait`,
            #[doc(hidden)]
            const EXTRA_CHECKS:EnabledTraits;

            $( 
                /// Whether the trait is required,
                /// and is usable by a `DynTrait` parameterized with this `InterfaceType`.
                const $selector:bool; 
            )*

        }

        const fn if_u16(cond:bool,if_true:u16)->u16{
            0_u16.wrapping_sub(cond as u16) & if_true
        }

        const fn if_u64(cond:bool,if_true:u64)->u64{
            0_u64.wrapping_sub(cond as u64) & if_true
        }

        #[allow(non_upper_case_globals)]
        impl<I> InterfaceBound for I
        where 
            I:InterfaceType,
            $( I::$auto_trait:Implementability, )*
            $( I::$marker_trait:Implementability, )*
            $( I::$selector:Implementability, )*
        {
            #[doc(hidden)]
            const EXTRA_CHECKS:EnabledTraits=EnabledTraits{
                
                // Auto traits have to be equivalent in every linked library,
                // this is why this is an array,it must match exactly.
                auto_traits:
                    $(
                        if_u16(
                            <I::$auto_trait as Implementability>::IS_IMPLD,
                            enabled_traits::auto_trait_mask::$auto_trait
                        )|
                    )*
                    0,

                // These traits can be a superset of the interface in the loaded library,
                // that is why it uses a map.
                regular_traits:
                    $(
                        if_u64(
                            <I::$marker_trait as Implementability>::IS_IMPLD,
                            enabled_traits::regular_trait_mask::$marker_trait
                        )|
                    )*
                    $(
                        if_u64(
                            <I::$selector as Implementability>::IS_IMPLD,
                            enabled_traits::regular_trait_mask::$selector
                        )|
                    )*
                    0,
            };

            $( 
                const $selector:bool=<I::$selector as Implementability>::IS_IMPLD;
            )*
            
            const __InterfaceBound_BLANKET_IMPL:PrivStruct<Self>=
                PrivStruct(PhantomData);
        }


        impl<'borr,$erased_ptr,$interf> Debug for VTable_Ref<'borr,$erased_ptr,$interf> 
        where
            $interf:InterfaceBound,
        {
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
                f.debug_struct("VTable_Ref")
                    .field("type_info",&self.type_info())
                    .finish()
            }
        }

        use self::enabled_traits::*;

        pub mod enabled_traits{
            declare_enabled_traits!{
                auto_traits[
                    $($auto_trait,)*
                ]

                regular_traits[
                    $($marker_trait,)*
                    $($selector,)*
                ]
            }
        }


    )
}

declare_meta_vtable! {
    interface=I;
    value  =T;
    erased_pointer=ErasedPtr;
    original_pointer=OrigP;

    auto_traits[
        [
            impl Send where [OrigP:Send,T:Send]
        ]
        [
            impl Sync where [OrigP:Sync,T:Sync]
        ]
    ]

    marker_traits[
        [
            impl Error where [T:std::error::Error]
        ]
    ]

    [
        #[sabi(accessible_if="<I as InterfaceBound>::Clone")]
        clone_ptr:    unsafe extern "C" fn(RRef<'_, ErasedPtr>)->ErasedPtr;
        priv _clone_ptr;
        option=Option,Some,None;
        field_index=field_index_for__clone_ptr;
        
        impl[] VtableFieldValue<Clone>
        where [OrigP:Clone]
        {
            clone_pointer_impl::<OrigP,ErasedPtr>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound>::Default")]
        default_ptr: unsafe extern "C" fn()->ErasedPtr ;
        priv _default_ptr;
        option=Option,Some,None;
        field_index=field_index_for__default_ptr;

        impl[] VtableFieldValue<Default>
        where [
            OrigP:GetPointerKind,
            OrigP:DefaultImpl<<OrigP as GetPointerKind>::Kind>,
        ]{
            default_pointer_impl::<OrigP,ErasedPtr>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound>::Display")]
        display:unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>;
        priv _display;
        option=Option,Some,None;
        field_index=field_index_for__display;

        impl[] VtableFieldValue<Display>
        where [T:Display]
        {
            display_impl::<T>
        }
    ]
    [
    #[sabi(accessible_if="<I as InterfaceBound>::Debug")]
        debug:unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>;
        priv _debug;
        option=Option,Some,None;
        field_index=field_index_for__debug;

        impl[] VtableFieldValue<Debug>
        where [T:Debug]
        {
            debug_impl::<T>
        }
    ]
    [
        #[sabi(unsafe_change_type=r#"
            for<'s>
            unsafe extern "C" fn(
                RRef<'s, ErasedObject>
            )->RResult<<I as GetSerializeProxyType<'s>>::ProxyType,RBoxError>
        "#)]
        #[sabi(accessible_if="<I as InterfaceBound>::Serialize")]
        erased_serialize:unsafe extern "C" fn(RRef<'_, ErasedObject>)->RResult<ErasedObject,RBoxError>;
        priv priv_serialize;
        option=Option,Some,None;
        field_index=field_index_for_priv_serialize;

        impl[] VtableFieldValue<Serialize>
        where [ 
            T:for<'s>SerializeImplType<'s,Interface=I>,
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
        #[sabi(accessible_if="<I as InterfaceBound>::PartialEq")]
        partial_eq: unsafe extern "C" fn(RRef<'_, ErasedObject>,RRef<'_, ErasedObject>)->bool;
        priv _partial_eq;
        option=Option,Some,None;
        field_index=field_index_for__partial_eq;

        impl[] VtableFieldValue<PartialEq>
        where [T:PartialEq,]
        {
            partial_eq_impl::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound>::Ord")]
        cmp:        unsafe extern "C" fn(RRef<'_, ErasedObject>,RRef<'_, ErasedObject>)->RCmpOrdering;
        priv _cmp;
        option=Option,Some,None;
        field_index=field_index_for__cmp;

        impl[] VtableFieldValue<Ord>
        where [T:Ord,]
        {
            cmp_ord::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound>::PartialOrd")]
        partial_cmp:unsafe extern "C" fn(RRef<'_, ErasedObject>,RRef<'_, ErasedObject>)->ROption<RCmpOrdering>;
        priv _partial_cmp;
        option=Option,Some,None;
        field_index=field_index_for__partial_cmp;

        impl[] VtableFieldValue<PartialOrd>
        where [T:PartialOrd,]
        {
            partial_cmp_ord::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound>::Hash")]
        hash:unsafe extern "C" fn(RRef<'_, ErasedObject>,trait_objects::HasherObject<'_>);
        priv _hash;
        option=Option,Some,None;
        field_index=field_index_for__hash;

        impl[] VtableFieldValue<Hash>
        where [T:Hash]
        {
            hash_Hash::<T>
        }
    ]
    [
        #[sabi(
            unsafe_change_type=
            "ROption<IteratorFns< <I as IteratorItemOrDefault<'borr>>::Item >>"
        )]
        #[sabi(accessible_if="<I as InterfaceBound>::Iterator")]
        erased_iter:IteratorFns< () >;
        priv _iter;
        option=ROption,RSome,RNone;
        field_index=field_index_for__iter;

        impl[] VtableFieldValue<Iterator>
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
            "ROption<DoubleEndedIteratorFns< <I as IteratorItemOrDefault<'borr>>::Item >>"
        )]
        #[sabi(accessible_if="<I as InterfaceBound>::DoubleEndedIterator")]
        erased_back_iter:DoubleEndedIteratorFns< () >;
        priv _back_iter;
        option=ROption,RSome,RNone;
        field_index=field_index_for__back_iter;

        impl[] VtableFieldValue<DoubleEndedIterator>
        where [
            T:DoubleEndedIterator,
            I:IteratorItemOrDefault<'borr,Item=<T as Iterator>::Item>,
        ]{
            MakeDoubleEndedIteratorFns::<T>::NEW
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound>::FmtWrite")]
        fmt_write_str:unsafe extern "C" fn(RMut<'_, ErasedObject>,RStr<'_>)->RResult<(),()>;
        priv _fmt_write_str;
        option=Option,Some,None;
        field_index=field_index_for__fmt_write_str;

        impl[] VtableFieldValue<FmtWrite>
        where [ T:FmtWrite ]
        {
            write_str_fmt_write::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound>::IoWrite")]
        io_write:IoWriteFns;
        priv _io_write;
        option=ROption,RSome,RNone;
        field_index=field_index_for__io_write;

        impl[] VtableFieldValue<IoWrite>
        where [ T:io::Write ]
        {
            MakeIoWriteFns::<T>::NEW
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound>::IoRead")]
        io_read:IoReadFns;
        priv _io_read;
        option=ROption,RSome,RNone;
        field_index=field_index_for__io_read;

        impl[] VtableFieldValue<IoRead>
        where [ T:io::Read ]
        {
            MakeIoReadFns::<T>::NEW
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound>::IoBufRead")]
        io_bufread:IoBufReadFns;
        priv _io_bufread;
        option=ROption,RSome,RNone;
        field_index=field_index_for__io_bufread;

        impl[] VtableFieldValue<IoBufRead>
        where [ 
            T:io::BufRead,
            I:InterfaceType<IoRead= Implemented<trait_marker::IoRead>>
        ]{
            MakeIoBufReadFns::<T>::NEW
        }
    ]
    [
        #[sabi(last_prefix_field)]
        #[sabi(accessible_if="<I as InterfaceBound>::IoSeek")]
        io_seek:unsafe extern "C" fn(RMut<'_, ErasedObject>,RSeekFrom)->RResult<u64,RIoError>;
        priv _io_seek;
        option=Option,Some,None;
        field_index=field_index_for__io_seek;

        impl[] VtableFieldValue<IoSeek>
        where [ T:io::Seek ]
        {
            io_Seek_seek::<T>
        }
    ]
}

//////////////


/// Used to prevent InterfaceBound being implemented outside this module,
/// since it is only constructed in the impl of InterfaceBound in this module.
#[doc(hidden)]
pub struct PrivStruct<T>(PhantomData<T>);

