use super::*;

use crate::utils::{expr_from_ident, type_from_ident};

pub(super) struct GenericParams<'a> {
    generics: &'a syn::Generics,
    type_param_range: StartLen,
    const_param_range: StartLen,
    ctokens: &'a CommonTokens<'a>,
}

impl<'a> GenericParams<'a> {
    pub(super) fn new(
        ds: &'a DataStructure<'a>,
        shared_vars: &mut SharedVars<'a>,
        config: &'a StableAbiOptions<'a>,
        ctokens: &'a CommonTokens<'a>,
    ) -> Self {
        let generics = ds.generics;
        let arenas = shared_vars.arenas();
        let type_param_range: StartLen = {
            let shared_vars = std::cell::RefCell::new(&mut *shared_vars);
            let phantom_type_params = config.phantom_type_params.iter().map(|ty| {
                let ty: &'a syn::Type = ty;
                shared_vars
                    .borrow_mut()
                    .push_type(LayoutConstructor::Regular, ty)
                    .to_u10()
            });

            let type_param_bounds = config
                .type_param_bounds
                .iter()
                .filter(|(_, bounds)| **bounds == ASTypeParamBound::NoBound)
                .map(|(type_param, &bounds)| {
                    let type_ = {
                        let x = type_from_ident(type_param.clone());
                        arenas.alloc(x)
                    };

                    let layout_ctor = bounds.into_::<LayoutConstructor>();
                    shared_vars
                        .borrow_mut()
                        .push_type(layout_ctor, type_)
                        .to_u10()
                });

            let mut iter = type_param_bounds.chain(phantom_type_params);
            let first = iter.next().unwrap_or(0);
            let mut last = first;
            for elem in iter {
                assert!(
                    first <= elem,
                    "BUG:\
                    The type parameters must all be stored contiguously in the SharedVars.\n\
                    last={} elem={}\
                    ",
                    last,
                    elem,
                );
                last = elem;
            }
            StartLen {
                start: first,
                len: last - first,
            }
        };

        let const_param_range = {
            let const_params = generics
                .const_params()
                .map(|cp| arenas.alloc(expr_from_ident(cp.ident.clone())))
                .chain(config.phantom_const_params.iter().cloned());
            shared_vars.extend_with_constants(const_params)
        };

        Self {
            generics,
            type_param_range,
            const_param_range,
            ctokens,
        }
    }
}

impl<'a> ToTokens for GenericParams<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let lifetimes = &self
            .generics
            .lifetimes()
            .map(|x| &x.lifetime)
            .collect::<Vec<_>>();
        let type_param_range = self.type_param_range.tokenizer(self.ctokens.as_ref());
        let const_param_range = self.const_param_range.tokenizer(self.ctokens.as_ref());
        quote!(abi_stable::tl_genparams!(
            #(#lifetimes),*;
            #type_param_range;
            #const_param_range
        ))
        .to_tokens(ts);
    }
}
