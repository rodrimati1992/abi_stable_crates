use super::*;

use as_derive_utils::spanned_err;

use quote::ToTokens;

use syn::{WhereClause, WherePredicate};

use crate::utils::{LinearResult, SynResultExt};

/// Parses and prints the syntactically valid where clauses in object safe traits.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct MethodWhereClause<'a> {
    pub requires_self_sized: bool,
    _marker: PhantomData<&'a ()>,
}

impl<'a> MethodWhereClause<'a> {
    pub fn new(where_: &'a WhereClause, ctokens: &'a CommonTokens) -> Result<Self, syn::Error> {
        let mut this = MethodWhereClause::default();
        let mut error = LinearResult::ok(());

        for predicate in &where_.predicates {
            match predicate {
                WherePredicate::Type(ty_pred) => {
                    if ty_pred.bounds.is_empty() {
                        error.push_err(spanned_err!(predicate, "The bounds are empty"));
                    }
                    if ty_pred.bounded_ty == ctokens.self_ty
                        && ty_pred.bounds[0] == ctokens.sized_bound
                    {
                        this.requires_self_sized = true;
                    } else {
                        error.push_err(spanned_err!(predicate, "This bound is not supported"));
                    }
                }
                WherePredicate::Lifetime { .. } => {
                    error.push_err(spanned_err!(
                        predicate,
                        "Lifetime constraints are not currently supported"
                    ));
                }
                WherePredicate::Eq { .. } => {
                    error.push_err(spanned_err!(
                        predicate,
                        "Type equality constraints are not currently supported",
                    ));
                }
            }
        }
        error.into_result().map(|_| this)
    }

    pub fn get_tokenizer(&self, ctokens: &'a CommonTokens) -> MethodWhereClauseTokenizer<'_> {
        MethodWhereClauseTokenizer {
            where_clause: self,
            ctokens,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MethodWhereClauseTokenizer<'a> {
    where_clause: &'a MethodWhereClause<'a>,
    ctokens: &'a CommonTokens,
}

impl<'a> ToTokens for MethodWhereClauseTokenizer<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let where_clause = self.where_clause;
        let ctokens = self.ctokens;

        if where_clause.requires_self_sized {
            ctokens.self_sized.to_tokens(ts);
        }
    }
}
