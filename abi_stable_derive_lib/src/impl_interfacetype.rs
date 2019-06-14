use std::collections::HashMap;


use syn::{
    ItemImpl,

    Type as SynType,
    ImplItem,
    ImplItemType,
    Visibility,
};

use proc_macro2::TokenStream as TokenStream2;

use quote::{ToTokens};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::parse_utils::parse_str_as_ident;


//////////////////////


#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum ObjectSafe{
    Yes,
    No,
}


//////////////////////

#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum DefaultVal{
    False,
    True,
    Hidden,
}

impl From<bool> for DefaultVal{
    fn from(b:bool)->Self{
        if b { DefaultVal::True }else{ DefaultVal::False }
    }
}

//////////////////////

struct DefaultValTypes{
    false_:SynType,
    true_:SynType,
}

//////////////////////

/// The types a trait can be used with.
#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct UsableBy{
    robject:bool,
    dyn_trait:bool,
}



impl UsableBy{
    pub const DYN_TRAIT:Self=Self{
        robject:false,
        dyn_trait:true,
    };
    pub const ROBJECT_AND_DYN_TRAIT:Self=Self{
        robject:true,
        dyn_trait:true,
    };

    pub const fn robject(&self)->bool{
        self.robject
    }
    pub const fn dyn_trait(&self)->bool{
        self.dyn_trait
    }
}

//////////////////////

/// A trait usable in either RObject or DynTrait.
#[derive(Debug,Copy,Clone)]
pub struct UsableTrait<N,M>{
    pub name:N,
    pub full_path:M,
    pub default_value:bool,
    pub object_safe:ObjectSafe,
    pub usable_by:UsableBy,
}

impl<N,M> UsableTrait<N,M>{
    #[allow(dead_code)]
    pub fn is_object_safe(&self)->bool{
        self.object_safe==ObjectSafe::Yes
    }
}


macro_rules! usable_traits_slice {
    ( 
        $( 
            ($name:expr,$full_path:expr,$default_value:expr,$object_safe:expr,$usable_by:expr), 
        )* 
    ) => (
        &[$(
            UsableTrait{
                name:$name,
                full_path:$full_path,
                default_value:$default_value,
                object_safe:$object_safe,
                usable_by:$usable_by,
            },
        )*]
    )
}

use self::{UsableBy as UB,ObjectSafe as OS};

pub static TRAIT_LIST:&[UsableTrait<&'static str,&'static str>]=usable_traits_slice![
    ("Clone"              ,"::std::clone::Clone"    ,false,OS::No ,UB::ROBJECT_AND_DYN_TRAIT),
    ("Default"            ,"::std::default::Default",false,OS::No ,UB::DYN_TRAIT),
    ("Display"            ,"::std::fmt::Display"    ,false,OS::Yes,UB::DYN_TRAIT),
    ("Debug"              ,"::std::fmt::Debug"      ,false,OS::Yes,UB::ROBJECT_AND_DYN_TRAIT),
    ("Serialize"          ,"::serde::Serialize"     ,false,OS::No ,UB::DYN_TRAIT),
    ("Eq"                 ,"::std::cmp::Eq"         ,false,OS::Yes,UB::DYN_TRAIT),
    ("PartialEq"          ,"::std::cmp::PartialEq"  ,false,OS::Yes,UB::DYN_TRAIT),
    ("Ord"                ,"::std::cmp::Ord"        ,false,OS::Yes,UB::DYN_TRAIT),
    ("PartialOrd"         ,"::std::cmp::PartialOrd" ,false,OS::Yes,UB::DYN_TRAIT),
    ("Hash"               ,"::std::hash::Hash"      ,false,OS::Yes,UB::DYN_TRAIT),
    ("Deserialize"        ,"::serde::Deserialize"   ,false,OS::No ,UB::DYN_TRAIT),
    ("Send"               ,"::std::marker::Send"    ,true ,OS::Yes,UB::ROBJECT_AND_DYN_TRAIT),
    ("Sync"               ,"::std::marker::Sync"    ,true ,OS::Yes,UB::ROBJECT_AND_DYN_TRAIT),
    ("Iterator"           ,"::std::iter::Iterator"  ,false,OS::Yes,UB::DYN_TRAIT),
    ("DoubleEndedIterator","::std::iter::DoubleEndedIterator",false,OS::Yes,UB::DYN_TRAIT),
    ("FmtWrite"           ,"::std::fmt::Write"      ,false,OS::Yes,UB::DYN_TRAIT),
    ("IoWrite"            ,"::std::io::Write"       ,false,OS::Yes,UB::DYN_TRAIT),
    ("IoSeek"             ,"::std::io::Seek"        ,false,OS::Yes,UB::DYN_TRAIT),
    ("IoRead"             ,"::std::io::Read"        ,false,OS::Yes,UB::DYN_TRAIT),
    ("IoBufRead"          ,"::std::io::BufRead"     ,false,OS::Yes,UB::DYN_TRAIT),
];

pub(crate) fn private_associated_type()->syn::Ident{
    parse_str_as_ident("define_this_in_the_impl_InterfaceType_macro")
}


pub fn the_macro(mut impl_:ItemImpl)->TokenStream2{
    let interfacetype:syn::Ident=syn::parse_str("InterfaceType").unwrap();
    
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
        let parse_type=|s:&str|->SynType{
            let s=format!("abi_stable::type_level::{}",s);
            syn::parse_str(&s).unwrap()
        };
        DefaultValTypes{
            false_:parse_type("bools::False"),
            true_:parse_type("bools::True"),
        }
    };

    let mut default_map=TRAIT_LIST
        .iter()
        .map(|ut|{
            ( parse_str_as_ident(ut.name) , DefaultVal::from(ut.default_value.clone()) ) 
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

    default_map.insert(private_associated_type(),DefaultVal::Hidden);

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