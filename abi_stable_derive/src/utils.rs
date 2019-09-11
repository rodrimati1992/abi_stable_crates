use std::{
    fmt::Display,
    mem::{self,ManuallyDrop},
    ops::{Deref,DerefMut},
    ptr,
    time::Instant,
};

use abi_stable_shared::test_utils::{FileSpan};

use core_extensions::measure_time::MyDuration;
use quote::{ToTokens};
use proc_macro2::{Span,TokenStream as TokenStream2};


#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub struct NoTokens;

impl ToTokens for NoTokens {
    fn to_tokens(&self, _: &mut TokenStream2) {}
}


#[allow(dead_code)]
pub struct PrintDurationOnDrop{
    start:Instant,
    file_span:FileSpan,
}

impl PrintDurationOnDrop{
    #[allow(dead_code)]
    pub fn new(file_span:FileSpan)->Self{
        Self{
            start:Instant::now(),
            file_span,
        }
    }
}

impl Drop for PrintDurationOnDrop{
    fn drop(&mut self){
        let span=self.file_span;
        let dur:MyDuration=self.start.elapsed().into();
        println!("{}-{}:taken {} to run",span.file,span.line,dur);
    }
}


////////////////////////////////////////////////////////////////////////////////

#[inline(never)]
pub(crate) fn dummy_ident()->syn::Ident{
    syn::Ident::new("DUMMY_IDENT",Span::call_site())
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn type_from_ident(ident: syn::Ident) -> syn::Type {
    let path: syn::Path = ident.into();
    let path = syn::TypePath { qself: None, path };
    path.into()
}

pub(crate) fn expr_from_ident(ident:syn::Ident)->syn::Expr{
    let x=syn::Path::from(ident);
    let x=syn::ExprPath{
        attrs:Vec::new(),
        qself:None,
        path:x,
    };
    syn::Expr::Path(x)
}

/// Used to tokenize an integer without a type suffix.
pub(crate) fn expr_from_int(int:u64)->syn::Expr{
    let x=proc_macro2::Literal::u64_unsuffixed(int);
    let x=syn::LitInt::from(x);
    let x=syn::Lit::Int(x);
    let x=syn::ExprLit{attrs:Vec::new(),lit:x};
    let x=syn::Expr::Lit(x);
    x
}

/// Used to tokenize an integer without a type suffix.
/// This one should be cheaper than `expr_from_int`.
pub(crate) fn uint_lit(int:u64)->syn::LitInt{
    let x=proc_macro2::Literal::u64_unsuffixed(int);
    syn::LitInt::from(x)
}


////////////////////////////////////////////////////////////////////////////////


pub(crate)trait SynPathExt{
    fn equals_str(&self,s:&str)->bool;
    fn equals_ident(&self,s:&syn::Ident)->bool;
    fn into_ident(self)->Result<syn::Ident,Self>
    where Self:Sized;
}

impl SynPathExt for syn::Path{
    fn equals_str(&self,s:&str)->bool{
        match self.get_ident() {
            Some(ident)=>ident==s,
            None=>false,
        }
    }
    fn equals_ident(&self,s:&syn::Ident)->bool{
        self.get_ident()==Some(s)
    }
    fn into_ident(mut self)->Result<syn::Ident,Self>{
        if self.segments.len()==1 {
            Ok(self.segments.pop().expect("TEST BUG").into_value().ident)
        }else{
            Err(self)
        }
    }
}


////////////////////////////////////////////////////////////////////////////////


pub(crate) trait SynResultExt{
    fn push_err(&mut self,err:syn::Error);
    fn combine_err<T>(&mut self,res:Result<T,syn::Error>);
    fn combine_into_err<T>(self,into:&mut Result<T,syn::Error>);
}

impl<T> SynResultExt for Result<T,syn::Error>{
    fn push_err(&mut self,err:syn::Error){
        match self {
            this@Ok(_)=>*this=Err(err),
            Err(e)=>e.combine(err),
        }
    }
    fn combine_err<T2>(&mut self,res:Result<T2,syn::Error>) {
        if let Err(err)=res {
            self.push_err(err);
        }
    }
    fn combine_into_err<T2>(self,into:&mut Result<T2,syn::Error>){
        into.combine_err(self);
    }
}


////////////////////////////////////////////////////////////////////////////////


/// A result wrapper which panics if it's the error variant is not handled,
/// by calling `.into_result()`.
#[derive(Debug,Clone)]
pub(crate) struct LinearResult<T>{
    errors:ManuallyDrop<Result<T,syn::Error>>,
}

impl<T> Drop for LinearResult<T>{
    fn drop(&mut self){
        let res=unsafe{ take_manuallydrop(&mut self.errors) };
        res.expect("Expected LinearResult to be handled");
    }
}

impl<T> LinearResult<T>{
    #[inline]
    pub(crate) fn new(res:Result<T,syn::Error>)->Self{
        Self{
            errors:ManuallyDrop::new(res),
        }
    }

    #[inline]
    pub(crate) fn ok(value:T)->Self{
        Self::new(Ok(value))
    }
}

impl<T> Default for LinearResult<T>
where
    T:Default
{
    fn default()->Self{
        Self::new(Ok(T::default()))
    }
}

impl<T> From<Result<T,syn::Error>> for LinearResult<T>{
    #[inline]
    fn from(res:Result<T,syn::Error>)->Self{
        Self::new(res)
    }
}

impl<T> Deref for LinearResult<T>{
    type Target=Result<T,syn::Error>;

    fn deref(&self)->&Result<T,syn::Error>{
        &self.errors
    }
}

impl<T> DerefMut for LinearResult<T>{
    fn deref_mut(&mut self)->&mut Result<T,syn::Error>{
        &mut self.errors
    }
}


impl<T> Into<Result<T,syn::Error>> for LinearResult<T>{
    #[inline]
    fn into(self)->Result<T,syn::Error>{
        self.into_result()
    }
}

#[allow(dead_code)]
impl<T> LinearResult<T>{
    #[inline]
    pub(crate) fn into_result(self)->Result<T,syn::Error>{
        let mut this=ManuallyDrop::new(self);
        unsafe{ take_manuallydrop(&mut this.errors) }
    }

    #[inline]
    pub(crate) fn take(&mut self)->Result<T,syn::Error>
    where
        T:Default
    {
        self.replace(Ok(Default::default()))
    }

    #[inline]
    pub(crate) fn replace(&mut self,other:Result<T,syn::Error>)->Result<T,syn::Error>{
        mem::replace(&mut *self.errors,other)
    }
}

impl<T> SynResultExt for LinearResult<T>{
    #[inline]
    fn push_err(&mut self,err:syn::Error){
        self.errors.push_err(err);
    }
    #[inline]
    fn combine_err<T2>(&mut self,res:Result<T2,syn::Error>) {
        self.errors.combine_err(res);
    }
    #[inline]
    fn combine_into_err<T2>(self,into:&mut Result<T2,syn::Error>){
        self.into_result().combine_into_err(into);
    }
}


////////////////////////////////////////////////////////////////////////////////


pub(crate) fn spanned_err(tokens:&dyn ToTokens, display:&dyn Display)-> syn::Error {
    syn::Error::new_spanned(tokens,display)
}

#[allow(dead_code)]
pub(crate) fn syn_err(span:Span,display:&dyn Display)-> syn::Error {
    syn::Error::new(span,display)
}


////////////////////////////////////////////////////////////////////////////////


/// Takes the contents out of a `ManuallyDrop<T>`.
///
/// # Safety
///
/// After this function is called `slot` will become uninitialized and 
/// must not be read again.
pub unsafe fn take_manuallydrop<T>(slot: &mut ManuallyDrop<T>) -> T {
    ManuallyDrop::into_inner(ptr::read(slot))
}