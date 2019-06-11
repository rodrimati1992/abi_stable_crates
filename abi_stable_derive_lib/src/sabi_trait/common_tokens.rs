use proc_macro2::{Span,TokenStream};



use crate::{
    *,
    fn_pointer_extractor::FnParamRet,
};

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

macro_rules! declare_common_tokens {
    (
        abi[ $( $field_abi:ident = $abi_str:expr , )* ]
        type_param_bound[ $( $field_ty_param_bound:ident = $ty_param_bound_str:expr , )* ]
        token_streams[ $( $field_ts:ident = $ts_str:expr , )* ]
        types[ $( $field_ty:ident = $ty_str:expr , )* ]
        idents[ $( $field_ident:ident = $ident_str:expr , )* ]
        lifetime[ $( $lifetime_ident:ident = $lifetime_str:expr , )* ]
        lifetime_def[ $( $lifetime_def_ident:ident = $lifetime_def_str:expr , )* ]
        str_lits[ $( $strlit_ident:ident = $strlit_str:expr , )* ]
        patterns[ $( $pat_ident:ident = $pat_str:expr , )* ]
    ) => {
        #[derive(Debug)]
        pub(crate) struct CommonTokens{
            $( pub(crate) $field_abi : ::syn::Abi , )*
            $( pub(crate) $field_ty_param_bound : ::syn::TypeParamBound , )*
            $( pub(crate) $field_ts : TokenStream , )*
            $( pub(crate) $field_ty : ::syn::Type , )*
            $( pub(crate) $field_ident : ::syn::Ident , )*
            $( pub(crate) $lifetime_ident : ::syn::Lifetime , )*
            $( pub(crate) $lifetime_def_ident : ::syn::LifetimeDef , )*
            $( pub(crate) $strlit_ident : ::syn::LitStr , )*
            $( pub(crate) $pat_ident : ::syn::Pat , )*
        }

        impl CommonTokens{
            #[allow(unused_variables)]
            pub(crate) fn new()->Self{
                let span=Span::call_site();
                Self{
                    $( $field_abi : ::syn::parse_str($abi_str).unwrap(), )*
                    $( $field_ty_param_bound : ::syn::parse_str($ty_param_bound_str).unwrap(), )*
                    $( $field_ts : ::syn::parse_str($ts_str).unwrap() , )*
                    $( $field_ty : ::syn::parse_str($ty_str).unwrap() , )*
                    $( $field_ident : ::syn::Ident::new($ident_str,span) , )*
                    $( $lifetime_ident : ::syn::parse_str($lifetime_str).unwrap() , )*
                    $( $lifetime_def_ident : ::syn::parse_str($lifetime_def_str).unwrap() , )*
                    $( $strlit_ident : ::syn::LitStr::new($strlit_str,span) , )*
                    $( $pat_ident : ::syn::parse_str($pat_str).unwrap() , )*
                }
            }
        }
    }
}

impl Eq for CommonTokens {}
impl PartialEq for CommonTokens {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl PartialOrd for CommonTokens {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl Ord for CommonTokens {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

declare_common_tokens! {
    abi[
        extern_c=r#"extern "C" "#,
    ]

    type_param_bound[
        sized_bound="Sized",
        // ref_self_ty="&self",
        // mut_self_ty="&mut Self",
        // val_self_ty="Self",
        // ref_uself_ty="&__ErasedObject<_Self>",
        // mut_uself_ty="&mut __ErasedObject<_Self>",
        // val_uself_ty="__sabi_re::MovePtr<'_,_Self>",
    ]

    token_streams[
        self_sized="Self:Sized,",
        makevtable_typarams="IA,_Self,_ErasedPtr,_OrigPtr,",
        vtable_typarams="_Self,_ErasedPtr,",
        trait_obj_typarams="'lt,_ErasedPtr",

        ptr_ref_bound=
            "_ErasedPtr: __DerefTrait<Target=()>,",
        ptr_mut_bound=
            "_ErasedPtr: __DerefMutTrait<Target=()>,",
        ptr_ref_val_bound=
            "_ErasedPtr: __DerefTrait<Target=()>+__sabi_re::OwnedPointer<Target=()>,",
        ptr_mut_val_bound=
            "_ErasedPtr: __DerefMutTrait<Target=()>+__sabi_re::OwnedPointer<Target=()>,",

        empty_ts="",

        ts_self ="Self",
        ts_uself="_Self",

        ts_self_colon2 ="Self::",
        ts_uself_colon2="_Self::",

        ts_lt="'lt,",
        ts_lt_self_erasability="'lt,_Self,Erasability,",
        ts_lt_rbox="'lt,__sabi_re::RBox<()>,",
        ts_lt_origptr_erasability="'lt,_OrigPtr,Erasability,",
        ts_lt_uself_erasability="'lt,_Self,Erasability,",
        ts_lt_rbox_uself_erasability="'lt,__sabi_re::RBox<_Self>,Erasability,",
        ts_make_vtable_args="Erasability,_OrigPtr::Target,_OrigPtr::TransmutedPtr,_OrigPtr,",
        ts_lt_transptr="'lt,_OrigPtr::TransmutedPtr,",
        ts_lt_erasedptr="'lt,_ErasedPtr,",
        ts_erasedptr="_ErasedPtr,",
        ts_self_erasedptr="_Self,_ErasedPtr,",
        ts_unit_erasedptr="(),_ErasedPtr,",

        ts_getvtable_params="IA,_Self,_ErasedPtr,_OrigPtr,",
    ]

    types[
        empty_tuple="()",
        self_ty="Self",
    ]

    idents[
        u_erased_ptr="_ErasedPtr",
        nope_ident="__NOPE__",
        self_ident="self",
        uself_ident="_self",
        u_capself="_Self",
        capself="Self",
    ]

    lifetime[
        static_lifetime="'static",
        under_lifetime="'_",
        uself_lifetime="'_self",
    ]

    lifetime_def[
        uself_lt_def="'_self",
    ]

    str_lits[
        c_abi_lit="C",
    ]

    patterns[
        ignored_pat="_",
    ]
}
