use super::{
    attribute_parsing::{MethodWithAttrs, SabiTraitAttrs},
    impl_interfacetype::{TraitStruct, WhichTrait, TRAIT_LIST},
    lifetime_unelider::BorrowKind,
    parse_utils::{parse_str_as_ident, parse_str_as_trait_bound, parse_str_as_type},
    replace_self_path::{self, ReplaceWith},
    *,
};

use crate::{
    set_span_visitor::SetSpanVisitor,
    utils::{dummy_ident, LinearResult, SynResultExt},
};

use as_derive_utils::{return_spanned_err, spanned_err, syn_err};

use std::{
    collections::{HashMap, HashSet},
    iter,
};

use core_extensions::{matches, IteratorExt};

use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::Unsafe,
    token::{Colon, Comma, Semi},
    visit_mut::VisitMut,
    Abi, Attribute, Block, FnArg, Ident, ItemTrait, Lifetime, LifetimeDef, TraitItem,
    TypeParamBound, WherePredicate,
};

use proc_macro2::Span;

#[derive(Debug, Clone)]
pub struct AssocTyWithIndex {
    pub index: usize,
    pub assoc_ty: syn::TraitItemType,
}

////////////////////////////////////////////////////////////////////////////////

/// Represents a trait for use in `#[sabi_trait]`.
#[derive(Debug, Clone)]
pub(crate) struct TraitDefinition<'a> {
    pub(crate) item: &'a ItemTrait,
    /// The name of the trait.
    pub(crate) name: &'a Ident,
    /// What type to use the backend for the trait object,DynTrait or RObject.
    pub(crate) which_object: WhichObject,
    /// The where predicates in the where clause of the trait,
    /// if it doesn't have one this is empty.
    pub(crate) where_preds: Punctuated<WherePredicate, Comma>,
    /// Attributes applied to the vtable.
    pub(crate) derive_attrs: &'a [Attribute],
    /// Attributes applied to the trait.
    pub(crate) other_attrs: &'a [Attribute],
    pub(crate) generics: &'a syn::Generics,
    /// The `Iterator::Item` type for this trait,
    /// None if it doesn't have Iterator as a supertrait.
    pub(crate) iterator_item: Option<&'a syn::Type>,
    #[allow(dead_code)]
    /// The path for the implemented serde::Deserialize trait
    /// (it may reference some trait lifetime parameter)
    pub(crate) deserialize_bound: Option<DeserializeBound>,
    /// The traits this has as supertraits.
    pub(crate) impld_traits: Vec<TraitImplness<'a>>,
    /// The traits this doesn't have as supertraits.
    pub(crate) unimpld_traits: Vec<&'a Ident>,
    /// A struct describing the traits this does and doesn't have as supertraits
    /// (true means implemented,false means unimplemented)
    pub(crate) trait_flags: TraitStruct<bool>,
    /// The region of code of the identifiers for the supertraits,
    /// with `Span::call_site()` for the ones that aren't supertraits.
    pub(crate) trait_spans: TraitStruct<Span>,
    /// The lifetimes declared in the trait generic parameter list that are used in
    /// `&'lifetime self` `&'lifetime mut self` method receivers,
    /// or used directly as supertraits.
    pub(crate) lifetime_bounds: Punctuated<&'a Lifetime, Comma>,
    /// The visibility of the trait.
    pub(crate) vis: VisibilityKind<'a>,
    /// The visibility of the trait,inside a submodule.
    pub(crate) submod_vis: RelativeVis<'a>,
    // The keys use the proginal identifier for the associated type.
    pub(crate) assoc_tys: HashMap<&'a Ident, AssocTyWithIndex>,
    ///
    pub(crate) methods: Vec<TraitMethod<'a>>,
    /// Whether this has by mutable reference methods.
    pub(crate) has_mut_methods: bool,
    /// Whether this has by-value methods.
    pub(crate) has_val_methods: bool,
    /// Disables `ìmpl Trait for Trait_TO`
    pub(crate) disable_trait_impl: bool,
    /// Whether this has `'static` as a supertrait syntactically.
    pub(crate) is_static: IsStaticTrait,
    /// A TokenStream with the equivalent of `<Pointer::PtrTarget as Trait>::`
    pub(crate) ts_fq_self: &'a TokenStream2,
    pub(crate) ctokens: &'a CommonTokens,
}

////////////////////////////////////////////////////////////////////////////////

impl<'a> TraitDefinition<'a> {
    pub(super) fn new(
        trait_: &'a ItemTrait,
        SabiTraitAttrs {
            attrs,
            methods_with_attrs,
            which_object,
            disable_trait_impl,
            disable_inherent_default,
            ..
        }: SabiTraitAttrs<'a>,
        arenas: &'a Arenas,
        ctokens: &'a CommonTokens,
    ) -> Result<Self, syn::Error> {
        let vis = VisibilityKind::new(&trait_.vis);
        let submod_vis = vis.submodule_level(1);
        let mut assoc_tys = HashMap::default();
        let mut methods = Vec::<TraitMethod<'a>>::new();

        let mut errors = LinearResult::ok(());

        methods_with_attrs
            .into_iter()
            .zip(disable_inherent_default)
            .filter_map(|(func, disable_inh_def)| {
                match TraitMethod::new(func, disable_inh_def, ctokens, arenas) {
                    Ok(x) => x,
                    Err(e) => {
                        errors.push_err(e);
                        None
                    }
                }
            })
            .extending(&mut methods);

        /////////////////////////////////////////////////////
        ////         Processing the supertrait bounds

        let mut is_static = IsStaticTrait::No;

        let lifetime_params: HashSet<&'a Lifetime> = trait_
            .generics
            .lifetimes()
            .map(|l| &l.lifetime)
            .chain(iter::once(&ctokens.static_lifetime))
            .collect();

        let GetSupertraits {
            impld_traits,
            unimpld_traits,
            mut lifetime_bounds,
            iterator_item,
            deserialize_bound,
            trait_flags,
            trait_spans,
            errors: supertrait_errors,
        } = get_supertraits(
            &trait_.supertraits,
            &lifetime_params,
            which_object,
            arenas,
            ctokens,
        );
        errors.combine_err(supertrait_errors.into());

        // Adding the lifetime parameters in `&'a self` and `&'a mut self`
        // that were declared in the trait generic parameter list.
        // This is done because those lifetime bounds are enforced as soon as
        // the vtable is created,instead of when the methods are called
        // (it's enforced in method calls in regular trait objects).
        for method in &methods {
            if let SelfParam::ByRef {
                lifetime: Some(lt), ..
            } = method.self_param
            {
                if lifetime_params.contains(lt) {
                    lifetime_bounds.push(lt);
                }
            }
        }

        for lt in &lifetime_bounds {
            if lt.ident == "static" {
                is_static = IsStaticTrait::Yes;
            }
        }
        /////////////////////////////////////////////////////

        let mut assoc_ty_index = 0;
        for item in &trait_.items {
            match item {
                TraitItem::Method { .. } => {}
                TraitItem::Type(assoc_ty) => {
                    let with_index = AssocTyWithIndex {
                        index: assoc_ty_index,
                        assoc_ty: assoc_ty.clone(),
                    };
                    assoc_tys.insert(&assoc_ty.ident, with_index);

                    assoc_ty_index += 1;
                }
                item => errors.push_err(spanned_err!(
                    item,
                    "Associated item not compatible with #[sabi_trait]",
                )),
            }
        }

        let has_mut_methods = methods.iter().any(|m| {
            matches!(
                &m.self_param,
                SelfParam::ByRef {
                    is_mutable: true,
                    ..
                }
            )
        });

        let has_val_methods = methods
            .iter()
            .any(|m| matches!(&m.self_param, SelfParam::ByVal));

        let ts_fq_self = {
            let (_, generics_params, _) = trait_.generics.split_for_impl();
            quote!( <_OrigPtr::PtrTarget as __Trait #generics_params >:: )
        };

        errors.into_result()?;

        Ok(TraitDefinition {
            item: trait_,
            name: &trait_.ident,
            which_object,
            where_preds: trait_
                .generics
                .where_clause
                .as_ref()
                .map(|wc| wc.predicates.clone())
                .unwrap_or_default(),
            derive_attrs: arenas.alloc(attrs.derive_attrs),
            other_attrs: arenas.alloc(attrs.other_attrs),
            generics: &trait_.generics,
            lifetime_bounds,
            iterator_item,
            deserialize_bound,
            impld_traits,
            unimpld_traits,
            trait_flags,
            trait_spans,
            vis,
            submod_vis,
            assoc_tys,
            methods,
            has_mut_methods,
            has_val_methods,
            disable_trait_impl,
            ts_fq_self: arenas.alloc(ts_fq_self),
            is_static,
            ctokens,
        })
    }

    /// Returns a clone of `self`,
    /// where usages of associated types are replaced for use in `which_item`.
    pub fn replace_self(&self, which_item: WhichItem) -> Result<Self, syn::Error> {
        let mut this = self.clone();

        let ctokens = self.ctokens;

        let mut errors = LinearResult::ok(());

        let replace_with = match which_item {
            WhichItem::Trait | WhichItem::TraitImpl => {
                return Ok(this);
            }
            WhichItem::TraitObjectImpl => ReplaceWith::Remove,
            WhichItem::VtableDecl => ReplaceWith::Remove,
            WhichItem::VtableImpl => ReplaceWith::Ident(ctokens.u_capself.clone()),
        };

        let is_assoc_type = |ident: &Ident| {
            if self.assoc_tys.contains_key(ident) {
                Some(ReplaceWith::Keep)
            } else {
                None
            }
        };

        for where_pred in &mut this.where_preds {
            replace_self_path::replace_self_path(where_pred, replace_with.clone(), is_assoc_type)
                .combine_into_err(&mut errors);
        }

        for assoc_ty in this.assoc_tys.values_mut() {
            replace_self_path::replace_self_path(
                &mut assoc_ty.assoc_ty,
                replace_with.clone(),
                is_assoc_type,
            )
            .combine_into_err(&mut errors);
        }

        for method in &mut this.methods {
            method
                .replace_self(replace_with.clone(), is_assoc_type)
                .combine_into_err(&mut errors);
        }
        errors.into_result().map(|_| this)
    }

    /// Returns a tokenizer for the generic parameters in this trait.
    ///
    /// # Parameters
    ///
    /// - `in_what`:
    ///     Determines where the generic parameters are printed.
    ///     Eg:impl headers,trait declaration,trait usage.
    ///
    /// - `with_assoc_tys`:
    ///     Whether associated types are printed,and how.
    ///
    /// - `after_lifetimes`:
    ///     What will be printed after lifetime parameters.
    ///
    pub fn generics_tokenizer(
        &self,
        in_what: InWhat,
        with_assoc_tys: WithAssocTys,
        after_lifetimes: &'a TokenStream2,
    ) -> GenericsTokenizer<'_> {
        let ctokens = self.ctokens;
        GenericsTokenizer {
            gen_params_in: GenParamsIn::with_after_lifetimes(
                self.generics,
                in_what,
                after_lifetimes,
            ),
            assoc_tys: match with_assoc_tys {
                WithAssocTys::Yes(WhichSelf::Regular) => {
                    Some((&self.assoc_tys, &ctokens.ts_self_colon2))
                }
                WithAssocTys::Yes(WhichSelf::Underscore) => {
                    Some((&self.assoc_tys, &ctokens.ts_uself_colon2))
                }
                WithAssocTys::Yes(WhichSelf::FullyQualified) => {
                    Some((&self.assoc_tys, self.ts_fq_self))
                }
                WithAssocTys::Yes(WhichSelf::NoSelf) => Some((&self.assoc_tys, &ctokens.empty_ts)),
                WithAssocTys::No => None,
            },
        }
    }

    /// Returns the where predicates for the erased pointer type of the ffi-safe trait object.
    ///
    /// Example erased pointer types:`RBox<()>`,`RArc<()>`,`&()`,`&mut ()`
    ///
    pub fn erased_ptr_preds(&self) -> &'a TokenStream2 {
        let ctokens = self.ctokens;
        match (self.has_mut_methods, self.has_val_methods) {
            (false, false) => &ctokens.ptr_ref_bound,
            (false, true) => &ctokens.ptr_ref_val_bound,
            (true, false) => &ctokens.ptr_mut_bound,
            (true, true) => &ctokens.ptr_mut_val_bound,
        }
    }

    /// Returns the where predicates of the inherent implementation of
    /// the ffi-safe trait object.
    pub fn trait_impl_where_preds(&self) -> Result<Punctuated<WherePredicate, Comma>, syn::Error> {
        let mut where_preds = self.where_preds.clone();
        let mut errors = LinearResult::ok(());
        for where_pred in &mut where_preds {
            replace_self_path::replace_self_path(where_pred, ReplaceWith::Remove, |ident| {
                self.assoc_tys.get(ident).map(|_| ReplaceWith::Remove)
            })
            .combine_into_err(&mut errors);
        }
        errors.into_result().map(|_| where_preds)
    }

    /// Returns a tokenizer that outputs the method definitions inside the `which_item` item.
    pub fn methods_tokenizer(&self, which_item: WhichItem) -> MethodsTokenizer<'_> {
        MethodsTokenizer {
            trait_def: self,
            which_item,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents a trait method for use in `#[sabi_trait]`.
#[derive(Debug, Clone)]
pub(crate) struct TraitMethod<'a> {
    pub(crate) disable_inherent_default: bool,
    pub(crate) unsafety: Option<&'a Unsafe>,
    pub(crate) abi: Option<&'a Abi>,
    /// Attributes applied to the method in the vtable.
    pub(crate) derive_attrs: &'a [Attribute],
    /// Attributes applied to the method in the trait definition.
    pub(crate) other_attrs: &'a [Attribute],
    /// The name of the method.
    pub(crate) name: &'a Ident,
    pub(crate) self_param: SelfParam<'a>,
    /// The lifetime parameters of this method.
    pub(crate) lifetimes: Vec<&'a LifetimeDef>,
    pub(crate) params: Vec<MethodParam<'a>>,
    /// The return type of this method,if None this returns `()`.
    pub(crate) output: Option<syn::Type>,

    /// Whether the return type borrows from self
    pub(crate) return_borrow_kind: Option<BorrowKind>,

    pub(crate) where_clause: MethodWhereClause<'a>,
    /// The default implementation of the method.
    pub(crate) default: Option<DefaultMethod<'a>>,
    /// The semicolon token for the method
    /// (when the method did not have a default implementation).
    pub(crate) semicolon: Option<&'a Semi>,
}

#[derive(Debug, Clone)]
pub(crate) struct DefaultMethod<'a> {
    pub(crate) block: &'a Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MethodParam<'a> {
    /// The name of the method parameter,
    /// which is `param_<number_of_parameter>` if the parameter is not just an identifier
    /// (ie:`(left,right)`,`Rect3D{x,y,z}`)
    pub(crate) name: &'a Ident,
    /// The parameter type.
    pub(crate) ty: syn::Type,
    /// The pattern for the parameter
    pub(crate) pattern: &'a syn::Pat,
}

impl<'a> TraitMethod<'a> {
    pub fn new(
        mwa: MethodWithAttrs<'a>,
        disable_inherent_default: bool,
        ctokens: &'a CommonTokens,
        arena: &'a Arenas,
    ) -> Result<Option<Self>, syn::Error> {
        let method_signature = &mwa.item.sig;
        let decl = method_signature;
        let name = &method_signature.ident;

        let mut errors = LinearResult::ok(());

        let push_error_msg = |errors: &mut Result<(), syn::Error>| {
            errors.push_err(spanned_err!(
                method_signature.ident,
                "Cannot define #[sabi_trait]traits containing methods \
                 without a `self`/`&self`/`&mut self` receiver (static methods)."
            ));
        };
        if decl.inputs.is_empty() {
            push_error_msg(&mut errors);
        }

        let mut input_iter = decl.inputs.iter();

        let mut self_param = match input_iter.next() {
            Some(FnArg::Receiver(receiver)) => match &receiver.reference {
                Some((_, lifetime)) => SelfParam::ByRef {
                    lifetime: lifetime.as_ref(),
                    is_mutable: receiver.mutability.is_some(),
                },
                None => SelfParam::ByVal,
            },
            Some(FnArg::Typed { .. }) => {
                push_error_msg(&mut errors);
                SelfParam::ByVal
            }
            None => {
                push_error_msg(&mut errors);
                return errors.into_result().map(|_| unreachable!());
            }
        };

        let mut lifetimes: Vec<&'a syn::LifetimeDef> = decl.generics.lifetimes().collect();

        let mut return_borrow_kind = None::<BorrowKind>;

        let output = match &decl.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => {
                let mut ty: syn::Type = (**ty).clone();
                if let SelfParam::ByRef { lifetime, .. } = &mut self_param {
                    let visit_data = LifetimeUnelider::new(lifetime).visit_type(&mut ty);

                    return_borrow_kind = visit_data.found_borrow_kind;

                    if let Some(lt) = visit_data.additional_lifetime_def {
                        lifetimes.push(lt);
                    }
                }
                Some(ty)
            }
        };

        let default = mwa
            .item
            .default
            .as_ref()
            .map(|block| DefaultMethod { block });

        let where_clause = decl
            .generics
            .where_clause
            .as_ref()
            .and_then(|wc| match MethodWhereClause::new(wc, ctokens) {
                Ok(x) => Some(x),
                Err(e) => {
                    errors.push_err(e);
                    None
                }
            })
            .unwrap_or_default();

        let mut params = Vec::<MethodParam<'a>>::with_capacity(input_iter.len());

        for (param_i, param) in input_iter.enumerate() {
            let (pattern, ty) = match param {
                FnArg::Receiver { .. } => unreachable!(),
                FnArg::Typed(typed) => (&*typed.pat, &*typed.ty),
            };

            let name = format!("param_{}", param_i);
            let mut name = syn::parse_str::<Ident>(&name).unwrap_or_else(|e| {
                errors.push_err(e);
                dummy_ident()
            });
            name.set_span(param.span());

            params.push(MethodParam {
                name: arena.alloc(name),
                ty: ty.clone(),
                pattern,
            });
        }

        errors.into_result()?;

        Ok(Some(Self {
            disable_inherent_default,
            unsafety: method_signature.unsafety.as_ref(),
            abi: method_signature.abi.as_ref(),
            derive_attrs: arena.alloc(mwa.attrs.derive_attrs),
            other_attrs: arena.alloc(mwa.attrs.other_attrs),
            name,
            lifetimes,
            self_param,
            params,
            output,
            return_borrow_kind,
            where_clause,
            default,
            semicolon: mwa.item.semi_token.as_ref(),
        }))
    }

    /// Returns a clone of `self`,
    /// where usages of associated types are replaced for use in `which_item`.
    ///
    /// Whether `Self::AssocTy` is an associated type is determined using `is_assoc_type`,
    /// which returns `Some()` with what to do with the associated type.
    pub fn replace_self<F>(
        &mut self,
        replace_with: ReplaceWith,
        mut is_assoc_type: F,
    ) -> Result<(), syn::Error>
    where
        F: FnMut(&Ident) -> Option<ReplaceWith>,
    {
        let mut errors = LinearResult::ok(());

        for param in self
            .params
            .iter_mut()
            .map(|x| &mut x.ty)
            .chain(self.output.as_mut())
        {
            replace_self_path::replace_self_path(param, replace_with.clone(), &mut is_assoc_type)
                .combine_into_err(&mut errors);
        }
        errors.into()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Used to print the generic parameters of a trait,
/// potentially including its associated types.
#[derive(Debug, Copy, Clone)]
pub struct GenericsTokenizer<'a> {
    gen_params_in: GenParamsIn<'a, &'a TokenStream2>,
    assoc_tys: Option<(&'a HashMap<&'a Ident, AssocTyWithIndex>, &'a TokenStream2)>,
}

impl<'a> GenericsTokenizer<'a> {
    /// Changes type parameters to have a `?Sized` bound.
    #[allow(dead_code)]
    pub fn set_unsized_types(&mut self) {
        self.gen_params_in.set_unsized_types();
    }
    /// Removes bounds on type parameters.
    pub fn set_no_bounds(&mut self) {
        self.gen_params_in.set_no_bounds();
    }
    pub fn skip_lifetimes(&mut self) {
        self.gen_params_in.skip_lifetimes();
    }
    #[allow(dead_code)]
    pub fn skip_consts(&mut self) {
        self.gen_params_in.skip_consts();
    }
}

impl<'a> ToTokens for GenericsTokenizer<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let with_bounds = self.gen_params_in.outputs_bounds();
        let with_default = self.gen_params_in.in_what == InWhat::ItemDecl;

        let unsized_types = self.gen_params_in.are_types_unsized();

        let in_dummy_struct = self.gen_params_in.in_what == InWhat::DummyStruct;

        let skips_unbounded = self.gen_params_in.skips_unbounded();

        self.gen_params_in.to_tokens(ts);
        if let Some((assoc_tys, self_tokens)) = self.assoc_tys {
            for with_index in assoc_tys.values() {
                let assoc_ty = &with_index.assoc_ty;

                if skips_unbounded && assoc_ty.bounds.is_empty() {
                    continue;
                }

                self_tokens.to_tokens(ts);

                if in_dummy_struct {
                    use syn::token::{Const, Star};
                    Star::default().to_tokens(ts);
                    Const::default().to_tokens(ts);
                }

                assoc_ty.ident.to_tokens(ts);

                let colon_token = assoc_ty.colon_token.filter(|_| with_bounds);

                if unsized_types {
                    if colon_token.is_none() {
                        Colon::default().to_tokens(ts);
                    }
                    quote!(?Sized+).to_tokens(ts);
                }
                if let Some(colon_token) = colon_token {
                    colon_token.to_tokens(ts);
                    assoc_ty.bounds.to_tokens(ts);
                }

                match &assoc_ty.default {
                    Some((eq_token, default_ty)) if with_default => {
                        eq_token.to_tokens(ts);
                        default_ty.to_tokens(ts);
                    }
                    _ => {}
                }

                Comma::default().to_tokens(ts);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents a `Deserialize<'de>` supertrait bound.
#[derive(Debug, Clone)]
pub(crate) struct DeserializeBound;

/// Used to returns the information about supertraits,to construct TraitDefinition.
struct GetSupertraits<'a> {
    impld_traits: Vec<TraitImplness<'a>>,
    unimpld_traits: Vec<&'a Ident>,
    lifetime_bounds: Punctuated<&'a Lifetime, Comma>,
    iterator_item: Option<&'a syn::Type>,
    deserialize_bound: Option<DeserializeBound>,
    trait_flags: TraitStruct<bool>,
    trait_spans: TraitStruct<Span>,
    errors: LinearResult<()>,
}

/// Contains information about a supertrait,including whether it's implemented.
#[derive(Debug, Clone)]
pub(crate) struct TraitImplness<'a> {
    pub(crate) ident: Ident,
    pub(crate) bound: syn::TraitBound,
    pub(crate) is_implemented: bool,
    pub(crate) _marker: PhantomData<&'a ()>,
}

/// Processes the supertrait bounds of a trait definition.
fn get_supertraits<'a, I>(
    supertraits: I,
    lifetime_params: &HashSet<&'a Lifetime>,
    which_object: WhichObject,
    arenas: &'a Arenas,
    _ctokens: &'a CommonTokens,
) -> GetSupertraits<'a>
where
    I: IntoIterator<Item = &'a TypeParamBound>,
{
    let trait_map = TRAIT_LIST
        .iter()
        .map(|t| (parse_str_as_ident(t.name), t.which_trait))
        .collect::<HashMap<Ident, WhichTrait>>();

    // A struct indexable by `WhichTrait`,
    // with information about all possible supertraits.
    let mut trait_struct = TraitStruct::TRAITS.map(|_, t| TraitImplness {
        ident: parse_str_as_ident(t.name),
        bound: parse_str_as_trait_bound(t.full_path).expect("BUG"),
        is_implemented: false,
        _marker: PhantomData,
    });

    let mut lifetime_bounds = Punctuated::<&'a Lifetime, Comma>::new();
    let mut iterator_item = None;
    let mut errors = LinearResult::ok(());
    let deserialize_bound = None;

    for supertrait_bound in supertraits {
        match supertrait_bound {
            TypeParamBound::Trait(trait_bound) => {
                let last_path_component = match trait_bound.path.segments.last() {
                    Some(x) => x,
                    None => continue,
                };
                let trait_ident = &last_path_component.ident;

                match trait_map.get(trait_ident) {
                    Some(&which_trait) => {
                        let usable_by = which_trait.usable_by();
                        match which_object {
                            WhichObject::DynTrait if !usable_by.dyn_trait() => {
                                errors.push_err(spanned_err!(
                                    trait_bound.path,
                                    "cannot use this trait with DynTrait",
                                ));
                            }
                            WhichObject::RObject if !usable_by.robject() => {
                                errors.push_err(spanned_err!(
                                    trait_bound.path,
                                    "cannot use this trait with RObject.
                                     To make that trait usable you must use the \
                                     #[sabi(use_dyntrait)] attribute,\
                                     which changes the trait object implementation \
                                     from using RObject to using DynTrait.\n\
                                    ",
                                ));
                            }
                            WhichObject::DynTrait | WhichObject::RObject => {}
                        }

                        fn set_impld(wtrait: &mut TraitImplness<'_>, span: Span) {
                            wtrait.is_implemented = true;
                            wtrait.ident.set_span(span);
                            SetSpanVisitor::new(span).visit_trait_bound_mut(&mut wtrait.bound);
                        }

                        let span = trait_bound.span();

                        set_impld(&mut trait_struct[which_trait], span);

                        match which_trait {
                            WhichTrait::Iterator | WhichTrait::DoubleEndedIterator => {
                                set_impld(&mut trait_struct.iterator, span);

                                let iter_item = extract_iterator_item(last_path_component, arenas);
                                iterator_item = iterator_item.or(iter_item);
                            }
                            WhichTrait::Deserialize => {
                                errors.push_err(spanned_err!(
                                    trait_bound.path,
                                    "Deserialize is not currently supported."
                                ));
                            }
                            WhichTrait::Serialize => {
                                errors.push_err(spanned_err!(
                                    trait_bound.path,
                                    "Serialize is not currently supported."
                                ));
                            }
                            WhichTrait::Eq | WhichTrait::PartialOrd => {
                                set_impld(&mut trait_struct.partial_eq, span);
                            }
                            WhichTrait::Ord => {
                                set_impld(&mut trait_struct.partial_eq, span);
                                set_impld(&mut trait_struct.eq, span);
                                set_impld(&mut trait_struct.partial_ord, span);
                            }
                            WhichTrait::IoBufRead => {
                                set_impld(&mut trait_struct.io_read, span);
                            }
                            WhichTrait::Error => {
                                set_impld(&mut trait_struct.display, span);
                                set_impld(&mut trait_struct.debug, span);
                            }
                            _ => {}
                        }
                    }
                    None => {
                        let list = trait_map
                            .keys()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>();

                        errors.push_err(spanned_err!(
                            supertrait_bound,
                            "Unexpected supertrait bound.\nExpected one of:\n{}",
                            list.join("/"),
                        ));
                        break;
                    }
                }
            }
            TypeParamBound::Lifetime(lt) => {
                if lifetime_params.contains(lt) {
                    lifetime_bounds.push(lt);
                } else {
                    errors.push_err(spanned_err!(
                        lt,
                        "Lifetimes is not from the trait or `'static`.",
                    ));
                    break;
                }
            }
        };
    }

    let iter_trait = &mut trait_struct.iterator;
    let de_iter_trait = &mut trait_struct.double_ended_iterator;
    if iter_trait.is_implemented || de_iter_trait.is_implemented {
        let iter_item: syn::Type = iterator_item.cloned().unwrap_or_else(|| {
            let span = if de_iter_trait.is_implemented {
                de_iter_trait.ident.span()
            } else {
                iter_trait.ident.span()
            };
            errors.push_err(syn_err!(span, "You must specify the Iterator item type."));
            parse_str_as_type("()").expect("BUG")
        });
        let path_args = type_as_iter_path_arguments(iter_item);

        fn set_last_arguments(bounds: &mut syn::TraitBound, path_args: syn::PathArguments) {
            bounds.path.segments.last_mut().expect("BUG").arguments = path_args;
        }

        if de_iter_trait.is_implemented {
            set_last_arguments(&mut de_iter_trait.bound, path_args.clone());
        }
        set_last_arguments(&mut iter_trait.bound, path_args);
    }

    let mut impld_traits = Vec::new();
    let mut unimpld_traits = Vec::new();
    let trait_flags = trait_struct.as_ref().map(|_, x| x.is_implemented);
    let trait_spans = trait_struct.as_ref().map(|_, x| x.ident.span());

    for trait_ in trait_struct.to_vec() {
        if trait_.is_implemented {
            impld_traits.push(trait_);
        } else {
            unimpld_traits.push(arenas.alloc(trait_.ident.clone()));
        }
    }

    GetSupertraits {
        impld_traits,
        unimpld_traits,
        lifetime_bounds,
        iterator_item,
        deserialize_bound,
        trait_flags,
        trait_spans,
        errors,
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Extracts the Iterator::Item out of a path component.
fn extract_iterator_item<'a>(
    last_path_component: &syn::PathSegment,
    arenas: &'a Arenas,
) -> Option<&'a syn::Type> {
    use syn::{GenericArgument, PathArguments};

    let angle_brackets = match &last_path_component.arguments {
        PathArguments::AngleBracketed(x) => x,
        _ => return None,
    };

    for gen_arg in &angle_brackets.args {
        match gen_arg {
            GenericArgument::Binding(bind) if bind.ident == "Item" => {
                return Some(arenas.alloc(bind.ty.clone()));
            }
            _ => {}
        }
    }
    None
}

/// Converts a type to `<Item= ty >`.
fn type_as_iter_path_arguments(ty: syn::Type) -> syn::PathArguments {
    let x = syn::Binding {
        ident: parse_str_as_ident("Item"),
        eq_token: Default::default(),
        ty,
    };

    let x = syn::GenericArgument::Binding(x);

    let x = syn::AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Default::default(),
        args: iter::once(x).collect(),
        gt_token: Default::default(),
    };

    syn::PathArguments::AngleBracketed(x)
}

/// Extracts the lifetime in `Deserialize<'lt>` out of a path component.
#[allow(dead_code)]
fn extract_deserialize_lifetime<'a>(
    last_path_component: &syn::PathSegment,
    arenas: &'a Arenas,
) -> Result<&'a syn::Lifetime, syn::Error> {
    use syn::{GenericArgument, PathArguments};

    let angle_brackets = match &last_path_component.arguments {
        PathArguments::AngleBracketed(x) => x,
        _ => return_spanned_err!(last_path_component, "Expected a lifetime parameter inside"),
    };

    for gen_arg in &angle_brackets.args {
        if let GenericArgument::Lifetime(lt) = gen_arg {
            return Ok(arenas.alloc(lt.clone()));
        }
    }
    Err(spanned_err!(
        last_path_component,
        "Expected a lifetime parameter inside"
    ))
}
