use std::collections::HashMap;


use syn::{
    ItemImpl,
    ImplItem,
    ImplItemType,
    Visibility,
};

use proc_macro2::TokenStream as TokenStream2;

use quote::{ToTokens,quote,quote_spanned};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::parse_utils::parse_str_as_ident;


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
pub struct UsableTrait{
    pub which_trait:WhichTrait,
    pub name:&'static str,
    pub full_path:&'static str,
}

macro_rules! usable_traits {
    ( 
        $( 
            $field:ident=
            (
                $which_trait:ident,
                $full_path:expr,
                $default_value:expr,
                $usable_by:expr
            ),
        )* 
    ) => (
        pub static TRAIT_LIST:&[UsableTrait]=&[$(
            UsableTrait{
                name:stringify!($which_trait),
                which_trait:WhichTrait::$which_trait,
                full_path:$full_path,
            },
        )*];

        #[repr(u8)]
        #[derive(Debug,Copy,Clone,PartialEq,Eq,Ord,PartialOrd,Hash)]
        pub enum WhichTrait{
            $($which_trait,)*
        }


        impl WhichTrait{
            pub fn default_value(self)->bool{
                match self {
                    $( WhichTrait::$which_trait=>$default_value, )*
                }
            }
            pub fn usable_by(self)->UsableBy{
                match self {
                    $( WhichTrait::$which_trait=>$usable_by, )*
                }
            }
        }


        #[derive(Debug,Copy,Clone,Default)]
        pub struct TraitStruct<T>{
            $(pub $field:T,)*
        }

        impl TraitStruct<UsableTrait>{
            pub const TRAITS:Self=TraitStruct{$(
                $field:UsableTrait{
                    name:stringify!($which_trait),
                    which_trait:WhichTrait::$which_trait,
                    full_path:$full_path,
                },
            )*};
        }

        impl<T> TraitStruct<T>{
            pub fn as_ref(&self)->TraitStruct<&T>{
                TraitStruct{
                    $($field:&self.$field,)*
                }
            }

            pub fn map<F,U>(self,mut f:F)->TraitStruct<U>
            where F:FnMut(WhichTrait,T)->U
            {
                TraitStruct{
                    $($field:f(WhichTrait::$which_trait,self.$field),)*
                }
            }

            pub fn to_vec(self)->Vec<T>{
                vec![
                    $( self.$field ,)*
                ]
            }
        }

        impl<T> ::std::ops::Index<WhichTrait> for TraitStruct<T>{
            type Output=T;
            fn index(&self, index: WhichTrait) -> &Self::Output {
                match index {
                    $( WhichTrait::$which_trait=>&self.$field, )*
                }
            }
        }

        impl<T> ::std::ops::IndexMut<WhichTrait> for TraitStruct<T>{
            fn index_mut(&mut self, index: WhichTrait) -> &mut Self::Output {
                match index {
                    $( WhichTrait::$which_trait=>&mut self.$field, )*
                }
            }
        }
    )
}

use self::{UsableBy as UB};

usable_traits!{
    clone=(Clone,"::std::clone::Clone",false,UB::ROBJECT_AND_DYN_TRAIT),
    default=(Default,"::std::default::Default",false,UB::DYN_TRAIT),
    display=(Display,"::std::fmt::Display",false,UB::DYN_TRAIT),
    debug=(Debug,"::std::fmt::Debug",false,UB::ROBJECT_AND_DYN_TRAIT),
    serialize=(Serialize,"::serde::Serialize",false,UB::DYN_TRAIT),
    eq=(Eq,"::std::cmp::Eq",false,UB::DYN_TRAIT),
    partial_eq=(PartialEq,"::std::cmp::PartialEq",false,UB::DYN_TRAIT),
    ord=(Ord,"::std::cmp::Ord",false,UB::DYN_TRAIT),
    partial_ord=(PartialOrd,"::std::cmp::PartialOrd",false,UB::DYN_TRAIT),
    hash=(Hash,"::std::hash::Hash",false,UB::DYN_TRAIT),
    deserialize=(Deserialize,"::serde::Deserialize",false,UB::DYN_TRAIT),
    send=(Send,"::std::marker::Send",true ,UB::ROBJECT_AND_DYN_TRAIT),
    sync=(Sync,"::std::marker::Sync",true ,UB::ROBJECT_AND_DYN_TRAIT),
    iterator=(Iterator,"::std::iter::Iterator",false,UB::DYN_TRAIT),
    double_ended_iterator=(
        DoubleEndedIterator,"::std::iter::DoubleEndedIterator",false,UB::DYN_TRAIT
    ),
    fmt_write=(FmtWrite,"::std::fmt::Write",false,UB::DYN_TRAIT),
    io_write=(IoWrite,"::std::io::Write",false,UB::DYN_TRAIT),
    io_seek=(IoSeek,"::std::io::Seek",false,UB::DYN_TRAIT),
    io_read=(IoRead,"::std::io::Read",false,UB::DYN_TRAIT),
    io_buf_read=(IoBufRead,"::std::io::BufRead",false,UB::DYN_TRAIT),
    error=(Error,"::std::error::Error",false,UB::DYN_TRAIT),
}

pub(crate) fn private_associated_type()->syn::Ident{
    parse_str_as_ident("define_this_in_the_impl_InterfaceType_macro")
}


pub fn the_macro(mut impl_:ItemImpl)->TokenStream2{
    let interfacetype:syn::Ident=syn::parse_str("InterfaceType").unwrap();

    let mut const_name=(&impl_.self_ty).into_token_stream().to_string();
    const_name.retain(|c| c.is_alphanumeric() );
    const_name.insert_str(0,"_impl_InterfaceType");
    let const_name=parse_str_as_ident(&const_name);
    
    let interface_path_s=impl_.trait_.as_ref().map(|x| &x.1.segments );
    let is_interface_type=interface_path_s
        .and_then(|x| x.last() )
        .map_or(false,|path_| path_.value().ident==interfacetype );

    if !is_interface_type {
        panic!(
            "expected 'impl<...> InterfaceType for {} ' ",
            (&impl_.self_ty).into_token_stream()
        );
    }
    
    let mut default_map=TRAIT_LIST
        .iter()
        .map(|ut|{
            ( parse_str_as_ident(ut.name) , DefaultVal::from(ut.which_trait.default_value()) ) 
        })
        .collect::<HashMap<_,_>>();

    for item in &mut impl_.items {
        match item {
            ImplItem::Type(assoc_ty)=>{
                assert_ne!(
                    assoc_ty.ident,
                    "define_this_in_the_impl_InterfaceType_macro",
                    "you are not supposed to define\n\t\
                     the 'define_this_in_the_impl_InterfaceType_macro' associated type yourself"
                );
                default_map.remove(&assoc_ty.ident);

                let old_ty=&assoc_ty.ty;
                let name=&assoc_ty.ident;
                let span=name.span();

                assoc_ty.ty=type_from_token_stream(
                    quote_spanned!(span=> ImplFrom<#old_ty, trait_marker::#name> )
                );
            }
            _=>{}
        }
    }

    default_map.insert(private_associated_type(),DefaultVal::Hidden);

    for (key,default_) in default_map {
        let mut attrs=Vec::<syn::Attribute>::new();

        let span=key.span();

        let ty=match default_ {
            DefaultVal::False=>quote_spanned!(span=> Unimplemented<trait_marker::#key> ),
            DefaultVal::True=>quote_spanned!(span=> Implemented<trait_marker::#key> ),
            DefaultVal::Hidden=>{
                attrs.extend(parse_syn_attributes("#[doc(hidden)]"));
                quote_spanned!(span=> () )
            },
        }.piped(type_from_token_stream);

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

    quote!(
        const #const_name:()={
            use ::abi_stable::derive_macro_reexports::{
                Implemented,
                Unimplemented,
                ImplFrom,
                trait_marker,
            };

            #impl_
        };
    )
}


fn type_from_token_stream(tts:TokenStream2)->syn::Type{
    let x=syn::TypeVerbatim{tts};
    syn::Type::Verbatim(x)
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