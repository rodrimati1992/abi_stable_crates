





use proc_macro2::TokenStream;
use syn::token::Paren;
use quote::{ToTokens};

use crate::common_tokens::LifetimeTokens;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LifetimeIndex {
    Static,
    Param { index: u8 },
}

impl LifetimeIndex {
    /// Produces the tokens of the type_layout::LifetimeIndex version of this type
    pub fn tokenizer<'a>(self,ctokens:&'a LifetimeTokens)->LifetimeIndexTokenizer<'a>{
        LifetimeIndexTokenizer{li:self,ctokens}
    }
}


pub struct LifetimeIndexTokenizer<'a>{
    li:LifetimeIndex,
    ctokens:&'a LifetimeTokens,
}


impl<'a> ToTokens for LifetimeIndexTokenizer<'a> {
    fn to_tokens(&self, ts: &mut TokenStream) {
        let ctokens=self.ctokens;
        match self.li {
            LifetimeIndex::Static=>{
                ctokens.li_static.to_tokens(ts);
            }
            LifetimeIndex::Param{index,..}=>{
                ctokens.li_index.to_tokens(ts);
                Paren::default().surround(ts,|ts| index.to_tokens(ts) );
            }
        }
    }
}

