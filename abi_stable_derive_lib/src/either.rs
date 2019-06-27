use quote::ToTokens;

use proc_macro2::TokenStream;

#[allow(dead_code)]
#[derive(Debug,Copy,Clone)]
pub enum Either<L,R>{
    Left(L),
    Right(R),
}


impl<L,R> ToTokens for Either<L,R>
where
    L:ToTokens,
    R:ToTokens,
{
    fn to_tokens(&self, ts: &mut TokenStream){
        match self {
            Either::Left(v)=>v.to_tokens(ts),
            Either::Right(v)=>v.to_tokens(ts),
        }
    }
}