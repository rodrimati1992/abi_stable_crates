//! This module defines the CommonTokens type,
//! used to pass constants of type from `syn` to
//! many functions in the `abi_stable_derive_lib::stable_abi` module.

use proc_macro2::{Span, TokenStream as TokenStream2};

use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    marker::PhantomData,
};

use crate::Arenas;

macro_rules! declare_common_tokens {
    (
        with_new[ $( $field_new:ident = $token_new:ty , )* ]
        token[ $( $field_token:ident = $token_token:ident , )* ]
        token_streams[ $( $field_ts:ident = $ts_str:expr , )* ]
        types[ $( $field_ty:ident = $ty_str:expr , )* ]
        idents[ $( $field_ident:ident = $ident_str:expr , )* ]
        lifetime[ $( $lifetime_ident:ident = $lifetime_str:expr , )* ]
        str_lits[ $( $strlit_ident:ident = $strlit_str:expr , )* ]
    ) => {
        #[derive(Debug)]
        pub(crate) struct CommonTokens<'a>{
            $( pub(crate) $field_new : $token_new , )*
            $( pub(crate) $field_token : ::syn::token::$token_token , )*
            $( pub(crate) $field_ts : TokenStream2 , )*
            $( pub(crate) $field_ty : ::syn::Type , )*
            $( pub(crate) $field_ident : ::syn::Ident , )*
            $( pub(crate) $lifetime_ident : ::syn::Lifetime , )*
            $( pub(crate) $strlit_ident : ::syn::LitStr , )*
            _marker: PhantomData<&'a ()>,
        }

        impl<'a> CommonTokens<'a>{
            #[allow(unused_variables)]
            pub(crate) fn new(arenas:&'a Arenas)->Self{
                let span=Span::call_site();
                Self{
                    $( $field_new : < $token_new >::new(span) , )*
                    $( $field_token : Default::default() , )*
                    $( $field_ts : ::syn::parse_str($ts_str).expect("BUG") , )*
                    $( $field_ty : ::syn::parse_str($ty_str).expect("BUG") , )*
                    $( $field_ident : ::syn::Ident::new($ident_str,span) , )*
                    $( $lifetime_ident : ::syn::parse_str($lifetime_str).expect("BUG") , )*
                    $( $strlit_ident : ::syn::LitStr::new($strlit_str,span) , )*
                    _marker: PhantomData,
                }
            }
        }

        $(
            impl<'a> AsRef<$token_new> for CommonTokens<'a>{
                fn as_ref(&self)->&$token_new{
                    &self.$field_new
                }
            }
        )*
    }
}

impl<'a> Eq for CommonTokens<'a> {}
impl<'a> PartialEq for CommonTokens<'a> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<'a> PartialOrd for CommonTokens<'a> {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl<'a> Ord for CommonTokens<'a> {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

declare_common_tokens! {
    with_new[
        start_len_tokens=crate::common_tokens::StartLenTokens,
        fn_pointer_tokens=crate::common_tokens::FnPointerTokens,
    ]

    token[
        and_=And,
        comma=Comma,
        equal=Eq,
        colon2=Colon2,
        bracket=Bracket,
        paren=Paren,
        lt=Lt,
        gt=Gt,
    ]

    token_streams[
        und_storage="__Storage,",
    ]

    types[
        empty_tuple="()",
    ]

    idents[
        some="Some",
        none="None",
        new="new",
        comp_tl_fields="__CompTLFields",
        //layout="LAYOUT",
        static_equivalent="__GetStaticEquivalent",
        cap_opaque_field="OPAQUE_FIELD",
        cap_sabi_opaque_field="SABI_OPAQUE_FIELD",
    ]

    lifetime[
        static_lt="'static",
    ]

    str_lits[
    ]
}
