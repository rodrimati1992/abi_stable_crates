//! Contains the `GenParamsIn` type,for printing generic parameters.

use syn::{
    token::{Colon, Comma, Const, Star},
    GenericParam, Generics,
};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::utils::NoTokens;

/// Used to print (with ToTokens) the generc parameter differently depending on where
/// it is being used/declared.
///
/// One can also add stuff to be printed after lifetime/type parameters.
#[derive(Debug, Copy, Clone)]
pub struct GenParamsIn<'a, AL = NoTokens> {
    pub generics: &'a Generics,
    /// Where the generic parameters are being printed.
    pub in_what: InWhat,
    /// Whether type parameters are all `?Sized`
    pub unsized_types: bool,
    /// Whether type bounds will be printed in type parameter declarations.
    pub with_bounds: bool,
    skip_lifetimes: bool,
    /// What will be printed after lifetime parameters
    after_lifetimes: Option<AL>,
    /// What will be printed after type parameters
    after_types: Option<AL>,
    skip_consts: bool,
    skip_unbounded: bool,
}

/// Where the generic parameters are being printed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InWhat {
    /// For the header of an impl block,as in the contents of `impl< ... >`.
    ImplHeader,
    /// For trait/struct/enum/union
    ItemDecl,
    /// For when using a generic as an argument for a trait/struct/enum/union.
    ItemUse,
    /// For defining the fields of a Dummy struct that is never instantiated,
    /// only using its associated items.
    DummyStruct,
}

impl<'a> GenParamsIn<'a> {
    #[allow(dead_code)]
    pub fn new(generics: &'a Generics, in_what: InWhat) -> Self {
        Self {
            generics,
            in_what,
            unsized_types: false,
            with_bounds: true,
            skip_lifetimes: false,
            after_lifetimes: None,
            after_types: None,
            skip_consts: false,
            skip_unbounded: false,
        }
    }
}

impl<'a, AL> GenParamsIn<'a, AL> {
    /// Constructs a GenParamsIn that prints `after lifetimes` after lifetime parameters.
    pub fn with_after_lifetimes(
        generics: &'a Generics,
        in_what: InWhat,
        after_lifetimes: AL,
    ) -> Self {
        Self {
            generics,
            in_what,
            unsized_types: false,
            with_bounds: true,
            skip_lifetimes: false,
            after_lifetimes: Some(after_lifetimes),
            after_types: None,
            skip_consts: false,
            skip_unbounded: false,
        }
    }
    /// Constructs a GenParamsIn that prints `after types` after type parameters.
    pub fn with_after_types(generics: &'a Generics, in_what: InWhat, after_types: AL) -> Self {
        Self {
            generics,
            in_what,
            unsized_types: false,
            with_bounds: true,
            skip_lifetimes: false,
            after_lifetimes: None,
            after_types: Some(after_types),
            skip_consts: false,
            skip_unbounded: false,
        }
    }
    /// Removes the bounds on type parameters on type parameter declarations.
    pub fn set_no_bounds(&mut self) {
        self.with_bounds = false;
    }
    /// Makes all type parameters `?Sized`.
    #[allow(dead_code)]
    pub fn set_unsized_types(&mut self) {
        self.unsized_types = true;
    }

    pub fn skip_lifetimes(&mut self) {
        self.skip_lifetimes = true;
    }

    pub fn skip_consts(&mut self) {
        self.skip_consts = true;
    }
    pub fn skip_unbounded(&mut self) {
        self.skip_unbounded = true;
        self.skip_consts();
        self.skip_lifetimes();
    }

    pub fn skips_unbounded(&self) -> bool {
        self.skip_unbounded
    }

    /// Queries whether bounds are printed.
    pub fn outputs_bounds(&self) -> bool {
        self.with_bounds && matches!(self.in_what, InWhat::ImplHeader | InWhat::ItemDecl)
    }

    /// Queries whether all types have a `?Sized` bound.
    pub fn are_types_unsized(&self) -> bool {
        self.unsized_types && matches!(self.in_what, InWhat::ItemDecl | InWhat::ImplHeader)
    }
}

impl<'a, AL> ToTokens for GenParamsIn<'a, AL>
where
    AL: ToTokens,
{
    fn to_tokens(&self, ts: &mut TokenStream) {
        let with_bounds = self.outputs_bounds();

        let with_default = self.in_what == InWhat::ItemDecl;

        let in_dummy_struct = self.in_what == InWhat::DummyStruct;

        let unsized_types = self.are_types_unsized();

        let mut iter = self.generics.params.iter().peekable();

        let comma = Comma::default();
        let brace = syn::token::Brace::default();

        if self.skip_lifetimes {
            while let Some(GenericParam::Lifetime { .. }) = iter.peek() {
                iter.next();
            }
        } else {
            while let Some(GenericParam::Lifetime(gen)) = iter.peek() {
                iter.next();

                if in_dummy_struct {
                    syn::token::And::default().to_tokens(ts);
                    gen.lifetime.to_tokens(ts);
                    syn::token::Paren::default().surround(ts, |_| ());
                } else {
                    gen.lifetime.to_tokens(ts);
                    if with_bounds {
                        gen.colon_token.to_tokens(ts);
                        gen.bounds.to_tokens(ts);
                    }
                }

                comma.to_tokens(ts);
            }
        }

        self.after_lifetimes.to_tokens(ts);

        while let Some(GenericParam::Type(gen)) = iter.peek() {
            iter.next();
            if gen.bounds.is_empty() && self.skip_unbounded {
                continue;
            }

            if in_dummy_struct {
                Star::default().to_tokens(ts);
                Const::default().to_tokens(ts);
            }
            gen.ident.to_tokens(ts);

            if (with_bounds && gen.colon_token.is_some()) || unsized_types {
                Colon::default().to_tokens(ts);
                if unsized_types {
                    quote!(?Sized+).to_tokens(ts);
                }
                if with_bounds {
                    gen.bounds.to_tokens(ts);
                }
            }

            if with_default {
                gen.eq_token.to_tokens(ts);
                gen.default.to_tokens(ts);
            }

            comma.to_tokens(ts);
        }

        self.after_types.to_tokens(ts);

        if !in_dummy_struct && !self.skip_consts {
            while let Some(GenericParam::Const(gen)) = iter.peek() {
                iter.next();

                if self.in_what != InWhat::ItemUse {
                    gen.const_token.to_tokens(ts);
                }
                if self.in_what == InWhat::ItemUse {
                    // Have to surround the const parameter when it's used,
                    // because otherwise it's interpreted as a type parameter.
                    // Remove this branch once outputting the identifier
                    // for the const parameter just works.
                    brace.surround(ts, |ts| {
                        gen.ident.to_tokens(ts);
                    });
                } else {
                    gen.ident.to_tokens(ts);
                }
                if self.in_what != InWhat::ItemUse {
                    Colon::default().to_tokens(ts);
                    gen.ty.to_tokens(ts);
                }
                if with_default {
                    gen.eq_token.to_tokens(ts);
                    gen.default.to_tokens(ts);
                }

                comma.to_tokens(ts);
            }
        }
    }
}
