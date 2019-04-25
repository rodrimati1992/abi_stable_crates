use std::collections::HashMap;


use syn::{
    ItemImpl,

    Type as SynType,
    Ident,
    ImplItem,
    ImplItemType,
    Visibility,
};

use proc_macro2::TokenStream as TokenStream2;

use quote::{ToTokens};

#[allow(unused_imports)]
use core_extensions::prelude::*;



#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
enum DefaultVal{
    False,
    True,
    Hidden,
}

pub struct DefaultValTypes{
    false_:SynType,
    true_:SynType,
}


pub fn the_macro(mut impl_:ItemImpl)->TokenStream2{
    let interfacetype:syn::Ident=syn::parse_str("InterfaceType").unwrap();
    
    static TRAIT_LIST:&[(&str,DefaultVal)]=&[
        ("Clone",DefaultVal::False),
        ("Default",DefaultVal::False),
        ("Display",DefaultVal::False),
        ("Debug",DefaultVal::False),
        ("Serialize",DefaultVal::False),
        ("Eq",DefaultVal::False),
        ("PartialEq",DefaultVal::False),
        ("Ord",DefaultVal::False),
        ("PartialOrd",DefaultVal::False),
        ("Hash",DefaultVal::False),
        ("Deserialize",DefaultVal::False),
        // ("Send",DefaultVal::True),
        // ("Sync",DefaultVal::True),
        ("define_this_in_the_impl_InterfaceType_macro",DefaultVal::Hidden),
    ];

    let interface_path_s=impl_.trait_.as_ref().map(|x| &x.1.segments );
    let is_interface_type=interface_path_s
        .and_then(|x| x.last() )
        .map_or(false,|path_| path_.value().ident==interfacetype );

    let defval_paths=if !is_interface_type {
        panic!(
            "expected 'impl<...> InterfaceType for {} ' ",
            (&impl_.self_ty).into_token_stream()
        );
    }else{
        let interface_path_s=interface_path_s.unwrap();
        let prefix=if interface_path_s.len()>=1 && interface_path_s[0].ident=="crate" {
            "crate::type_level"
        }else{
            "abi_stable::type_level"
        };
        let parse_type=|s:&str|->SynType{
            let s=format!("{}::{}",prefix,s);
            syn::parse_str(&s).unwrap()
        };
        DefaultValTypes{
            false_:parse_type("bools::False"),
            true_:parse_type("bools::True"),
        }
    };

    let mut default_map=TRAIT_LIST
        .iter()
        .map(|(trait_,ty)|{
            ( syn::parse_str::<Ident>(trait_).unwrap() , ty.clone() )
        })
        .collect::<HashMap<_,_>>();

    for item in &impl_.items {
        match item {
            ImplItem::Type(assoc_ty)=>{
                assert_ne!(
                    assoc_ty.ident,
                    "define_this_in_the_impl_InterfaceType_macro",
                    "you are not supposed to define\n\t\
                     the 'define_this_in_the_impl_InterfaceType_macro' associated type yourself"
                );
                default_map.remove(&assoc_ty.ident);
            }
            _=>{}
        }
    }

    for (key,default_) in default_map {
        let mut attrs=Vec::<syn::Attribute>::new();

        let ty=match default_ {
            DefaultVal::False=>&defval_paths.false_,
            DefaultVal::True=>&defval_paths.true_,
            DefaultVal::Hidden=>{
                attrs.extend(parse_syn_attributes("#[doc(hidden)]"));
                &defval_paths.false_
            },
        }.clone();

        let defaulted=ImplItemType{
            attrs,
            vis: Visibility::Inherited,
            defaultness: None,
            type_token: Default::default(),
            ident:key,
            generics: Default::default(),
            eq_token: Default::default(),
            ty,
            semi_token: Default::default(),
        };
        impl_.items.push(ImplItem::Type(defaulted))
    }

    impl_.into_token_stream()
}







pub fn parse_syn_attributes(str_: &str) -> Vec<syn::Attribute> {
    syn::parse_str::<ParseOuter>(str_).unwrap().attributes
}


struct ParseOuter {
    attributes: Vec<syn::Attribute>,
}

impl syn::parse::Parse for ParseOuter {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        Ok(Self {
            attributes: syn::Attribute::parse_outer(input)?,
        })
    }
}