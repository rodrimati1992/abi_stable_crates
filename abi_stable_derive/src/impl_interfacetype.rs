//! The implementation of both the `#[sabi(impl_InterfaceType())]` helper attributes,
//! and the `impl_InterfaceType!{}` macro.

use std::collections::HashMap;

#[allow(unused_imports)]
use core_extensions::SelfOps;

use quote::{quote, quote_spanned, ToTokens};

use syn::Ident;

use as_derive_utils::to_token_fn::ToTokenFnMut;

use crate::parse_utils::parse_str_as_ident;

pub(crate) mod attribute_parsing;
mod macro_impl;

pub(crate) use self::{
    attribute_parsing::{parse_impl_interfacetype, ImplInterfaceType},
    macro_impl::the_macro,
};

//////////////////////

/// The default value for an associated type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DefaultVal {
    /// The value of the associated type is `Unimplemented<trait_marker::AssocTypeName>`
    Unimplemented,
    /// The value of the associated type is `Implemented<trait_marker::AssocTypeName>`
    Implemented,
    /// The associated type is `#[doc(hidden)]`,
    /// to signal to users that the trait is not supposed to be implemented manually,
    Hidden,
}

impl From<bool> for DefaultVal {
    fn from(b: bool) -> Self {
        if b {
            DefaultVal::Implemented
        } else {
            DefaultVal::Unimplemented
        }
    }
}

//////////////////////

/// The trait object implementations (either RObject or DynTrait)
/// that a trait can be used with.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsableBy {
    robject: bool,
    dyn_trait: bool,
}

impl UsableBy {
    pub const DYN_TRAIT: Self = Self {
        robject: false,
        dyn_trait: true,
    };
    pub const ROBJECT_AND_DYN_TRAIT: Self = Self {
        robject: true,
        dyn_trait: true,
    };

    pub const fn robject(&self) -> bool {
        self.robject
    }
    pub const fn dyn_trait(&self) -> bool {
        self.dyn_trait
    }
}

//////////////////////

/// Information about a trait that is usable in RObject and/or DynTrait.
#[derive(Debug, Copy, Clone)]
pub struct UsableTrait {
    pub which_trait: WhichTrait,
    pub name: &'static str,
    pub full_path: &'static str,
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
        /// A list of all the traits usable in RObject and/or DynTrait.
        pub static TRAIT_LIST:&[UsableTrait]=&[$(
            UsableTrait{
                name:stringify!($which_trait),
                which_trait:WhichTrait::$which_trait,
                full_path:$full_path,
            },
        )*];

        /// Represents all the trait usable in `RObject` and/or `DynTrait`,
        /// usable as an index for `TraitStruct`.
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

        /// An generic struct with all the traits usable in RObject and/or DynTrait,
        /// indexable by `WhichTrait`.
        ///
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

            // Bad clippy, you're bad.
            #[allow(clippy::wrong_self_convention)]
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

use self::UsableBy as UB;

usable_traits! {
    clone=(Clone,"::std::clone::Clone",false,UB::ROBJECT_AND_DYN_TRAIT),
    default=(Default,"::std::default::Default",false,UB::DYN_TRAIT),
    display=(Display,"::std::fmt::Display",false,UB::ROBJECT_AND_DYN_TRAIT),
    debug=(Debug,"::std::fmt::Debug",false,UB::ROBJECT_AND_DYN_TRAIT),
    serialize=(Serialize,"::serde::Serialize",false,UB::DYN_TRAIT),
    eq=(Eq,"::std::cmp::Eq",false,UB::DYN_TRAIT),
    partial_eq=(PartialEq,"::std::cmp::PartialEq",false,UB::DYN_TRAIT),
    ord=(Ord,"::std::cmp::Ord",false,UB::DYN_TRAIT),
    partial_ord=(PartialOrd,"::std::cmp::PartialOrd",false,UB::DYN_TRAIT),
    hash=(Hash,"::std::hash::Hash",false,UB::DYN_TRAIT),
    deserialize=(Deserialize,"::serde::Deserialize",false,UB::DYN_TRAIT),
    send=(Send,"::std::marker::Send",false ,UB::ROBJECT_AND_DYN_TRAIT),
    sync=(Sync,"::std::marker::Sync",false ,UB::ROBJECT_AND_DYN_TRAIT),
    iterator=(Iterator,"::std::iter::Iterator",false,UB::DYN_TRAIT),
    double_ended_iterator=(
        DoubleEndedIterator,"::std::iter::DoubleEndedIterator",false,UB::DYN_TRAIT
    ),
    fmt_write=(FmtWrite,"::std::fmt::Write",false,UB::DYN_TRAIT),
    io_write=(IoWrite,"::std::io::Write",false,UB::DYN_TRAIT),
    io_seek=(IoSeek,"::std::io::Seek",false,UB::DYN_TRAIT),
    io_read=(IoRead,"::std::io::Read",false,UB::DYN_TRAIT),
    io_buf_read=(IoBufRead,"::std::io::BufRead",false,UB::DYN_TRAIT),
    error=(Error,"::std::error::Error",false,UB::ROBJECT_AND_DYN_TRAIT),
    unpin=(Unpin,"::std::marker::Unpin",false,UB::ROBJECT_AND_DYN_TRAIT),
}

pub(crate) fn private_associated_type() -> syn::Ident {
    parse_str_as_ident("define_this_in_the_impl_InterfaceType_macro")
}

//////////////////////////////////////////////////////////////////////////////

/// Returns a tokenizer
/// which prints an implementation of InterfaceType for `name`,
/// with `impl_interfacetype` determining the associated types.
pub(crate) fn impl_interfacetype_tokenizer<'a>(
    name: &'a Ident,
    generics: &'a syn::Generics,
    impl_interfacetype: Option<&'a ImplInterfaceType>,
) -> impl ToTokens + 'a {
    ToTokenFnMut::new(move |ts| {
        let ImplInterfaceType { impld, unimpld } = match impl_interfacetype {
            Some(x) => x,
            None => return,
        };

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let const_ident = crate::parse_utils::parse_str_as_ident(&format!(
            "_impl_InterfaceType_constant_{}",
            name,
        ));

        let impld_a = impld;
        let impld_b = impld;

        let unimpld_a = unimpld;
        let unimpld_b = unimpld;

        let priv_assocty = private_associated_type();

        quote!(
            const #const_ident:()={
                use abi_stable::{
                    type_level::{
                        impl_enum::{
                            Implemented as __Implemented,
                            Unimplemented as __Unimplemented,
                        },
                        trait_marker,
                    },
                };
                impl #impl_generics abi_stable::InterfaceType for #name #ty_generics
                #where_clause
                {
                    #( type #impld_a=__Implemented<trait_marker::#impld_b>; )*
                    #( type #unimpld_a=__Unimplemented<trait_marker::#unimpld_b>; )*
                    type #priv_assocty=();
                }
            };
        )
        .to_tokens(ts);
    })
}

//////////////////////////////////////////////////////////////////////////////
