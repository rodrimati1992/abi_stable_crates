use super::*;

use std::{
    borrow::Borrow,
    fmt::Display,
};

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens};


#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd)]
pub struct StartLen{
    pub start:u16,
    pub len:u16,
}

impl StartLen{
    pub const EMPTY:Self=Self{start:0,len:0};

    #[inline]
    pub fn new(start:u16,len:u16)->Self{
        Self{start,len}
    }

    pub(crate) fn tokenize<'a>(self,ct:&'a CommonTokens<'a>,ts: &mut TokenStream2){
        self.tokenizer(ct).to_tokens(ts);
    }

    #[inline]
    pub(crate) fn tokenizer<'a>(self,ctokens:&'a CommonTokens<'a>)->StartLenTokenizer<'a>{
        StartLenTokenizer{
            start:self.start,
            len:self.len,
            ctokens,
        }
    }
}

pub struct StartLenTokenizer<'a>{
    start:u16,
    len:u16,
    ctokens:&'a CommonTokens<'a>,
}

impl<'a> ToTokens for StartLenTokenizer<'a> {
    fn to_tokens(&self,ts: &mut TokenStream2) {
        let ct=self.ctokens;
        to_stream!(ts; ct.start_len,ct.colon2,ct.new );
        ct.paren.surround(ts,|ts|{
            to_stream!(ts; self.start,ct.comma,self.len );
        });
    }
}

///////////////////////////////////////////////////////////////////////

#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd)]
pub struct CompTLFunction<'a>{
    pub(crate) ctokens:&'a CommonTokens<'a>,
    pub(crate) name:StartLen,
    pub(crate) bound_lifetimes:StartLen,
    pub(crate) param_names:StartLen,
    pub(crate) param_abi_infos:StartLen,
    pub(crate) paramret_lifetime_indices:StartLen,
    pub(crate) return_abi_info:Option<u16>,
}


impl<'a> CompTLFunction<'a>{
    pub(crate) fn new(ctokens:&'a CommonTokens)->Self{
        CompTLFunction{
            ctokens,
            name:StartLen::EMPTY,
            bound_lifetimes:StartLen::EMPTY,
            param_names:StartLen::EMPTY,
            param_abi_infos:StartLen::EMPTY,
            paramret_lifetime_indices:StartLen::EMPTY,
            return_abi_info:None,
        }
    }
}


impl<'a> ToTokens for CompTLFunction<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let ct=self.ctokens;
        to_stream!(ts;ct.comp_tl_functions,ct.colon2,ct.new);
        ct.paren.surround(ts,|ts|{
            self.name.tokenize(ct,ts);
            ct.comma.to_tokens(ts);

            self.bound_lifetimes.tokenize(ct,ts);
            ct.comma.to_tokens(ts);

            self.param_names.tokenize(ct,ts);
            ct.comma.to_tokens(ts);

            self.param_abi_infos.tokenize(ct,ts);
            ct.comma.to_tokens(ts);

            self.paramret_lifetime_indices.tokenize(ct,ts);
            ct.comma.to_tokens(ts);

            match self.return_abi_info {
                Some(x) => {
                    ct.rsome.to_tokens(ts);
                    ct.paren.surround(ts,|ts|{
                        x.to_tokens(ts);
                    });
                },
                None => {
                    ct.rnone.to_tokens(ts);
                },
            }
            ct.comma.to_tokens(ts);
        });
    }
}

///////////////////////////////////////////////////////////////////////


pub struct TLFunctionsString{
    buffer:String,
}

impl TLFunctionsString{
    pub fn new()->Self{
        Self{
            buffer:String::with_capacity(128),
        }
    }

    fn len(&self)->usize{
        self.buffer.len()
    }

    pub fn push_str(&mut self,s:&str)->StartLen{
        let start=self.len() as u16;
        self.buffer.push_str(s);
        StartLen::new(start,s.len() as u16)
    }

    pub fn push_display<D>(&mut self,s:&D)->StartLen
    where D:Display,
    {
        use std::fmt::Write;
        let start=self.len();
        let _=write!(self.buffer,"{}",s);
        StartLen::new(
            start as u16,
            (self.len()-start) as u16
        )
    }

    pub fn extend_with_str<I>(&mut self,separator:&str,iter:I)->StartLen
    where
        I:IntoIterator,
        I::Item:Borrow<str>,
    {
        let start=self.len();
        for s in iter {
            self.buffer.push_str(s.borrow());
            self.buffer.push_str(separator);
        }
        StartLen::new(start as u16,(self.len()-start)as u16)
    }

    pub fn extend_with_display<I>(&mut self,separator:&str,iter:I)->StartLen
    where
        I:IntoIterator,
        I::Item:Display,
    {
        use std::fmt::Write;
        let start=self.len();
        for elem in iter {
            let _=write!(self.buffer,"{}",elem);
            self.buffer.push_str(separator);
        }
        StartLen::new(start as u16,(self.len()-start)as u16)
    }

    pub fn into_inner(self)->String{
        self.buffer
    }
}

///////////////////////////////////////////////////////////////////////


pub struct TLFunctionsVec<T>{
    list:Vec<T>,
}


impl<T> TLFunctionsVec<T>{
    pub fn new()->Self{
        Self{
            list:Vec::new(),
        }
    }

    pub fn with_capacity(capacity:usize)->Self{
        Self{
            list:Vec::with_capacity(capacity),
        }
    }

    fn len(&self)->usize{
        self.list.len()
    }

    pub fn push(&mut self,elem:T)->u16{
        let ind=self.len();
        self.list.push(elem);
        ind as u16
    }

    pub fn extend<I>(&mut self,iter:I)->StartLen
    where
        I:IntoIterator<Item=T>,
    {
        let start=self.len();
        self.list.extend(iter);
        StartLen::new(start as u16,(self.len()-start)as u16)
    }

    pub fn into_inner(self)->Vec<T>{
        self.list
    }
}