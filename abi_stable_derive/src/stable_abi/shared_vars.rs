use crate::{
    arenas::Arenas,
    composite_collections::SmallStartLen as StartLen,
    lifetimes::{LifetimeIndex, LifetimeIndexPair, LifetimeRange},
    literals_constructors::rslice_tokenizer,
    utils::{join_spans, LinearResult, SynResultExt},
};

use super::{
    attribute_parsing::LayoutConstructor, tl_multi_tl::TypeLayoutIndex, CommonTokens, ConstIdents,
};

use as_derive_utils::{syn_err, to_token_fn::ToTokenFnMut};

use core_extensions::SelfOps;

use proc_macro2::Span;

use quote::{quote, ToTokens};

use std::{collections::HashMap, fmt::Display};

pub(crate) struct SharedVars<'a> {
    const_idents: &'a ConstIdents,
    arenas: &'a Arenas,
    ctokens: &'a CommonTokens<'a>,
    strings: String,
    lifetime_indices: Vec<LifetimeIndex>,
    type_layouts_map: HashMap<(LayoutConstructor, &'a syn::Type), u16>,
    type_layouts: Vec<(LayoutConstructor, &'a syn::Type)>,
    constants: Vec<&'a syn::Expr>,
    overflowed_strings: LinearResult<()>,
    overflowed_lifetime_indices: LinearResult<()>,
    overflowed_type_layouts: LinearResult<()>,
    overflowed_constants: LinearResult<()>,
    extra_errs: LinearResult<()>,
}

impl<'a> SharedVars<'a> {
    pub(crate) fn new(
        arenas: &'a Arenas,
        const_idents: &'a ConstIdents,
        ctokens: &'a CommonTokens,
    ) -> Self {
        Self {
            const_idents,
            arenas,
            ctokens,
            strings: String::new(),
            lifetime_indices: Vec::new(),
            type_layouts: Vec::new(),
            type_layouts_map: HashMap::new(),
            constants: Vec::new(),
            overflowed_strings: LinearResult::ok(()),
            overflowed_lifetime_indices: LinearResult::ok(()),
            overflowed_type_layouts: LinearResult::ok(()),
            overflowed_constants: LinearResult::ok(()),
            extra_errs: LinearResult::ok(()),
        }
    }

    pub(crate) fn strings(&self) -> &str {
        &self.strings
    }

    pub(crate) fn arenas(&self) -> &'a Arenas {
        self.arenas
    }

    pub(crate) fn ctokens(&self) -> &'a CommonTokens<'a> {
        self.ctokens
    }

    pub(crate) fn extract_errs(&mut self) -> Result<(), syn::Error> {
        let mut errors = Ok::<(), syn::Error>(());
        self.overflowed_strings.take().combine_into_err(&mut errors);
        self.overflowed_lifetime_indices
            .take()
            .combine_into_err(&mut errors);
        self.overflowed_type_layouts
            .take()
            .combine_into_err(&mut errors);
        self.overflowed_constants
            .take()
            .combine_into_err(&mut errors);
        self.extra_errs.take().combine_into_err(&mut errors);
        errors
    }

    fn push_str_inner<F>(&mut self, f: F) -> StartLen
    where
        F: FnOnce(&mut Self) -> Option<Span>,
    {
        let start = self.strings.len();
        let span = f(self);
        let len = self.strings.len() - start;

        if start >= (1 << 16) || len >= (1 << 16) {
            self.string_overflow_err(span);
            StartLen::DUMMY
        } else {
            StartLen::from_start_len(start, len)
        }
    }

    pub(crate) fn combine_err(&mut self, r: Result<(), syn::Error>) {
        self.extra_errs.combine_err(r);
    }

    pub(crate) fn push_ident(&mut self, ident: &syn::Ident) -> StartLen {
        self.push_str_inner(|this| {
            use std::fmt::Write;
            let _ = write!(this.strings, "{}", ident);
            Some(ident.span())
        })
    }

    pub(crate) fn push_str(&mut self, s: &str, span: Option<Span>) -> StartLen {
        self.push_str_inner(|this| {
            this.strings.push_str(s);
            span
        })
    }

    pub fn extend_with_idents<I>(&mut self, separator: &str, iter: I) -> StartLen
    where
        I: IntoIterator<Item = &'a syn::Ident>,
    {
        self.push_str_inner(|this| {
            use std::fmt::Write;
            let mut last_span = None;
            for ident in iter {
                last_span = Some(ident.span());
                let _ = write!(this.strings, "{}", ident);
                this.push_str(separator, last_span);
            }
            last_span
        })
    }

    pub fn extend_with_display<I, T>(&mut self, separator: &str, iter: I) -> StartLen
    where
        I: IntoIterator<Item = (T, Span)>,
        T: Display,
    {
        self.push_str_inner(|this| {
            use std::fmt::Write;
            let mut last_span = None;
            for (elem, span) in iter {
                last_span = Some(span);
                let _ = write!(this.strings, "{}", elem);
                this.push_str(separator, Some(span));
            }
            last_span
        })
    }

    #[inline(never)]
    #[cold]
    pub(crate) fn string_overflow_err(&mut self, span: Option<Span>) {
        if self.overflowed_strings.is_ok() {
            self.overflowed_strings.push_err(syn_err!(
                span.unwrap_or_else(Span::call_site),
                "Cannot have more than 64 kylobytes of identifiers in the type combined.
                 This amount is approximate since it stores separators after some identifiers.\
                ",
            ));
        }
    }

    pub(crate) fn extend_with_lifetime_indices<I>(&mut self, iter: I) -> LifetimeRange
    where
        I: IntoIterator<Item = LifetimeIndex>,
    {
        let start = self.lifetime_indices.len();
        self.lifetime_indices.extend(iter);
        let len = self.lifetime_indices.len() - start;

        if len <= 5 {
            let mut drainer = self.lifetime_indices.drain(start..).fuse();
            LifetimeRange::from_array([
                drainer.next().unwrap_or(LifetimeIndex::NONE),
                drainer.next().unwrap_or(LifetimeIndex::NONE),
                drainer.next().unwrap_or(LifetimeIndex::NONE),
                drainer.next().unwrap_or(LifetimeIndex::NONE),
                drainer.next().unwrap_or(LifetimeIndex::NONE),
            ])
        } else {
            if (len & 1) == 1 {
                self.lifetime_indices.push(LifetimeIndex::NONE);
            }

            let half_len = self.lifetime_indices.len() / 2;
            if half_len > LifetimeRange::MAX_START {
                self.lifetime_overflow_start_err();
                LifetimeRange::DUMMY
            } else if half_len > LifetimeRange::MAX_LEN {
                self.lifetime_overflow_len_err();
                LifetimeRange::DUMMY
            } else {
                LifetimeRange::from_range(start / 2..half_len)
            }
        }
    }

    #[inline(never)]
    #[cold]
    pub(crate) fn lifetime_overflow_start_err(&mut self) {
        if self.overflowed_lifetime_indices.is_ok() {
            self.overflowed_lifetime_indices.push_err(syn_err!(
                Span::call_site(),
                "Cannot have more than {} lifetimes arguments within a type definition.\n\
                 The amount is approximate,\n
                 since this stores 5 or fewer lifetimes in fields/function pointers inline,
                 and those don't contribute to the lifetime arguments limit.",
                LifetimeRange::MAX_START * 2,
            ));
        }
    }

    #[inline(never)]
    #[cold]
    pub(crate) fn lifetime_overflow_len_err(&mut self) {
        if self.overflowed_lifetime_indices.is_ok() {
            self.overflowed_lifetime_indices.push_err(syn_err!(
                Span::call_site(),
                "Cannot have more than {} lifetimes arguments within \
                 a field or function pointer.\n",
                LifetimeRange::MAX_LEN * 2,
            ));
        }
    }

    pub(crate) fn push_type(
        &mut self,
        layout_ctor: LayoutConstructor,
        type_: &'a syn::Type,
    ) -> TypeLayoutIndex {
        let type_layouts = &mut self.type_layouts;

        let key = (layout_ctor, type_);

        let len = *self.type_layouts_map.entry(key).or_insert_with(move || {
            let len = type_layouts.len();
            type_layouts.push(key);
            len as u16
        });
        if len > TypeLayoutIndex::MAX_VAL_U16 {
            self.construct_type_overflow_err();
            TypeLayoutIndex::DUMMY
        } else {
            TypeLayoutIndex::from_u10(len)
        }
    }

    pub(crate) fn extend_type<I>(&mut self, layout_ctor: LayoutConstructor, types: I) -> StartLen
    where
        I: IntoIterator<Item = &'a syn::Type>,
    {
        let start = self.type_layouts.len();
        for ty in types {
            self.type_layouts.push((layout_ctor, ty));
        }
        let end = self.type_layouts.len();

        if end > TypeLayoutIndex::MAX_VAL {
            self.construct_type_overflow_err();
            StartLen::DUMMY
        } else {
            StartLen::from_start_len(start, end - start)
        }
    }

    #[inline(never)]
    #[cold]
    fn construct_type_overflow_err(&mut self) {
        if self.overflowed_type_layouts.is_ok() {
            self.overflowed_type_layouts.push_err(syn_err!(
                join_spans(
                    self.type_layouts
                        .drain(TypeLayoutIndex::MAX_VAL..)
                        .map(|(_, ty)| ty)
                ),
                "Cannot have more than {} unique types(ignoring lifetime parameters) \
                 within a type definition.",
                TypeLayoutIndex::MAX_VAL,
            ));
        }
    }

    pub fn get_type(&self, index: usize) -> Option<&'a syn::Type> {
        self.type_layouts.get(index).map(|(_, ty)| *ty)
    }

    pub(crate) fn extend_with_constants<I>(&mut self, iter: I) -> StartLen
    where
        I: IntoIterator<Item = &'a syn::Expr>,
    {
        let start = self.constants.len();
        for expr in iter {
            self.constants.push(expr);
        }
        let end = self.constants.len();
        if end >= (1 << 8) {
            self.construct_const_overflow_err();
            StartLen::DUMMY
        } else {
            StartLen::from_start_len(start, end - start)
        }
    }

    #[inline(never)]
    #[cold]
    fn construct_const_overflow_err(&mut self) {
        if self.overflowed_constants.is_ok() {
            self.overflowed_constants.push_err(syn_err!(
                Span::call_site(),
                "Cannot have more than {} unique types(ignoring lifetime parameters) \
                 within a type definition.",
                TypeLayoutIndex::MAX_VAL,
            ));
        }
    }

    pub(crate) fn mono_shared_vars_tokenizer(&self) -> impl ToTokens + '_ {
        ToTokenFnMut::new(move |ts| {
            let lifetime_indices = self
                .lifetime_indices
                .chunks(2)
                .map(|chunk| {
                    let first = chunk[0];
                    let second = chunk.get(1).map_or(LifetimeIndex::NONE, |x| *x);
                    LifetimeIndexPair::new(first, second).to_u8()
                })
                .piped(rslice_tokenizer);

            let strings = &self.const_idents.strings;

            quote!(
                abi_stable::type_layout::MonoSharedVars::new(
                    #strings,
                    #lifetime_indices,
                )
            )
            .to_tokens(ts);
        })
    }

    pub(crate) fn shared_vars_tokenizer(
        &self,
        mono_type_layout: &'a syn::Ident,
    ) -> impl ToTokens + '_ {
        ToTokenFnMut::new(move |ts| {
            let ct = self.ctokens;
            let type_layouts = self
                .type_layouts
                .iter()
                .map(|&(layout_ctor, ty)| make_get_type_layout_tokenizer(ty, layout_ctor, ct));

            let constants = self.constants.iter();

            quote!(
                const __SABI_CONST_PARAMS: &'static [__ConstGeneric] = &[
                    #(__ConstGeneric::new(&#constants),)*
                ];

                const __SABI_SHARED_VARS: &'static __sabi_re::SharedVars =
                    &abi_stable::type_layout::SharedVars::new (
                        #mono_type_layout.shared_vars_static(),
                        abi_stable::_sabi_type_layouts!( #(#type_layouts,)* ),
                        __sabi_re::RSlice::from_slice(Self::__SABI_CONST_PARAMS),
                    );
            )
            .to_tokens(ts);
        })
    }
}

#[must_use]
fn make_get_type_layout_tokenizer<'a, T: 'a>(
    ty: T,
    field_transparency: LayoutConstructor,
    ct: &'a CommonTokens<'a>,
) -> impl ToTokens + 'a
where
    T: ToTokens,
{
    ToTokenFnMut::new(move |ts| {
        ty.to_tokens(ts);
        let opt = match field_transparency {
            LayoutConstructor::Regular => None,
            LayoutConstructor::Opaque => Some(&ct.cap_opaque_field),
            LayoutConstructor::SabiOpaque => Some(&ct.cap_sabi_opaque_field),
        };

        if let Some(assoc_const) = opt {
            ct.equal.to_tokens(ts);
            assoc_const.to_tokens(ts)
        }
    })
}
