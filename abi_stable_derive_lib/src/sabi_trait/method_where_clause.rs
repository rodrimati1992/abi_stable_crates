use super::*;

use quote::ToTokens;

use syn::{WhereClause,WherePredicate};

#[derive(Debug,Default,Clone,PartialEq,Eq)]
pub(crate) struct MethodWhereClause<'a>{
    pub requires_self_sized:bool,
    _marker:PhantomData<&'a ()>,
}


impl<'a> MethodWhereClause<'a>{
    pub fn new(where_:&'a WhereClause,ctokens:&'a CommonTokens)->Self{
        let mut this=MethodWhereClause::default();
        
        for predicate in &where_.predicates {
            match predicate {
                WherePredicate::Type(ty_pred)=>{
                    if ty_pred.bounds.is_empty() {
                        panic!("The bounds are empty:'{}'",predicate.into_token_stream());
                    }
                    if ty_pred.bounded_ty==ctokens.self_ty && 
                        ty_pred.bounds[0]==ctokens.sized_bound 
                    {
                        this.requires_self_sized=true;
                    }else{
                        panic!("This bound is not supported:\n{}\n",predicate.into_token_stream());
                    }
                }
                WherePredicate::Lifetime{..}=>{
                    panic!(
                        "Lifetime constraints are not currently supported.\n`{}`\n",
                        predicate.into_token_stream()
                    )
                }
                WherePredicate::Eq{..}=>{
                    panic!(
                        "Type equality constraints are not currently supported.\n`{}`\n",
                        predicate.into_token_stream()
                    )
                }
            }
        }
        this
    }

    pub fn get_tokenizer(&self,ctokens:&'a CommonTokens)->MethodWhereClauseTokenizer<'_>{
        MethodWhereClauseTokenizer{
            where_clause:self,
            ctokens,
        }
    }
}



#[derive(Debug,Clone,PartialEq,Eq)]
pub(crate) struct MethodWhereClauseTokenizer<'a>{
    where_clause:&'a MethodWhereClause<'a>,
    ctokens:&'a CommonTokens,
}


impl<'a> ToTokens for MethodWhereClauseTokenizer<'a>{
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let where_clause=self.where_clause;
        let ctokens=self.ctokens;

        if where_clause.requires_self_sized {
            ctokens.self_sized.to_tokens(ts);
        }
    }
}