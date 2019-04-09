use proc_macro2::Span;


use crate::{
    *,
    fn_pointer_extractor::FnParamRet,
};

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

macro_rules! declare_common_tokens {
    (
        with_new[ $( $field_new:ident = $token_new:ident , )* ]
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
            $( pub(crate) $field_ts : TokenStream , )*
            $( pub(crate) $field_ty : ::syn::Type , )*
            $( pub(crate) $field_ident : ::syn::Ident , )*
            $( pub(crate) $lifetime_ident : ::syn::Lifetime , )*
            $( pub(crate) $strlit_ident : ::syn::LitStr , )*
            pub(crate) unit_ret:FnParamRet<'a>,
        }

        impl<'a> CommonTokens<'a>{
            #[allow(unused_variables)]
            pub(crate) fn new(arenas:&'a Arenas)->Self{
                let span=Span::call_site();
                Self{
                    $( $field_new : $token_new::new(span) , )*
                    $( $field_token : Default::default() , )*
                    $( $field_ts : ::syn::parse_str($ts_str).unwrap() , )*
                    $( $field_ty : ::syn::parse_str($ty_str).unwrap() , )*
                    $( $field_ident : ::syn::Ident::new($ident_str,span) , )*
                    $( $lifetime_ident : ::syn::parse_str($lifetime_str).unwrap() , )*
                    $( $strlit_ident : ::syn::LitStr::new($strlit_str,span) , )*
                    unit_ret:FnParamRet::unit_ret(arenas),
                }
            }
        }
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

    ]

    token[
        dot=Dot,
        and_=And,
        bang=Bang,
        comma=Comma,
        semicolon=Semi,
        colon=Colon,
        colon2=Colon2,
        brace=Brace,
        bracket=Bracket,
        paren=Paren,
        lt=Lt,
        gt=Gt,
        as_=As,
    ]

    token_streams[

    ]

    types[

    ]

    idents[
        cratename="abi_stable",
        tl_field="__TLField",
        tl_enum_variant="__TLEnumVariant",
        tl_data="__TLData",
        struct_under="struct_",
        enum_under="enum_",
        cap_repr_transparent="ReprTransparent",
        new="new",
        env="env",
        name="name",
        get_abi_info="__GetAbiInfo",
        instantiate_field="instantiate_field",
        lifetime_indices="lifetime_indices",
        make_get_abi_info="__MakeGetAbiInfo",
        stable_abi="__StableAbi",
        type_identity="TypeIdentity",
        marker_type="MarkerType",
        assert_zero_sized="__assert_zero_sized",
        abi_info="ABI_INFO",
        get="get",
        stable_abi_bound="__StableAbi_Bound",
        unsafe_opaque_field_bound="__UnsafeOpaqueField_Bound",
        sabi_reexports="_sabi_reexports",
        cmp_ignored="__CmpIgnored",
        lifetime_index="__LifetimeIndex",
        li_static="__LIStatic",
        li_index="__LIParam",
        cap_static="Static",
        cap_param="Param",
        cap_const="CONST",
        underscore="_",
        for_="for",
        static_="static",
        stringify_="stringify",
    ]

    lifetime[
        underscore_lt="'_",
        static_lt="'static",
    ]

    str_lits[
        c_abi_lit="C",
    ]
}
