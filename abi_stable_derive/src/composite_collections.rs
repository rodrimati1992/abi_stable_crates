//! Helper types for constructing strings and arrays composed of other strings and arrays.
//!
//! These datatypes are special-cased for small composite collections ,
//! whose indices fit in a u16.

use std::{
    borrow::Borrow,
    convert::TryFrom,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Add, Range},
};

use as_derive_utils::{return_syn_err, to_stream};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;

use crate::common_tokens::StartLenTokens;

/// A `{start:16,len:u16}` range.
pub type SmallStartLen = StartLen<u16>;

/// A `{start:N,len:N}` range.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct StartLen<N> {
    pub start: N,
    pub len: N,
}

impl StartLen<u16> {
    abi_stable_shared::declare_start_len_bit_methods! {}
}

impl<N> StartLen<N> {
    #[inline]
    pub(crate) fn from_start_len(start: usize, len: usize) -> Self
    where
        N: TryFrom<usize>,
        N::Error: Debug,
    {
        Self {
            start: N::try_from(start).unwrap(),
            len: N::try_from(len).unwrap(),
        }
    }

    #[inline]
    pub const fn new(start: N, len: N) -> Self {
        Self { start, len }
    }

    #[allow(dead_code)]
    pub(crate) fn into_range(self) -> Range<N>
    where
        N: Copy + Add<N, Output = N>,
    {
        self.start..(self.start + self.len)
    }

    #[inline]
    pub(crate) fn tokenizer(self, ctokens: &StartLenTokens) -> StartLenTokenizer<'_, N> {
        StartLenTokenizer {
            start: self.start,
            len: self.len,
            ctokens,
        }
    }
}

impl StartLen<u16> {
    pub const DUMMY: Self = Self {
        start: (1u16 << 15) + 1,
        len: (1u16 << 15) + 1,
    };

    pub const EMPTY: Self = Self { start: 0, len: 0 };

    /// The start of this range.
    #[inline]
    pub const fn start(self) -> usize {
        self.start as usize
    }

    #[inline]
    pub const fn len(self) -> usize {
        self.len as usize
    }

    /// Converts this StartLen to a u32.
    pub const fn to_u32(self) -> u32 {
        self.start as u32 | ((self.len as u32) << 16)
    }

    pub fn check_ident_length(&self, span: Span) -> Result<(), syn::Error> {
        if self.len > Self::IDENT_MAX_LEN {
            return_syn_err!(
                span,
                "Identifier is too long,it must be at most {} bytes.",
                Self::IDENT_MAX_LEN,
            );
        }
        Ok(())
    }
}

pub struct StartLenTokenizer<'a, N> {
    start: N,
    len: N,
    ctokens: &'a StartLenTokens,
}

impl<'a, N> ToTokens for StartLenTokenizer<'a, N>
where
    N: ToTokens,
{
    fn to_tokens(&self, ts: &mut TokenStream2) {
        use syn::token::{Colon2, Comma, Paren};

        let ct = self.ctokens;
        to_stream!(ts; ct.start_len,Colon2::default(),ct.new );
        Paren::default().surround(ts, |ts| {
            to_stream!(ts; self.start,Comma::default(),self.len );
        });
    }
}

///////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub type SmallCompositeString = CompositeString<u16>;

/// A String-like type,
/// returning a `{start:16,len:u16}` range from methods that extend it.
pub struct CompositeString<N> {
    buffer: String,
    _integer: PhantomData<N>,
}

#[allow(dead_code)]
impl<N> CompositeString<N>
where
    N: TryFrom<usize>,
    N::Error: Debug,
{
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(128),
            _integer: PhantomData,
        }
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn push_str(&mut self, s: &str) -> StartLen<N> {
        let start = self.len();
        self.buffer.push_str(s);
        StartLen::from_start_len(start, s.len())
    }

    pub fn push_display<D>(&mut self, s: &D) -> StartLen<N>
    where
        D: Display,
    {
        use std::fmt::Write;
        let start = self.len();
        let _ = write!(self.buffer, "{}", s);
        StartLen::from_start_len(start, self.len() - start)
    }

    #[allow(dead_code)]
    pub fn extend_with_str<I>(&mut self, separator: &str, iter: I) -> StartLen<N>
    where
        I: IntoIterator,
        I::Item: Borrow<str>,
    {
        let start = self.len();
        for s in iter {
            self.buffer.push_str(s.borrow());
            self.buffer.push_str(separator);
        }
        StartLen::from_start_len(start, self.len() - start)
    }

    pub fn extend_with_display<I>(&mut self, separator: &str, iter: I) -> StartLen<N>
    where
        I: IntoIterator,
        I::Item: Display,
    {
        use std::fmt::Write;
        let start = self.len();
        for elem in iter {
            let _ = write!(self.buffer, "{}", elem);
            self.buffer.push_str(separator);
        }
        StartLen::from_start_len(start, self.len() - start)
    }

    pub fn into_inner(self) -> String {
        self.buffer
    }
    pub fn as_inner(&self) -> &str {
        &self.buffer
    }
}

///////////////////////////////////////////////////////////////////////

pub type SmallCompositeVec<T> = CompositeVec<T, u16>;

/// A Vec-like type,
/// returning a `{start:16,len:u16}` range from methods that extend it.
pub struct CompositeVec<T, N> {
    list: Vec<T>,
    _integer: PhantomData<N>,
}

#[allow(dead_code)]
impl<T, N> CompositeVec<T, N>
where
    N: TryFrom<usize>,
    N::Error: Debug,
{
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            _integer: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            list: Vec::with_capacity(capacity),
            _integer: PhantomData,
        }
    }

    fn len(&self) -> usize {
        self.list.len()
    }

    pub fn push(&mut self, elem: T) -> u16 {
        let ind = self.len();
        self.list.push(elem);
        ind as u16
    }

    pub fn extend<I>(&mut self, iter: I) -> StartLen<N>
    where
        I: IntoIterator<Item = T>,
    {
        let start = self.len();
        self.list.extend(iter);
        StartLen::from_start_len(start, self.len() - start)
    }

    pub fn into_inner(self) -> Vec<T> {
        self.list
    }
    pub fn as_inner(&self) -> &[T] {
        &self.list
    }
}
