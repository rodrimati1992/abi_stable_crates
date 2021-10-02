/*!
This module defines the CommonTokens type,
used to pass constants of type from `syn` to
many functions in the `abi_stable_derive_lib::stable_abi` module.
*/

use proc_macro2::Span;

use crate::{fn_pointer_extractor::FnParamRet, *};

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

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
            pub(crate) unit_ret:FnParamRet<'a>,
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
                    unit_ret:FnParamRet::unit_ret(arenas),
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
        dot=Dot,
        and_=And,
        add=Add,
        bang=Bang,
        comma=Comma,
        equal=Eq,
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
        ts_empty="",
        und_storage="__Storage,",
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
        comp_tl_function="__CompTLFunction",
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
        field_1to1="__Field1to1",
        comp_tl_fields="__CompTLFields",
        slice_and_field_indices="__SAFI",
        with_field_index="__WithFieldIndex",
        from_vari_field_val="from_vari_field_val",
        instantiate_field="instantiate_field",
        lifetime_indices="lifetime_indices",
        stable_abi="__StableAbi",
        type_identity="TypeIdentity",
        marker_type="MarkerType",
        assert_zero_sized="__assert_zero_sized",
        //layout="LAYOUT",
        get="get",
        stable_abi_bound="__StableAbi_Bound",
        unsafe_extern_fn_type_layout="__UNSAFE_EXTERN_FN_LAYOUT",
        extern_fn_type_layout="__EXTERN_FN_LAYOUT",
        get_type_layout_ctor="__GetTypeLayoutCtor",
        sabi_reexports="_sabi_reexports",
        cmp_ignored="__CmpIgnored",
        lifetime_index="__LifetimeIndex",
        static_equivalent="__GetStaticEquivalent",
        cap_static="Static",
        cap_param="Param",
        cap_const="CONST",
        cap_opaque_field="OPAQUE_FIELD",
        cap_sabi_opaque_field="SABI_OPAQUE_FIELD",
        cap_stable_abi="STABLE_ABI",
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
