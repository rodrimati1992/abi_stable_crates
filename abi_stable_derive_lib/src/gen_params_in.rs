use syn::{
    token::Comma,
    Generics,GenericParam,
};

use core_extensions::matches;
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::utils::NoTokens;

/// Used to print (with ToTokens) the generc parameter differently depending on where
/// it is being used/declared.
#[derive(Debug, Copy, Clone)]
pub(crate) struct GenParamsIn<'a,AL=NoTokens> {
    pub generics: &'a Generics,
    pub in_what: InWhat,
    pub with_bounds:bool,
    pub after_lifetimes:Option<AL>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InWhat {
    /// For the header of an impl block,as in the contents of `impl< ... >`.
    ImplHeader,
    /// For trait/struct/enum/union
    ItemDecl,
    /// For when using a generic as an argument for a trait/struct/enum/union.
    ItemUse,
}

impl<'a> GenParamsIn<'a>{
    pub fn new(generics: &'a Generics,in_what: InWhat)->Self{
        Self{
            generics,
            in_what,
            with_bounds:true,
            after_lifetimes:None
        }
    }

}

impl<'a,AL> GenParamsIn<'a,AL>{
    pub fn with_after_lifetimes(generics: &'a Generics,in_what: InWhat,after_lifetimes:AL)->Self{
        Self{
            generics,
            in_what,
            with_bounds:true,
            after_lifetimes:Some(after_lifetimes),
        }
    }
    pub fn set_no_bounds(&mut self){
        self.with_bounds=false;
    }
}


impl<'a,AL> ToTokens for GenParamsIn<'a,AL> 
where
    AL:ToTokens,
{
    fn to_tokens(&self, ts: &mut TokenStream) {
        let with_bounds = self.with_bounds && self.in_what != InWhat::ItemUse;
        let with_default = self.in_what == InWhat::ItemDecl;
        
        let mut past_lifetimes=false;

        for generic in &self.generics.params {
            if !past_lifetimes && !matches!(GenericParam::Lifetime{..}=generic) {
                self.after_lifetimes.to_tokens(ts);
                past_lifetimes=true;
            }

            match generic {
                GenericParam::Type(gen) => {
                    gen.ident.to_tokens(ts);
                    if with_bounds {
                        gen.colon_token.to_tokens(ts);
                        gen.bounds.to_tokens(ts);
                    }
                    if with_default {
                        gen.eq_token.to_tokens(ts);
                        gen.default.to_tokens(ts);
                    }
                }
                GenericParam::Lifetime(gen) => {
                    gen.lifetime.to_tokens(ts);
                    if with_bounds {
                        gen.colon_token.to_tokens(ts);
                        gen.bounds.to_tokens(ts);
                    }
                }
                GenericParam::Const(gen) => {
                    if self.in_what != InWhat::ItemUse {
                        gen.const_token.to_tokens(ts);
                    }
                    gen.ident.to_tokens(ts);
                    if with_bounds {
                        gen.colon_token.to_tokens(ts);
                        gen.ty.to_tokens(ts);
                    }
                    if with_default {
                        gen.eq_token.to_tokens(ts);
                        gen.default.to_tokens(ts);
                    }
                }
            }
            Comma::default().to_tokens(ts);
        }

    }
}




