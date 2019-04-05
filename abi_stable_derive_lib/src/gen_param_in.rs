use syn::GenericParam;

use proc_macro2::TokenStream;
use quote::ToTokens;

/// Used to print (with ToTokens) the generc parameter differently depending on where
/// it is being used/declared.
pub(crate) struct GenParamIn<'a> {
    param: &'a GenericParam,
    in_: InWhat,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum InWhat {
    /// For the header of an impl block,as in the contents of `impl< ... >`.
    ImplHeader,
    /// For trait/struct/enum/union
    ItemDecl,
    /// For when using a generic as an argument for a trait/struct/enum/union.
    ItemUse,
}

impl<'a> GenParamIn<'a> {
    pub fn impl_header(param: &'a GenericParam) -> Self {
        Self {
            param,
            in_: InWhat::ImplHeader,
        }
    }

    pub fn item_decl(param: &'a GenericParam) -> Self {
        Self {
            param,
            in_: InWhat::ItemDecl,
        }
    }

    pub fn item_use(param: &'a GenericParam) -> Self {
        Self {
            param,
            in_: InWhat::ItemUse,
        }
    }
}

impl<'a> ToTokens for GenParamIn<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let with_bounds = self.in_ != InWhat::ItemUse;
        let with_default = self.in_ == InWhat::ItemDecl;

        match *self.param {
            GenericParam::Type(ref gen) => {
                gen.ident.to_tokens(tokens);
                if with_bounds {
                    gen.colon_token.to_tokens(tokens);
                    gen.bounds.to_tokens(tokens);
                }
                if with_default {
                    gen.eq_token.to_tokens(tokens);
                    gen.default.to_tokens(tokens);
                }
            }
            GenericParam::Lifetime(ref gen) => {
                gen.lifetime.to_tokens(tokens);
                if with_bounds {
                    gen.colon_token.to_tokens(tokens);
                    gen.bounds.to_tokens(tokens);
                }
            }
            GenericParam::Const(ref gen) => {
                if self.in_ != InWhat::ItemUse {
                    gen.const_token.to_tokens(tokens);
                }
                gen.ident.to_tokens(tokens);
                if with_bounds {
                    gen.colon_token.to_tokens(tokens);
                    gen.ty.to_tokens(tokens);
                }
                if with_default {
                    gen.eq_token.to_tokens(tokens);
                    gen.default.to_tokens(tokens);
                }
            }
        }
    }
}
