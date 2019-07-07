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
    traits::{IteratorItemOrDefault},
};

use crate::{
    StableAbi,
    marker_type::ErasedObject,
    prefix_type::{PrefixTypeTrait,WithMetadata,panic_on_missing_fieldname},
    pointer_trait::GetPointerKind,
    std_types::{Tuple3,RSome,RNone,RIoError,RSeekFrom},
    type_layout::Tag,
    type_level::{
        impl_enum::{Implemented,Unimplemented,IsImplemented},
        trait_marker,
    },
};


use core_extensions::TypeIdentity;





#[doc(hidden)]
/// Returns the vtable used by DynTrait to do dynamic dispatch.
pub trait GetVtable<'borr,This,ErasedPtr,OrigPtr,I:InterfaceBound<'borr>> {
    
    const TMP_VTABLE:VTableVal<'borr,ErasedPtr,I>;

    const GET_VTABLE:*const WithMetadata<VTableVal<'borr,ErasedPtr,I>>=
        &WithMetadata::new(
            PrefixTypeTrait::METADATA,
            Self::TMP_VTABLE
        );


    /// Retrieves the VTable of the type.
    fn get_vtable<'a>() -> &'a VTable<'borr,ErasedPtr,I>
    where
        This: 'a,
    {
        // I am just getting a vtable
        unsafe { (*Self::GET_VTABLE).as_prefix() }
    }
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
            kind(Prefix(prefix_struct="VTable")),
            missing_field(panic),
            prefix_bound="I:InterfaceBound<'borr>",
            bound="<I as InterfaceBound<'borr>>::IteratorItem:StableAbi",
            $($(bound=$struct_bound,)*)*
        )]
        pub struct VTableVal<'borr,$erased_ptr,$interf>
        where $interf:InterfaceBound<'borr>
        {
            pub type_info:&'static TypeInfo,
            _marker:PhantomData<extern fn()->Tuple3<$erased_ptr,$interf,&'borr()>>,
            pub drop_ptr:unsafe extern "C" fn(&mut $erased_ptr),
            $(
                $( #[$field_attr] )*
                $priv_field:$option_ty<($field_ty)>,
            )*
        }


        impl<'borr,$erased_ptr,$interf> VTable<'borr,$erased_ptr,$interf>
        where   
            $interf:InterfaceBound<'borr>,
        {
            $(
                pub fn $field(&self)->($field_ty)
                where
                    $interf:InterfaceType<$selector=Implemented<trait_marker::$selector>>,
                {
                    match self.$priv_field().into() {
                        Some(v)=>v,
                        None=>panic_on_missing_fieldname::<
                            VTableVal<'borr,$erased_ptr,$interf>,
                        >(
                            Self::$field_index,
                            self._prefix_type_layout(),
                        )
                    }
                }
            )*
        }

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
                $interf:InterfaceBound<'borr>,
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
                $interf:InterfaceBound<'borr>,
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
            $interf:InterfaceBound<'borr>,
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
            const TMP_VTABLE:VTableVal<'borr,$erased_ptr,$interf>=VTableVal{
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
                _marker:PhantomData,
            };

        }



        /// Trait used to capture all the bounds of DynTrait<_>.
        #[allow(non_upper_case_globals)]
        pub trait InterfaceBound<'borr>:InterfaceType {
            #[doc(hidden)]
            // Used to prevent users from implementing this trait.
            const __InterfaceBound_BLANKET_IMPL:PrivStruct<Self>;

            /// The Item type being iterated over,
            /// if `Iterator=False` this is `()`.
            type IteratorItem:'borr;

            /// Describes which traits are implemented,
            /// stored in the layout of the type in StableAbi,
            /// using the `#[sabi(tag="<I as InterfaceBound<'borr>>::TAG")]` attribute
            const TAG:Tag;

            $( 
                /// Used by the `StableAbi` derive macro to determine whether the field 
                /// this is associated with is disabled.
                const $selector:bool; 
            )*

        }   

        #[allow(non_upper_case_globals)]
        impl<'borr,I> InterfaceBound<'borr> for I
        where 
            I:InterfaceType,
            I:IteratorItemOrDefault<'borr,<I as InterfaceType>::Iterator>,
            $( I::$auto_trait:IsImplemented, )*
            $( I::$marker_trait:IsImplemented, )*
            $( I::$selector:IsImplemented, )*
        {
            type IteratorItem=
                <I as IteratorItemOrDefault<'borr,<I as InterfaceType>::Iterator>>::Item ;


            const TAG:Tag={
                const fn str_if(cond:bool,s:&'static str)->Tag{
                    // nulls are stripped in Tag collection variants.
                    //
                    // I'm using null here because using Vec<_> in constants isn't possible.
                    [ Tag::null(), Tag::str(s) ][cond as usize]
                }

                tag!{{
                    // Auto traits have to be equivalent in every linked library,
                    // this is why this is an array,it must match exactly.
                    "auto traits"=>tag![[
                        $(
                            str_if(
                                <I::$auto_trait as IsImplemented>::VALUE,
                                stringify!($auto_trait)
                            ),
                        )*
                    ]],
                    // These traits can be a superset of the interface in the loaded library,
                    // that is why it uses a map.
                    "required traits"=>tag!{{
                        $(
                            str_if(
                                <I::$selector as IsImplemented>::VALUE,
                                stringify!($selector)
                            ),
                        )*
                        $(
                            str_if(
                                <I::$marker_trait as IsImplemented>::VALUE,
                                stringify!($marker_trait)
                            ),
                        )*
                    }}
                }}
            };

            $( 
                const $selector:bool=<I::$selector as IsImplemented>::VALUE;
            )*
            
            const __InterfaceBound_BLANKET_IMPL:PrivStruct<Self>=
                PrivStruct(PhantomData);
        }


        impl<'borr,$erased_ptr,$interf> Debug for VTable<'borr,$erased_ptr,$interf> 
        where
            $interf:InterfaceBound<'borr>,
        {
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
                f.debug_struct("VTable")
                    .field("type_info",&self.type_info())
                    // $(
                    //     .field(
                    //         stringify!($field),
                    //         &format_args!("{:x}",self.$priv_field().map_or(0,|x|x as usize))
                    //     )
                    // )*
                    .finish()
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::Clone")]
        clone_ptr:    extern "C" fn(&ErasedPtr)->ErasedPtr;
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::Default")]
        default_ptr: extern "C" fn()->ErasedPtr ;
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::Display")]
        display:    extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>;
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
    #[sabi(accessible_if="<I as InterfaceBound<'borr>>::Debug")]
        debug:      extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>;
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::Serialize")]
        serialize:  extern "C" fn(&ErasedObject)->RResult<RCow<'_,str>,RBoxError>;
        priv _serialize;
        option=Option,Some,None;
        field_index=field_index_for__serialize;

        impl[] VtableFieldValue<Serialize>
        where [ T:SerializeImplType, ]{
            serialize_impl::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::PartialEq")]
        partial_eq: extern "C" fn(&ErasedObject,&ErasedObject)->bool;
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::Ord")]
        cmp:        extern "C" fn(&ErasedObject,&ErasedObject)->RCmpOrdering;
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::PartialOrd")]
        partial_cmp:extern "C" fn(&ErasedObject,&ErasedObject)->ROption<RCmpOrdering>;
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::Hash")]
        hash:extern "C" fn(&ErasedObject,trait_objects::HasherObject<'_>);
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::Iterator")]
        iter:IteratorFns< <I as InterfaceBound<'borr>>::IteratorItem >;
        priv _iter;
        option=ROption,RSome,RNone;
        field_index=field_index_for__iter;

        impl[] VtableFieldValue<Iterator>
        where [
            T:Iterator,
            T::Item:'borr,
            I:InterfaceBound<'borr,IteratorItem=<T as Iterator>::Item>,
        ]{
            MakeIteratorFns::<T>::NEW
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::DoubleEndedIterator")]
        back_iter:DoubleEndedIteratorFns< <I as InterfaceBound<'borr>>::IteratorItem >;
        priv _back_iter;
        option=ROption,RSome,RNone;
        field_index=field_index_for__back_iter;

        impl[] VtableFieldValue<DoubleEndedIterator>
        where [
            T:DoubleEndedIterator,
            T::Item:'borr,
            I:InterfaceBound<
                'borr,
                Iterator=Implemented<trait_marker::Iterator>,
                IteratorItem=<T as Iterator>::Item
            >,
        ]{
            MakeDoubleEndedIteratorFns::<T>::NEW
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::FmtWrite")]
        fmt_write_str:extern "C" fn(&mut ErasedObject,RStr<'_>)->RResult<(),()>;
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::IoWrite")]
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::IoRead")]
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::IoBufRead")]
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
        #[sabi(accessible_if="<I as InterfaceBound<'borr>>::IoSeek")]
        io_seek:extern "C" fn(&mut ErasedObject,RSeekFrom)->RResult<u64,RIoError>;
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

