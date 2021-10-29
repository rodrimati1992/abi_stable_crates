//! This module contains helper types/functions to output
//! the literals and constructors for types that don't implement ToTokens.

use as_derive_utils::to_token_fn::ToTokenFnMut;

use quote::{quote, ToTokens, TokenStreamExt};

/// Constructs an RSlice constant.
pub fn rslice_tokenizer<'a, I, T>(iter: I) -> impl ToTokens + 'a
where
    I: IntoIterator<Item = T> + 'a,
    T: ToTokens,
{
    let mut iter = iter.into_iter();
    ToTokenFnMut::new(move |ts| {
        let iter = &mut iter;
        ts.append_all(quote!(
            abi_stable::std_types::RSlice::from_slice(&[ #( #iter, )* ])
        ));
    })
}

/// Constructs an RStr constant.
pub fn rstr_tokenizer<S>(string: S) -> impl ToTokens
where
    S: AsRef<str>,
{
    ToTokenFnMut::new(move |ts| {
        let string = string.as_ref();

        ts.append_all(quote!( abi_stable::std_types::RStr::from_str(#string) ));
    })
}
