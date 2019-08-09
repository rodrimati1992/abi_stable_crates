use proc_macro2::TokenStream;
use syn::token::Paren;
use quote::{ToTokens};

use crate::common_tokens::LifetimeTokens;


/// Represents a lifetime used within a type definition.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LifetimeIndex {
    /// The 'static lifetime.
    Static,
    /// Represents the nth lifetime parameter of the type definition,
    /// 
    /// When used inside a type definition,
    /// it also refers to the lifetimes introduces by using `for<'a>`,
    /// with indices higher than those of the data structure itself.
    ///
    /// Eg:`LifetimeIndex::Param{index:0}` refers to the `'w` lifetime in `struct Foo<'w,'x>` .
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

