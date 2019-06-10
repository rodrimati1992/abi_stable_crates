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
        pound=Pound,
        lt=Lt,
        gt=Gt,
        as_=As,
    ]

    token_streams[

    ]

    types[
        empty_tuple="()",
    ]

    idents[
        cratename="abi_stable",
        some="Some",
        none="None",
        rsome="__RSome",
        rnone="__RNone",
        tl_field="__TLField",
        tl_functions="__TLFunctions",
        comp_tl_functions="__CompTLFunction",
        tl_enum_variant="__TLEnumVariant",
        start_len="__StartLen",
        value_kind ="__ValueKind",
        prefix_kind="__PrefixKind",
        tl_data="__TLData",
        struct_="struct",
        struct_under="struct_derive",
        union_under="union_derive",
        enum_under="enum_derive",
        prefix_type="prefix_type_derive",
        cap_repr_transparent="ReprTransparent",
        cap_prefix_type="PrefixType",
        new="new",
        env="env",
        name="name",
        fields="fields",
        get_abi_info="__GetAbiInfo",
        field_1to1="__Field1to1",
        tl_fields="__TLFields",
        slice_and_field_indices="__SAFI",
        with_field_index="__WithFieldIndex",
        from_vari_field_val="from_vari_field_val",
        instantiate_field="instantiate_field",
        lifetime_indices="lifetime_indices",
        make_get_abi_info="__MakeGetAbiInfo",
        stable_abi="__StableAbi",
        shared_stable_abi="__SharedStableAbi",
        type_identity="TypeIdentity",
        marker_type="MarkerType",
        assert_zero_sized="__assert_zero_sized",
        abi_info="ABI_INFO",
        get="get",
        stable_abi_bound="__StableAbi_Bound",
        unsafe_opaque_field_bound="__UnsafeOpaqueField_Bound",
        unsafe_extern_fn_abi_info="__UNSAFE_EXTERN_FN_ABI_INFO",
        extern_fn_abi_info="__EXTERN_FN_ABI_INFO",
        make_get_abi_info_sa="__sabi_MakeGetAbiInfoSA",
        sabi_reexports="_sabi_reexports",
        cmp_ignored="__CmpIgnored",
        lifetime_index="__LifetimeIndex",
        static_equivalent="__StaticEquivalent",
        li_static="__LIStatic",
        li_index="__LIParam",
        cap_static="Static",
        cap_param="Param",
        cap_const="CONST",
        subfields="subfields",
        with_functions="with_functions",
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
