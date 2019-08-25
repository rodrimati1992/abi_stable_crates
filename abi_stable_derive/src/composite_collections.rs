/*!
Helper types for constructing strings and arrays composed of other strings and arrays.

These datatypes are special-cased for small composite collections ,
whose indices fit in a u16.

*/

use std::{
    borrow::Borrow,
    fmt::Display,
};

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens};

use crate::{
    common_tokens::StartLenTokens,
};

/// A `{start:16,len:u16}` range.
#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd)]
pub struct SmallStartLen{
    pub start:u16,
    pub len:u16,
}

impl SmallStartLen{
    pub const EMPTY:Self=Self{start:0,len:0};

    #[inline]
    pub const fn new(start:u16,len:u16)->Self{
        Self{start,len}
    }

    pub(crate) fn tokenize<'a>(self,ct:&'a StartLenTokens,ts: &mut TokenStream2){
        self.tokenizer(ct).to_tokens(ts);
    }

    #[inline]
    pub(crate) fn tokenizer<'a>(self,ctokens:&'a StartLenTokens)->SmallStartLenTokenizer<'a>{
        SmallStartLenTokenizer{
            start:self.start,
            len:self.len,
            ctokens,
        }
    }
}

pub struct SmallStartLenTokenizer<'a>{
    start:u16,
    len:u16,
    ctokens:&'a StartLenTokens,
}

impl<'a> ToTokens for SmallStartLenTokenizer<'a> {
    fn to_tokens(&self,ts: &mut TokenStream2) {
        use syn::token::{Colon2,Comma,Paren};

        let ct=self.ctokens;
        to_stream!(ts; ct.start_len,Colon2::default(),ct.new );
        Paren::default().surround(ts,|ts|{
            to_stream!(ts; self.start,Comma::default(),self.len );
        });
    }
}


///////////////////////////////////////////////////////////////////////

/// A String-like type,
/// returning a `{start:16,len:u16}` range from methods that extend it.
pub struct SmallCompositeString{
    buffer:String,
}

impl SmallCompositeString{
    pub fn new()->Self{
        Self{
            buffer:String::with_capacity(128),
        }
    }

    fn len(&self)->usize{
        self.buffer.len()
    }

    pub fn push_str(&mut self,s:&str)->SmallStartLen{
        let start=self.len() as u16;
        self.buffer.push_str(s);
        SmallStartLen::new(start,s.len() as u16)
    }

    pub fn push_display<D>(&mut self,s:&D)->SmallStartLen
    where D:Display,
    {
        use std::fmt::Write;
        let start=self.len();
        let _=write!(self.buffer,"{}",s);
        SmallStartLen::new(
            start as u16,
            (self.len()-start) as u16
        )
    }

    #[allow(dead_code)]
    pub fn extend_with_str<I>(&mut self,separator:&str,iter:I)->SmallStartLen
    where
        I:IntoIterator,
        I::Item:Borrow<str>,
    {
        let start=self.len();
        for s in iter {
            self.buffer.push_str(s.borrow());
            self.buffer.push_str(separator);
        }
        SmallStartLen::new(start as u16,(self.len()-start)as u16)
    }

    pub fn extend_with_display<I>(&mut self,separator:&str,iter:I)->SmallStartLen
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
        SmallStartLen::new(start as u16,(self.len()-start)as u16)
    }

    pub fn into_inner(self)->String{
        self.buffer
    }
}

///////////////////////////////////////////////////////////////////////

/// A Vec-like type,
/// returning a `{start:16,len:u16}` range from methods that extend it.
pub struct SmallCompositeVec<T>{
    list:Vec<T>,
}


impl<T> SmallCompositeVec<T>{
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

    pub fn extend<I>(&mut self,iter:I)->SmallStartLen
    where
        I:IntoIterator<Item=T>,
    {
        let start=self.len();
        self.list.extend(iter);
        SmallStartLen::new(start as u16,(self.len()-start)as u16)
    }

    pub fn into_inner(self)->Vec<T>{
        self.list
    }
}