use std::{
    fmt::Display,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr,
};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::spanned::Spanned;

////////////////////////////////////////////////////////////////////////////////

pub trait SynErrorExt: Sized {
    fn into_syn_err(self) -> syn::Error;

    fn prepend_msg<M>(self, msg: M) -> syn::Error
    where
        M: AsRef<str>,
    {
        let e = self.into_syn_err();
        syn::Error::new(e.span(), format!("{}{}", msg.as_ref(), e))
    }
}

impl SynErrorExt for syn::Error {
    #[inline(always)]
    fn into_syn_err(self) -> syn::Error {
        self
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NoTokens;

impl ToTokens for NoTokens {
    fn to_tokens(&self, _: &mut TokenStream2) {}
}

////////////////////////////////////////////////////////////////////////////////

pub trait SynResultExt {
    fn push_err(&mut self, err: syn::Error);
    fn combine_err<T>(&mut self, res: Result<T, syn::Error>);
    fn combine_into_err<T>(self, into: &mut Result<T, syn::Error>);
}

impl<T> SynResultExt for Result<T, syn::Error> {
    fn push_err(&mut self, err: syn::Error) {
        match self {
            this @ Ok(_) => *this = Err(err),
            Err(e) => e.combine(err),
        }
    }

    fn combine_err<T2>(&mut self, res: Result<T2, syn::Error>) {
        if let Err(err) = res {
            self.push_err(err);
        }
    }

    fn combine_into_err<T2>(self, into: &mut Result<T2, syn::Error>) {
        into.combine_err(self);
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A result wrapper which panics if it's the error variant is not handled,
/// by calling `.into_result()`.
#[derive(Debug, Clone)]
pub struct LinearResult<T> {
    errors: ManuallyDrop<Result<T, syn::Error>>,
}

impl<T> Drop for LinearResult<T> {
    fn drop(&mut self) {
        let res = unsafe { take_manuallydrop(&mut self.errors) };
        res.expect("Expected LinearResult to be handled");
    }
}

impl<T> LinearResult<T> {
    #[inline]
    pub fn new(res: Result<T, syn::Error>) -> Self {
        Self {
            errors: ManuallyDrop::new(res),
        }
    }

    #[inline]
    pub fn ok(value: T) -> Self {
        Self::new(Ok(value))
    }
}

impl<T> Default for LinearResult<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(Ok(T::default()))
    }
}

impl<T> From<Result<T, syn::Error>> for LinearResult<T> {
    #[inline]
    fn from(res: Result<T, syn::Error>) -> Self {
        Self::new(res)
    }
}

impl<T> Deref for LinearResult<T> {
    type Target = Result<T, syn::Error>;

    fn deref(&self) -> &Result<T, syn::Error> {
        &self.errors
    }
}

impl<T> DerefMut for LinearResult<T> {
    fn deref_mut(&mut self) -> &mut Result<T, syn::Error> {
        &mut self.errors
    }
}

impl<T> From<LinearResult<T>> for Result<T, syn::Error> {
    #[inline]
    fn from(this: LinearResult<T>) -> Result<T, syn::Error> {
        this.into_result()
    }
}

#[allow(dead_code)]
impl<T> LinearResult<T> {
    #[inline]
    pub fn into_result(self) -> Result<T, syn::Error> {
        let mut this = ManuallyDrop::new(self);
        unsafe { take_manuallydrop(&mut this.errors) }
    }

    #[inline]
    pub fn take(&mut self) -> Result<T, syn::Error>
    where
        T: Default,
    {
        self.replace(Ok(Default::default()))
    }

    #[inline]
    pub fn replace(&mut self, other: Result<T, syn::Error>) -> Result<T, syn::Error> {
        mem::replace(&mut *self.errors, other)
    }
}

impl<T> SynResultExt for LinearResult<T> {
    #[inline]
    fn push_err(&mut self, err: syn::Error) {
        self.errors.push_err(err);
    }

    #[inline]
    fn combine_err<T2>(&mut self, res: Result<T2, syn::Error>) {
        self.errors.combine_err(res);
    }

    #[inline]
    fn combine_into_err<T2>(self, into: &mut Result<T2, syn::Error>) {
        self.into_result().combine_into_err(into);
    }
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

////////////////////////////////////////////////////////////////////////////////

pub fn spanned_err(tokens: &dyn ToTokens, display: &dyn Display) -> syn::Error {
    syn::Error::new_spanned(tokens, display)
}

#[allow(dead_code)]
pub fn syn_err(span: Span, display: &dyn Display) -> syn::Error {
    syn::Error::new(span, display)
}

////////////////////////////////////////////////////////////////////////////////

pub fn join_spans<I, T>(iter: I) -> Span
where
    I: IntoIterator<Item = T>,
    T: Spanned,
{
    let call_site = Span::call_site();
    let mut iter = iter.into_iter();
    let first: Span = match iter.next() {
        Some(x) => x.span(),
        None => return call_site,
    };

    iter.fold(first, |l, r| l.join(r.span()).unwrap_or(call_site))
}

////////////////////////////////////////////////////////////////////////////////

#[inline(never)]
pub fn dummy_ident() -> syn::Ident {
    syn::Ident::new("DUMMY_IDENT", Span::call_site())
}

////////////////////////////////////////////////////////////////////////////////

pub fn type_from_ident(ident: syn::Ident) -> syn::Type {
    let path: syn::Path = ident.into();
    let path = syn::TypePath { qself: None, path };
    path.into()
}

pub fn expr_from_ident(ident: syn::Ident) -> syn::Expr {
    let x = syn::Path::from(ident);
    let x = syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: x,
    };
    syn::Expr::Path(x)
}

/// Used to tokenize an integer without a type suffix.
pub fn expr_from_int(int: u64) -> syn::Expr {
    let x = proc_macro2::Literal::u64_unsuffixed(int);
    let x = syn::LitInt::from(x);
    let x = syn::Lit::Int(x);
    let x = syn::ExprLit {
        attrs: Vec::new(),
        lit: x,
    };
    syn::Expr::Lit(x)
}

/// Used to tokenize an integer without a type suffix.
/// This one should be cheaper than `expr_from_int`.
pub fn uint_lit(int: u64) -> syn::LitInt {
    let x = proc_macro2::Literal::u64_unsuffixed(int);
    syn::LitInt::from(x)
}
