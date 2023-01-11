//! This module defines the CommonTokens type,
//! used to pass constants of type from `syn` to
//! many functions in the `abi_stable_derive_lib::sabi_trait` module.

use proc_macro2::{Span, TokenStream};

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

macro_rules! declare_common_tokens {
    (
        abi[ $( $field_abi:ident = $abi_str:expr , )* ]
        type_param_bound[ $( $field_ty_param_bound:ident = $ty_param_bound_str:expr , )* ]
        token_streams[ $( $field_ts:ident = $ts_str:expr , )* ]
        types[ $( $field_ty:ident = $ty_str:expr , )* ]
        idents[ $( $field_ident:ident = $ident_str:expr , )* ]
        lifetime[ $( $lifetime_ident:ident = $lifetime_str:expr , )* ]
        str_lits[ $( $strlit_ident:ident = $strlit_str:expr , )* ]
        patterns[ $( $pat_ident:ident = $pat_str:expr , )* ]
        token[ $( $token_ident:ident = $token_path:ident , )* ]
    ) => {
        #[derive(Debug)]
        pub(crate) struct CommonTokens{
            $( pub(crate) $field_abi : ::syn::Abi , )*
            $( pub(crate) $field_ty_param_bound : ::syn::TypeParamBound , )*
            $( pub(crate) $field_ts : TokenStream , )*
            $( pub(crate) $field_ty : ::syn::Type , )*
            $( pub(crate) $field_ident : ::syn::Ident , )*
            $( pub(crate) $lifetime_ident : ::syn::Lifetime , )*
            $( pub(crate) $strlit_ident : ::syn::LitStr , )*
            $( pub(crate) $pat_ident : ::syn::Pat , )*
            $( pub(crate) $token_ident : ::syn::token::$token_path , )*
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
                    $( $strlit_ident : ::syn::LitStr::new($strlit_str,span) , )*
                    $( $pat_ident : ::syn::parse_str($pat_str).unwrap() , )*
                    $( $token_ident : Default::default() , )*
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

        ptr_ref_bound=
            "_ErasedPtr: __sabi_re::AsPtr<PtrTarget=()>,",
        ptr_mut_bound=
            "_ErasedPtr: __sabi_re::AsMutPtr<PtrTarget=()>,",
        ptr_ref_val_bound=
            "_ErasedPtr: __sabi_re::OwnedPointer<PtrTarget=()>,",
        ptr_mut_val_bound=
            "_ErasedPtr: __sabi_re::OwnedPointer<PtrTarget=()>,",
        ptr_val_bound=
            "_ErasedPtr: __sabi_re::OwnedPointer<PtrTarget=()>,",

        empty_ts="",
        ts_empty="",

        ts_self_colon2 ="Self::",
        ts_uself_colon2="_Self::",

        ts_make_vtable_args="Downcasting,_OrigPtr::PtrTarget,_OrigPtr::TransmutedPtr,_OrigPtr,",
        ts_make_vtable_args_const="Downcasting,_Self,__sabi_re::RRef<'_sub, ()>, &'_sub _Self,",
        ts_erasedptr_and2="_ErasedPtr,_ErasedPtr2,",
        ts_self_erasedptr="_Self,_ErasedPtr,",
        ts_unit_erasedptr="(),_ErasedPtr,",

        ts_getvtable_params="'lt,'_sub,IA,_Self,_ErasedPtr,_OrigPtr,",
        ts_getvtable_dummy_struct_fields="&'lt(),&'_sub(),IA,_Self,_ErasedPtr,_OrigPtr,",
        missing_field_option="#[sabi(missing_field(option))]",
    ]

    types[
        self_ty="Self",
    ]

    idents[
        u_capself="_Self",
    ]

    lifetime[
        static_lifetime="'static",
    ]

    str_lits[]

    patterns[]

    token[
        unsafe_=Unsafe,
    ]
}

////////////////////////////////////////////////////////

macro_rules! declare_lifetime_tokens {
    (
        lifetime_tokens=[ $( $ident:ident = $expr:expr ,)* ]
        one_lifetime_tokens=[ $( $one_ident:ident = $one_expr:expr ,)* ]
        static_lifetime_tokens=[ $( $static_ident:ident = $static_expr:expr ,)* ]
    ) => (

        #[derive(Debug,Clone,Copy)]
        pub(crate) enum IsStaticTrait{
            Yes,
            No,
        }

        #[derive(Debug,Clone)]
        pub(crate) struct LifetimeTokens{
            $(
                pub(crate) $ident:TokenStream,
            )*
            $(
                pub(crate) $static_ident:TokenStream,
            )*
            $(
                pub(crate) $one_ident:TokenStream,
            )*
            pub(crate) plus_lt:TokenStream,
        }

        impl LifetimeTokens{
            pub(crate) fn new(is_it:IsStaticTrait)->Self{
                let is_static=matches!(is_it, IsStaticTrait::Yes);
                let lt=if is_static { "" }else{ "'lt," };
                let static_lt=if is_static { "" }else{ "'static," };
                let one_lt=if is_static { "'static," }else{ "'lt," };

                LifetimeTokens{
                    $(
                        $ident: {
                            let s=format!("{}{}",lt,$expr);
                            syn::parse_str::<TokenStream>(&s).unwrap()
                        },
                    )*
                    $(
                        $one_ident: {
                            let s=format!("{}{}",one_lt,$one_expr);
                            syn::parse_str::<TokenStream>(&s).unwrap()
                        },
                    )*
                    $(
                        $static_ident: {
                            let s=format!("{}{}",static_lt,$static_expr);
                            syn::parse_str::<TokenStream>(&s).unwrap()
                        },
                    )*
                    plus_lt: syn::parse_str(if is_static { "" }else{ "+ 'lt" }).unwrap(),
                }
            }
        }

    )
}

declare_lifetime_tokens! {
    lifetime_tokens=[
        lt="",
        lt_erasedptr="_ErasedPtr,",
        lt_rbox="__sabi_re::RBox<()>,",
        lt_rref="__sabi_re::RRef<'_sub,()>,",
        lt_rmut="__sabi_re::RMut<'_sub,()>,",
        lt_sub_lt="'_sub,",
    ]
    one_lifetime_tokens=[
        one_lt="",
    ]
    static_lifetime_tokens=[
        staticlt_erasedptr2="_ErasedPtr2,",
        staticlt_erasedptr="_ErasedPtr,",
    ]
}
