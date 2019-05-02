/*!

Contains `DynTrait<_>`'s vtable,and related types/traits.

*/
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use super::c_functions::*;
use super::*;

use crate::{
    abi_stability::{Tag,SharedStableAbi},
    marker_type::ErasedObject,
    prefix_type::{PrefixTypeTrait,WithMetadata,panic_on_missing_fieldname},
    std_types::Tuple2,
};

use core_extensions::{ResultLike, StringExt};





#[doc(hidden)]
/// Returns the vtable used by DynTrait to do dynamic dispatch.
pub trait GetVtable<This,ErasedPtr,OrigPtr,I:InterfaceConstsBound>: ImplType {
    
    const TMP_VTABLE:VTableVal<ErasedPtr,I>;

    const GET_VTABLE:*const WithMetadata<VTableVal<ErasedPtr,I>>=
        &WithMetadata::new(
            PrefixTypeTrait::METADATA,
            Self::TMP_VTABLE
        );


    /// Retrieves the VTable of the type.
    fn get_vtable<'a>() -> &'a VTable<ErasedPtr,I>
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

        marker_traits[
            $([
                impl $marker_trait:ident where [ $($phantom_where_clause:tt)* ]
            ])*
        ]

        $([
            $( #[$field_attr:meta] )*
            $field:ident : $field_ty:ty ;
            priv $priv_field:ident;
            $(struct_bound=$struct_bound:expr;)*
            
            impl[$($impl_params:tt)*] VtableFieldValue<$selector:ident>
            where [ $($where_clause:tt)* ]
            { $field_value:expr }
        ])*
    ) => (

        /// This is the vtable for DynTrait<_>,
        ///
        #[repr(C)]
        #[derive(StableAbi)]
        #[sabi(
            inside_abi_stable_crate,
            kind(Prefix(prefix_struct="VTable")),
            missing_field(panic),
            prefix_bound="I:InterfaceConstsBound",
            bound="<I as SharedStableAbi>::StaticEquivalent:InterfaceBound",
            $($(bound=$struct_bound,)*)*
        )]
        pub struct VTableVal<$erased_ptr,$interf>
        where $interf:InterfaceBound
        {
            pub type_info:&'static TypeInfo,
            _marker:PhantomData<extern fn()->Tuple2<$erased_ptr,$interf>>,
            pub drop_ptr:unsafe extern "C" fn(&mut $erased_ptr),
            $(
                $( #[$field_attr] )*
                $priv_field:Option<($field_ty)>,
            )*
        }


        impl<$erased_ptr,$interf> VTable<$erased_ptr,$interf>
        where   
            $interf:InterfaceConstsBound,
        {
            $(
                pub fn $field(&self)->($field_ty)
                where
                    $interf:InterfaceType<$selector=True>,
                {
                    const NAME:&'static &'static str=&stringify!($field);

                    match self.$priv_field() {
                        Some(v)=>v,
                        None=>panic_on_missing_fieldname::<
                            VTableVal<$erased_ptr,$interf>,
                            $field_ty
                        >(
                            NAME,
                            self._prefix_type_layout(),
                        )
                    }
                }
            )*
        }

        /// Returns the type of a vtable field.
        pub type VTableFieldType<Selector,$value,$erased_ptr,$orig_ptr>=
            <Selector as VTableFieldType_<$value,$erased_ptr,$orig_ptr>>::Field;

        /// Returns the type of a vtable field.
        pub trait VTableFieldType_<$value,$erased_ptr,$orig_ptr>{
            type Field;
        }

        /// Returns the value of a vtable field in the current binary
        /// (this can be a different value in a dynamically_linked_library/executable).
        pub trait VTableFieldValue<Ty,IsImpld,$value,$erased_ptr,$orig_ptr>{
            const FIELD:Option<Ty>;
        }

        pub trait MarkerTrait<IsImpld,$value,$erased_ptr,$orig_ptr>{}


        $(
            impl<$value,$erased_ptr,$orig_ptr> 
                VTableFieldType_<$value,$erased_ptr,$orig_ptr> 
            for trait_selector::$selector 
            {
                type Field=($field_ty);
            }

            impl<$value,$erased_ptr,$orig_ptr,$($impl_params)*>
                VTableFieldValue<($field_ty),True,$value,$erased_ptr,$orig_ptr>
            for trait_selector::$selector
            where $($where_clause)*
            {
                const FIELD:Option<($field_ty)>=Some($field_value);
            }
        )*
        impl<AnyFieldTy,AnySelector,$value,$erased_ptr,$orig_ptr>
            VTableFieldValue<AnyFieldTy,False,$value,$erased_ptr,$orig_ptr>
        for AnySelector
        {
            const FIELD:Option<AnyFieldTy>=None;
        }



        impl<Anything,$value,$erased_ptr,$orig_ptr> 
            MarkerTrait<False,$value,$erased_ptr,$orig_ptr> 
        for Anything
        {}

        $(
            impl<$value,$erased_ptr,$orig_ptr> 
                MarkerTrait<True,$value,$erased_ptr,$orig_ptr> 
            for trait_selector::$marker_trait
            where $($phantom_where_clause)*
            {}
        )*

        ///////////////////////////////////////////////////////////

        /// Contains marker types representing traits of the same name.
        pub mod trait_selector{
            $(
                /// Marker type representing the trait of the same name.
                pub struct $marker_trait;
            )*
            $(
                /// Marker type representing the trait of the same name.
                pub struct $selector;
            )*
        }


        impl<This,$value,$erased_ptr,$orig_ptr,$interf> 
            GetVtable<$value,$erased_ptr,$orig_ptr,$interf>
        for This
        where
            This:ImplType<Interface=$interf>,
            $interf:InterfaceConstsBound,
            $(
                trait_selector::$marker_trait:
                    MarkerTrait<$interf::$marker_trait,$value,$erased_ptr,$orig_ptr>,
            )*
            $(
                trait_selector::$selector:VTableFieldValue<
                    ($field_ty),
                    $interf::$selector,
                    $value,
                    $erased_ptr,
                    $orig_ptr,
                >,
            )*
        {
            const TMP_VTABLE:VTableVal<$erased_ptr,$interf>=VTableVal{
                type_info:This::INFO,
                drop_ptr:drop_pointer_impl::<$orig_ptr,$erased_ptr>,
                $(
                    $priv_field:
                        <trait_selector::$selector as
                            VTableFieldValue<
                                VTableFieldType<
                                    trait_selector::$selector,
                                    $value,
                                    $erased_ptr,
                                    $orig_ptr,
                                >,
                                $interf::$selector,
                                $value,
                                $erased_ptr,
                                $orig_ptr,
                            >
                        >::FIELD,
                )*
                _marker:PhantomData,
            };

        }


        /// Trait used to capture all the bounds that an InterfaceType 
        /// when used as a type parameter of a type.
        #[allow(non_upper_case_globals)]
        pub trait InterfaceBound:InterfaceType {
            #[doc(hidden)]
            const __InterfaceBound_BLANKET_IMPL:PrivStruct<Self>;
        }   

        /// Associated constants derived from an InterfaceType.
        #[allow(non_upper_case_globals)]
        pub trait InterfaceConstsBound:InterfaceBound {
            const TAG:Tag;

            $( const $selector:bool; )*

            #[doc(hidden)]
            const __InterfaceConstsBound_BLANKET_IMPL:PrivStruct<Self>;
        }   


        #[allow(non_upper_case_globals)]
        impl<I> InterfaceBound for I
        where 
            I:InterfaceType,
        {
            const __InterfaceBound_BLANKET_IMPL:PrivStruct<Self>=
                PrivStruct(PhantomData);
        }


        #[allow(non_upper_case_globals)]
        impl<I> InterfaceConstsBound for I
        where 
            I:InterfaceBound,
            $( I::$marker_trait:Boolean, )*
            $( I::$selector:Boolean, )*
        {
            const TAG:Tag={
                const fn str_if(cond:bool,s:&'static str)->Tag{
                    [ Tag::null(), Tag::str(s) ][cond as usize]
                }

                tag!{{
                    "auto traits"=>tag![[
                        $(
                            str_if(
                                <I::$marker_trait as Boolean>::VALUE,
                                stringify!($marker_trait)
                            ),
                        )*
                    ]],
                    "required traits"=>tag!{{
                        $(
                            str_if(
                                <I::$selector as Boolean>::VALUE,
                                stringify!($selector)
                            ),
                        )*
                    }}
                }}
            };

            $( 
                const $selector:bool=<I::$selector as Boolean>::VALUE;
            )*
            
            const __InterfaceConstsBound_BLANKET_IMPL:PrivStruct<Self>=
                PrivStruct(PhantomData);
        }


        impl<$erased_ptr,$interf> Debug for VTable<$erased_ptr,$interf> 
        where
            $interf:InterfaceConstsBound,
        {
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
                f.debug_struct("VTable")
                    .field("type_info",&self.type_info())
                    $(
                        .field(
                            stringify!($field),
                            &format_args!("{:x}",self.$priv_field().map_or(0,|x|x as usize))
                        )
                    )*
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

    marker_traits[
        [
            impl Send where [OrigP:Send]
        ]
        [
            impl Sync where [OrigP:Sync]
        ]
    ]

    [
        #[sabi(accessible_if="<I as InterfaceConstsBound>::Clone")]
        clone_ptr:    extern "C" fn(&ErasedPtr)->ErasedPtr;
        priv _clone_ptr;
        
        impl[] VtableFieldValue<Clone>
        where [OrigP:Clone]
        {
            clone_pointer_impl::<OrigP,ErasedPtr>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceConstsBound>::Default")]
        default_ptr: extern "C" fn()->ErasedPtr ;
        priv _default_ptr;
        impl[] VtableFieldValue<Default>
        where [OrigP:Default]
        {
            default_pointer_impl::<OrigP,ErasedPtr>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceConstsBound>::Display")]
        display:    extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>;
        priv _display;
        impl[] VtableFieldValue<Display>
        where [T:Display]
        {
            display_impl::<T>
        }
    ]
    [
    #[sabi(accessible_if="<I as InterfaceConstsBound>::Debug")]
        debug:      extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>;
        priv _debug;
        impl[] VtableFieldValue<Debug>
        where [T:Debug]
        {
            debug_impl::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceConstsBound>::Serialize")]
        serialize:  extern "C" fn(&ErasedObject)->RResult<RCow<'_,RStr<'_>>,RBoxError>;
        priv _serialize;
        impl[] VtableFieldValue<Serialize>
        where [
            T:ImplType+SerializeImplType,
            T::Interface:InterfaceType<Serialize=True>,
        ]{
            serialize_impl::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceConstsBound>::PartialEq")]
        partial_eq: extern "C" fn(&ErasedObject,&ErasedObject)->bool;
        priv _partial_eq;
        impl[] VtableFieldValue<PartialEq>
        where [T:PartialEq,]
        {
            partial_eq_impl::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceConstsBound>::Ord")]
        cmp:        extern "C" fn(&ErasedObject,&ErasedObject)->RCmpOrdering;
        priv _cmp;
        impl[] VtableFieldValue<Ord>
        where [T:Ord,]
        {
            cmp_ord::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceConstsBound>::PartialOrd")]
        partial_cmp:extern "C" fn(&ErasedObject,&ErasedObject)->ROption<RCmpOrdering>;
        priv _partial_cmp;
        impl[] VtableFieldValue<PartialOrd>
        where [T:PartialOrd,]
        {
            partial_cmp_ord::<T>
        }
    ]
    [
        #[sabi(accessible_if="<I as InterfaceConstsBound>::Hash")]
        #[sabi(last_prefix_field)]
        hash:extern "C" fn(&ErasedObject,trait_objects::HasherObject<'_>);
        priv _hash;
        impl[] VtableFieldValue<Hash>
        where [T:Hash]
        {
            hash_Hash::<T>
        }
    ]
}

//////////////


/// Used to prevent InterfaceBound being implemented outside this module,
/// since it is only constructed in the impl of InterfaceBound in this module.
#[doc(hidden)]
pub struct PrivStruct<T>(PhantomData<T>);

