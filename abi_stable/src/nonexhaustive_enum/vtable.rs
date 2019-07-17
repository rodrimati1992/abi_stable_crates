use std::{
    cmp::{PartialEq,Ord,PartialOrd},
    fmt::{Debug,Display},
    hash::Hash,
    marker::PhantomData,
};

use crate::{
    erased_types::{c_functions,trait_objects,InterfaceType,FormattingMode},
    marker_type::{ErasedObject,UnsafeIgnoredType},
    nonexhaustive_enum::{
        alt_c_functions,NonExhaustive,EnumInfo,GetEnumInfo,SerializeEnum,
    },
    prefix_type::{PrefixTypeTrait,WithMetadata,panic_on_missing_fieldname},
    type_level::{
        impl_enum::{Implemented,Unimplemented,IsImplemented},
        trait_marker,
    },
    sabi_types::{StaticRef},
    std_types::{ROption,RResult,RString,RCow,RCmpOrdering,RBoxError},
    type_layout::Tag,
    traits::InlineStorage,
};


/// Gets the vtable of `NonExhaustive<Self,S,I>`.
pub unsafe trait GetVTable<S,I>:GetEnumInfo{
    const VTABLE_VAL:NonExhaustiveVtableVal<Self,S,I>;
    
    const VTABLE_PTR: *const WithMetadata<NonExhaustiveVtableVal<Self,S,I>> = 
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VTABLE_VAL);

    const VTABLE_REF:StaticRef<NonExhaustiveVtable<Self,S,I>>=unsafe{
        let full=WithMetadata::raw_as_prefix(Self::VTABLE_PTR);
        StaticRef::from_raw(full)
    };
}



/// The vtable for NonExhaustive<>.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    not_stableabi(E,S,I),
    missing_field(default),
    kind(Prefix(prefix_struct="NonExhaustiveVtable")),
    //debug_print,
)]
pub struct NonExhaustiveVtableVal<E,S,I>{
    pub(crate) _sabi_tys:UnsafeIgnoredType<(E,S,I)>,
    
    pub enum_info:&'static EnumInfo,

    pub(crate) _sabi_drop :unsafe extern "C" fn(this:&mut ErasedObject),

    #[sabi(unsafe_opaque_field)]
    pub(crate) _sabi_clone:Option<
        extern "C" fn(
            &ErasedObject,
            StaticRef<NonExhaustiveVtable<E,S,I>>,
        )->NonExhaustive<E,S,I>
    >,

    pub(crate) _sabi_debug:Option<
        extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>
    >,
    pub(crate) _sabi_display:Option<
        extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>
    >,
    pub(crate) _sabi_serialize: Option<
        extern "C" fn(&ErasedObject)->RResult<RCow<'_,str>,RBoxError>
    >,
    pub(crate) _sabi_partial_eq: Option<
        extern "C" fn(&ErasedObject,&ErasedObject)->bool
    >,
    pub(crate) _sabi_cmp: Option<
        extern "C" fn(&ErasedObject,&ErasedObject)->RCmpOrdering,
    >,
    pub(crate) _sabi_partial_cmp: Option<
        extern "C" fn(&ErasedObject,&ErasedObject)->ROption<RCmpOrdering>,
    >,
    #[sabi(last_prefix_field)]
    pub(crate) _sabi_hash:Option<
        extern "C" fn(&ErasedObject,trait_objects::HasherObject<'_>)
    >,
}


unsafe impl<E,S,I> Sync for NonExhaustiveVtable<E,S,I>{}
unsafe impl<E,S,I> Send for NonExhaustiveVtable<E,S,I>{}


unsafe impl<E,S,I> GetVTable<S,I> for E
where 
    S:InlineStorage,
    I:InterfaceType,
    E:GetEnumInfo,
    I::Sync:RequiresSync<E,S,I>,
    I::Send:RequiresSend<E,S,I>,
    I::Clone:InitCloneField<E,S,I>,
    I::Debug:InitDebugField<E,S,I>,
    I::Display:InitDisplayField<E,S,I>,
    I::Serialize:InitSerializeField<E,S,I>,
    I::PartialEq:InitPartialEqField<E,S,I>,
    I::PartialOrd:InitPartialOrdField<E,S,I>,
    I::Ord:InitOrdField<E,S,I>,
    I::Hash:InitHashField<E,S,I>,
{
    const VTABLE_VAL:NonExhaustiveVtableVal<E,S,I>=
        NonExhaustiveVtableVal{
            _sabi_tys:UnsafeIgnoredType::DEFAULT,
            enum_info:E::ENUM_INFO,
            _sabi_drop:alt_c_functions::drop_impl::<E>,
            _sabi_clone:<I::Clone as InitCloneField<E,S,I>>::VALUE,
            _sabi_debug:<I::Debug as InitDebugField<E,S,I>>::VALUE,
            _sabi_display:<I::Display as InitDisplayField<E,S,I>>::VALUE,
            _sabi_serialize:<I::Serialize as InitSerializeField<E,S,I>>::VALUE,
            _sabi_partial_eq:<I::PartialEq as InitPartialEqField<E,S,I>>::VALUE,
            _sabi_partial_cmp:<I::PartialOrd as InitPartialOrdField<E,S,I>>::VALUE,
            _sabi_cmp:<I::Ord as InitOrdField<E,S,I>>::VALUE,
            _sabi_hash:<I::Hash as InitHashField<E,S,I>>::VALUE,
        };
}





use self::trait_bounds::*;
pub mod trait_bounds{
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

            impl<E,S,$interf> NonExhaustiveVtable<E,S,$interf>{
                pub fn $field(&self)->$field_ty
                where
                    $interf:InterfaceType<$selector=Implemented<trait_marker::$selector>>,
                {
                    match self.$priv_field().into() {
                        Some(v)=>v,
                        None=>panic_on_missing_fieldname::<
                            NonExhaustiveVtableVal<E,S,$interf>,
                        >(
                            Self::$field_index,
                            self._prefix_type_layout(),
                        )
                    }
                }
            }
        )
    }


    declare_conditional_marker!{
        type Send;
        trait RequiresSend[E,S,I]
        where [ E:Send ]
    }

    declare_conditional_marker!{
        type Sync;
        trait RequiresSync[E,S,I]
        where [ E:Sync ]
    }

    declare_field_initalizer!{
        type Clone;
        trait InitCloneField[E,S,I]
        where_for_both[ E:GetEnumInfo, ]
        where [ E:Clone ]
        _sabi_clone,clone_: 
            extern "C" fn(
                &ErasedObject,
                StaticRef<NonExhaustiveVtable<E,S,I>>
            )->NonExhaustive<E,S,I>;
        field_index=field_index_for__sabi_clone;
        value=alt_c_functions::clone_impl::<E,S,I>,
    }
    declare_field_initalizer!{
        type Debug;
        trait InitDebugField[E,S,I]
        where [ E:Debug ]
        _sabi_debug,debug: 
            extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>;
        field_index=field_index_for__sabi_debug;
        value=c_functions::debug_impl::<E>,
    }
    declare_field_initalizer!{
        type Display;
        trait InitDisplayField[E,S,I]
        where [ E:Display ]
        _sabi_display,display: 
            extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>;
        field_index=field_index_for__sabi_display;
        value=c_functions::display_impl::<E>,
    }
    declare_field_initalizer!{
        type Serialize;
        trait InitSerializeField[E,S,I]
        where [ I:SerializeEnum<E> ]
        _sabi_serialize,serialize: 
            extern "C" fn(&ErasedObject)->RResult<RCow<'_,str>,RBoxError>;
        field_index=field_index_for__sabi_serialize;
        value=alt_c_functions::serialize_impl::<E,I>,
    }
    declare_field_initalizer!{
        type PartialEq;
        trait InitPartialEqField[E,S,I]
        where_for_both[ E:GetEnumInfo, ]
        where [ E:PartialEq ]
        _sabi_partial_eq,partial_eq: extern "C" fn(&ErasedObject,&ErasedObject)->bool;
        field_index=field_index_for__sabi_partial_eq;
        value=alt_c_functions::partial_eq_impl::<E,S,I>,
    }
    declare_field_initalizer!{
        type PartialOrd;
        trait InitPartialOrdField[E,S,I]
        where_for_both[ E:GetEnumInfo, ]
        where [ E:PartialOrd ]
        _sabi_partial_cmp,partial_cmp:
            extern "C" fn(&ErasedObject,&ErasedObject)->ROption<RCmpOrdering>;
        field_index=field_index_for__sabi_partial_cmp;
        value=alt_c_functions::partial_cmp_ord::<E,S,I>,
    }
    declare_field_initalizer!{
        type Ord;
        trait InitOrdField[E,S,I]
        where_for_both[ E:GetEnumInfo, ]
        where [ E:Ord ]
        _sabi_cmp,cmp: extern "C" fn(&ErasedObject,&ErasedObject)->RCmpOrdering;
        field_index=field_index_for__sabi_cmp;
        value=alt_c_functions::cmp_ord::<E,S,I>,
    }
    declare_field_initalizer!{
        type Hash;
        trait InitHashField[E,S,I]
        where [ E:Hash ]
        _sabi_hash,hash: extern "C" fn(&ErasedObject,trait_objects::HasherObject<'_>);
        field_index=field_index_for__sabi_hash;
        value=c_functions::hash_Hash::<E>,
    }
}

macro_rules! declare_InterfaceBound {
    (
        auto_traits=[ $( $auto_trait:ident ),* $(,)* ]
        required_traits=[ $( $required_traits:ident ),* $(,)* ]
    ) => (

        #[allow(non_upper_case_globals)]
        pub trait InterfaceBound:InterfaceType{
            const TAG:Tag;
            $(const $auto_trait:bool;)*
            $(const $required_traits:bool;)*
        }

        #[allow(non_upper_case_globals)]
        impl<I> InterfaceBound for I
        where 
            I:InterfaceType,
            $(I::$auto_trait:IsImplemented,)*
            $(I::$required_traits:IsImplemented,)*
        {
            const TAG:Tag={
                const fn str_if(cond:bool,s:&'static str)->Tag{
                    [ Tag::null(), Tag::str(s) ][cond as usize]
                }

                tag!{{
                    "auto traits"=>tag![[
                        $(  
                            str_if(
                                <I::$auto_trait as IsImplemented>::VALUE,
                                stringify!($auto_trait)
                            ),
                        )*
                    ]],
                    "required traits"=>tag!{{
                        $(  
                            str_if(
                                <I::$required_traits as IsImplemented>::VALUE,
                                stringify!($required_traits)
                            ),
                        )*
                    }}
                }}
            };

            $(const $auto_trait:bool=<I::$auto_trait as IsImplemented>::VALUE;)*
            $(const $required_traits:bool=<I::$required_traits as IsImplemented>::VALUE;)*
        }
    )
}

declare_InterfaceBound!{
    auto_traits=[ Sync,Send ]
    required_traits=[ 
        Clone,
        Debug,Display,
        Serialize,Deserialize,
        Eq,PartialEq,Ord,PartialOrd,
        Hash,Error,
    ]
}