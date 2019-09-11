/*!
Helper types for constructing strings and arrays composed of other strings and arrays.

These datatypes are special-cased for small composite collections ,
whose indices fit in a u16.

*/

use std::{
    borrow::Borrow,
    convert::TryFrom,
    fmt::{Debug,Display},
    marker::PhantomData,
    ops::{Add,Range},
};

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens};

use crate::{
    common_tokens::StartLenTokens,
};


/// A `{start:16,len:u16}` range.
pub type SmallStartLen=StartLen<u16>;


/// A `{start:N,len:N}` range.
#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd)]
pub struct StartLen<N>{
    pub start:N,
    pub len:N,
}

macro_rules! start_len_constructors {
    ( $($int_ty:ty),* ) => (
        impl StartLen<u16>{
            pub const EMPTY:Self=Self{start:0,len:0};
        }
    )
}

start_len_constructors!{u16,usize}


impl<N> StartLen<N>{
    #[inline]
    pub(crate) fn new(start:usize,len:usize)->Self
    where
        N:TryFrom<usize>,
        N::Error:Debug,
    {
        Self{
            start: N::try_from(start).unwrap(),
            len: N::try_from(len).unwrap(),
        }
    }

    pub(crate) fn tokenize<'a>(self,ct:&'a StartLenTokens,ts: &mut TokenStream2)
    where
        N:ToTokens
    {
        self.tokenizer(ct).to_tokens(ts);
    }

    #[allow(dead_code)]
    pub(crate) fn into_range(self)->Range<N>
    where
        N:Copy+Add<N,Output=N>
    {
        self.start..(self.start+self.len)
    }

    #[inline]
    pub(crate) fn tokenizer<'a>(self,ctokens:&'a StartLenTokens)->StartLenTokenizer<'a,N>{
        StartLenTokenizer{
            start:self.start,
            len:self.len,
            ctokens,
        }
    }
}

pub struct StartLenTokenizer<'a,N>{
    start:N,
    len:N,
    ctokens:&'a StartLenTokens,
}

impl<'a,N> ToTokens for StartLenTokenizer<'a,N> 
where
    N:ToTokens
{
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


pub type SmallCompositeString=CompositeString<u16>;


/// A String-like type,
/// returning a `{start:16,len:u16}` range from methods that extend it.
pub struct CompositeString<N>{
    buffer:String,
    _integer:PhantomData<N>,
}

impl<N> CompositeString<N>
where
    N:TryFrom<usize>,
    N::Error:Debug,
{
    pub fn new()->Self{
        Self{
            buffer:String::with_capacity(128),
            _integer:PhantomData,
        }
    }

    fn len(&self)->usize{
        self.buffer.len()
    }

    pub fn push_str(&mut self,s:&str)->StartLen<N>{
        let start=self.len();
        self.buffer.push_str(s);
        StartLen::new(start,s.len())
    }

    pub fn push_display<D>(&mut self,s:&D)->StartLen<N>
    where D:Display,
    {
        use std::fmt::Write;
        let start=self.len();
        let _=write!(self.buffer,"{}",s);
        StartLen::new(start,self.len()-start)
    }

    #[allow(dead_code)]
    pub fn extend_with_str<I>(&mut self,separator:&str,iter:I)->StartLen<N>
    where
        I:IntoIterator,
        I::Item:Borrow<str>,
    {
        let start=self.len();
        for s in iter {
            self.buffer.push_str(s.borrow());
            self.buffer.push_str(separator);
        }
        StartLen::new(start,self.len()-start)
    }

    pub fn extend_with_display<I>(&mut self,separator:&str,iter:I)->StartLen<N>
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
        StartLen::new(start,self.len()-start)
    }

    pub fn into_inner(self)->String{
        self.buffer
    }
}

///////////////////////////////////////////////////////////////////////

pub type SmallCompositeVec<T>=CompositeVec<T,u16>;

/// A Vec-like type,
/// returning a `{start:16,len:u16}` range from methods that extend it.
pub struct CompositeVec<T,N>{
    list:Vec<T>,
    _integer:PhantomData<N>,
}


impl<T,N> CompositeVec<T,N>
where
    N:TryFrom<usize>,
    N::Error:Debug,
{
    pub fn new()->Self{
        Self{
            list:Vec::new(),
            _integer:PhantomData,
        }
    }

    pub fn with_capacity(capacity:usize)->Self{
        Self{
            list:Vec::with_capacity(capacity),
            _integer:PhantomData,
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

    pub fn extend<I>(&mut self,iter:I)->StartLen<N>
    where
        I:IntoIterator<Item=T>,
    {
        let start=self.len();
        self.list.extend(iter);
        StartLen::new(start,self.len()-start)
    }

    pub fn into_inner(self)->Vec<T>{
        self.list
    }
}