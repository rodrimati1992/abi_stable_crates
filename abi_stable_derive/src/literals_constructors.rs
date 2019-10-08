//! This module contains helper types/functions to output 
//! the literals and constructors for types that don't implement ToTokens.

use crate::{
    to_token_fn::ToTokenFnMut,
};

use proc_macro2::TokenStream as TokenStream2;

use quote::{ToTokens,quote};

use syn::Token;


/// Constructs an RSlice constant.
pub fn rslice_tokenizer<'a,I,T>(iter:I)->impl ToTokens+'a 
where
    I:IntoIterator<Item=T>+'a,
    T:ToTokens,
{
    let mut iter=iter.into_iter();
    ToTokenFnMut::new(move|ts|{
        let comma=<Token![,]>::default();
        let mut list=TokenStream2::new();
        let mut len=0usize;
        for elem in &mut iter {
            elem.to_tokens(&mut list);
            comma.to_tokens(&mut list);
            len+=1;
        }
        let list=list.into_iter();

        if cfg!(feature="rust_1_39") {
            quote!( abi_stable::std_types::RSlice::from_slice(&[ #(#list)* ]) )
        }else{
            quote!(
                unsafe{
                    abi_stable::std_types::RSlice::from_raw_parts_with_lifetime( 
                        &[ #(#list)* ],
                        #len
                    )
                }
            )
        }.to_tokens(ts);

    })
}

/// Constructs an RStr constant.
pub fn rstr_tokenizer<S>(string:S)->impl ToTokens
where
    S:AsRef<str>
{
    ToTokenFnMut::new(move|ts|{
        let string=string.as_ref();
        let len=string.len();

        if cfg!(feature="rust_1_39") {
            quote!( abi_stable::std_types::RStr::from_str(#string) )
        }else{
            quote!(
                unsafe{
                    abi_stable::std_types::RStr::from_raw_parts( 
                        #string.as_ptr(),
                        #len
                    )
                }
            )
        }.to_tokens(ts);
    })
}