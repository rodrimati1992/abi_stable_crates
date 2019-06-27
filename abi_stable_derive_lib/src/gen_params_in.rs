use syn::{
    token::{Comma},
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
    after_lifetimes:Option<AL>,
    after_types:Option<AL>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InWhat {
    /// For the header of an impl block,as in the contents of `impl< ... >`.
    ImplHeader,
    /// For trait/struct/enum/union
    ItemDecl,
    /// For when using a generic as an argument for a trait/struct/enum/union.
    ItemUse,
    /**
For defining the fields of a Dummy struct that is never instantiated,
only using its associated items.
    */
    DummyStruct,
}

impl<'a> GenParamsIn<'a>{
    #[allow(dead_code)]
    pub fn new(generics: &'a Generics,in_what: InWhat)->Self{
        Self{
            generics,
            in_what,
            with_bounds:true,
            after_lifetimes:None,
            after_types:None,
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
            after_types:None,
        }
    }
    pub fn with_after_types(generics: &'a Generics,in_what: InWhat,after_types:AL)->Self{
        Self{
            generics,
            in_what,
            with_bounds:true,
            after_lifetimes:None,
            after_types:Some(after_types),
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
        let with_bounds = 
            self.with_bounds && 
            matches!(InWhat::ImplHeader|InWhat::ItemDecl=self.in_what);

        let with_default = self.in_what == InWhat::ItemDecl;
        
        let in_dummy_struct= self.in_what == InWhat::DummyStruct;

        let mut iter=self.generics.params.iter().peekable();

        let comma=Comma::default();

        while let Some(GenericParam::Lifetime(gen))=iter.peek() {
            iter.next();

            if in_dummy_struct{
                syn::token::And::default().to_tokens(ts);
                gen.lifetime.to_tokens(ts);
                syn::token::Paren::default().surround(ts,|_|());
            }else{
                gen.lifetime.to_tokens(ts);
                if with_bounds {
                    gen.colon_token.to_tokens(ts);
                    gen.bounds.to_tokens(ts);
                }
            }

            comma.to_tokens(ts);
        }

        self.after_lifetimes.to_tokens(ts);

        while let Some(GenericParam::Type(gen))=iter.peek() {
            iter.next();
            
            gen.ident.to_tokens(ts);
            if with_bounds {
                gen.colon_token.to_tokens(ts);
                gen.bounds.to_tokens(ts);
            }
            if with_default {
                gen.eq_token.to_tokens(ts);
                gen.default.to_tokens(ts);
            }
            
            comma.to_tokens(ts);
        }

        self.after_types.to_tokens(ts);

        if !in_dummy_struct{
            while let Some(GenericParam::Const(gen))=iter.peek() {
                iter.next();
                
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
                
                comma.to_tokens(ts);
            }
        }
    }
}




