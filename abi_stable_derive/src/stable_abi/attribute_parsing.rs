use as_derive_utils::{
    datastructure::{DataStructure, DataVariant, Field, FieldMap, TypeParamMap},
    parse_utils::{ret_err_on, ret_err_on_peek, ParseBufferExt},
    return_spanned_err, return_syn_err, spanned_err, syn_err,
    utils::SynErrorExt,
};

use syn::{
    parse::ParseBuffer, punctuated::Punctuated, token::Comma, Attribute, Expr, ExprLit, Ident, Lit,
    Token, Type, TypeParamBound, WherePredicate,
};

use std::{collections::HashSet, mem};

use core_extensions::{matches, IteratorExt};

use proc_macro2::Span;

use crate::*;

use crate::{
    attribute_parsing::contains_doc_hidden,
    impl_interfacetype::{parse_impl_interfacetype, ImplInterfaceType},
    parse_utils::{parse_str_as_ident, ParseBounds, ParsePunctuated},
    utils::{LinearResult, SynResultExt},
};

use super::{
    nonexhaustive::{
        EnumInterface, ExprOrType, NonExhaustive, UncheckedNEVariant, UncheckedNonExhaustive,
        UncheckedVariantConstructor,
    },
    prefix_types::{
        AccessorOrMaybe, FirstSuffixField, OnMissingField, PrefixKind, PrefixKindCtor,
        PrefixKindField,
    },
    reflection::{FieldAccessor, ModReflMode},
    repr_attrs::{
        DiscriminantRepr, Repr, ReprAttr, UncheckedReprAttr, UncheckedReprKind, REPR_ERROR_MSG,
    },
};

mod kw {
    syn::custom_keyword! {accessible_if}
    syn::custom_keyword! {accessor_bound}
    syn::custom_keyword! {assert_nonexhaustive}
    syn::custom_keyword! {align}
    syn::custom_keyword! {bounds}
    syn::custom_keyword! {bound}
    syn::custom_keyword! {Clone}
    syn::custom_keyword! {C}
    syn::custom_keyword! {Debug}
    syn::custom_keyword! {debug_print}
    syn::custom_keyword! {default}
    syn::custom_keyword! {Deref}
    syn::custom_keyword! {Deserialize}
    syn::custom_keyword! {Display}
    syn::custom_keyword! {Eq}
    syn::custom_keyword! {Error}
    syn::custom_keyword! {extra_checks}
    syn::custom_keyword! {Hash}
    syn::custom_keyword! {ident}
    syn::custom_keyword! {interface}
    syn::custom_keyword! {impl_InterfaceType}
    syn::custom_keyword! {impl_prefix_stable_abi}
    syn::custom_keyword! {kind}
    syn::custom_keyword! {last_prefix_field}
    syn::custom_keyword! {missing_field}
    syn::custom_keyword! {module_reflection}
    syn::custom_keyword! {Module}
    syn::custom_keyword! {not_stableabi}
    syn::custom_keyword! {Opaque}
    syn::custom_keyword! {option}
    syn::custom_keyword! {Ord}
    syn::custom_keyword! {packed}
    syn::custom_keyword! {panic}
    syn::custom_keyword! {PartialEq}
    syn::custom_keyword! {PartialOrd}
    syn::custom_keyword! {phantom_const_param}
    syn::custom_keyword! {phantom_field}
    syn::custom_keyword! {phantom_type_param}
    syn::custom_keyword! {prefix_bounds}
    syn::custom_keyword! {prefix_bound}
    syn::custom_keyword! {prefix_fields}
    syn::custom_keyword! {prefix_ref}
    syn::custom_keyword! {prefix_ref_docs}
    syn::custom_keyword! {Prefix}
    syn::custom_keyword! {pub_getter}
    syn::custom_keyword! {refl}
    syn::custom_keyword! {rename}
    syn::custom_keyword! {size}
    syn::custom_keyword! {Send}
    syn::custom_keyword! {Serialize}
    syn::custom_keyword! {Sync}
    syn::custom_keyword! {tag}
    syn::custom_keyword! {transparent}
    syn::custom_keyword! {traits}
    syn::custom_keyword! {unrecognized}
    syn::custom_keyword! {unsafe_allow_type_macros}
    syn::custom_keyword! {unsafe_change_type}
    syn::custom_keyword! {unsafe_opaque_fields}
    syn::custom_keyword! {unsafe_opaque_field}
    syn::custom_keyword! {unsafe_sabi_opaque_fields}
    syn::custom_keyword! {unsafe_sabi_opaque_field}
    syn::custom_keyword! {unsafe_unconstrained}
    syn::custom_keyword! {value}
    syn::custom_keyword! {Value}
    syn::custom_keyword! {with_boxed_constructor}
    syn::custom_keyword! {with_constructor}
    syn::custom_keyword! {with_field_indices}
    syn::custom_keyword! {WithNonExhaustive}
    syn::custom_keyword! {with}
}

pub(crate) struct StableAbiOptions<'a> {
    pub(crate) debug_print: bool,
    pub(crate) kind: StabilityKind<'a>,
    pub(crate) repr: ReprAttr,

    pub(crate) type_param_bounds: TypeParamMap<'a, ASTypeParamBound>,

    pub(crate) extra_bounds: Vec<WherePredicate>,

    pub(crate) tags: Option<syn::Expr>,
    pub(crate) extra_checks: Option<syn::Expr>,

    pub(crate) layout_ctor: FieldMap<LayoutConstructor>,

    pub(crate) override_field_accessor: FieldMap<Option<FieldAccessor<'a>>>,

    pub(crate) renamed_fields: FieldMap<Option<&'a Ident>>,
    pub(crate) changed_types: FieldMap<Option<&'a Type>>,

    pub(crate) doc_hidden_attr: Option<&'a TokenStream2>,

    pub(crate) mod_refl_mode: ModReflMode<usize>,

    pub(crate) impl_interfacetype: Option<ImplInterfaceType>,

    pub(crate) phantom_fields: Vec<(&'a Ident, &'a Type)>,
    pub(crate) phantom_type_params: Vec<&'a Type>,
    pub(crate) phantom_const_params: Vec<&'a syn::Expr>,

    pub(crate) const_idents: ConstIdents,

    pub(crate) allow_type_macros: bool,
    pub(crate) with_field_indices: bool,
}

//////////////////////

/// Identifiers of generated top-level constants.
pub struct ConstIdents {
    /// The identifier of a constant where the string in MonoSharedVars will be stored.
    pub(crate) strings: Ident,
}

//////////////////////

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) enum ASTypeParamBound {
    NoBound,
    GetStaticEquivalent,
    StableAbi,
}

impl Default for ASTypeParamBound {
    fn default() -> Self {
        ASTypeParamBound::StableAbi
    }
}

//////////////////////

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) enum LayoutConstructor {
    Regular,
    Opaque,
    SabiOpaque,
}

impl LayoutConstructor {
    pub(crate) fn is_opaque(self) -> bool {
        matches!(self, LayoutConstructor::Opaque { .. })
    }
}

impl From<ASTypeParamBound> for LayoutConstructor {
    fn from(bound: ASTypeParamBound) -> Self {
        match bound {
            ASTypeParamBound::NoBound => LayoutConstructor::Opaque,
            ASTypeParamBound::GetStaticEquivalent => LayoutConstructor::Opaque,
            ASTypeParamBound::StableAbi => LayoutConstructor::Regular,
        }
    }
}

impl Default for LayoutConstructor {
    fn default() -> Self {
        LayoutConstructor::Regular
    }
}

//////////////////////

pub(crate) enum StabilityKind<'a> {
    Value { impl_prefix_stable_abi: bool },
    Prefix(PrefixKind<'a>),
    NonExhaustive(NonExhaustive<'a>),
}

impl<'a> StabilityKind<'a> {
    pub(crate) fn field_accessor(
        &self,
        mod_refl_mode: ModReflMode<usize>,
        field: &Field<'a>,
    ) -> FieldAccessor<'a> {
        let is_public = field.is_public() && mod_refl_mode != ModReflMode::Opaque;
        match (is_public, self) {
            (false, _) => FieldAccessor::Opaque,
            (true, StabilityKind::Value { .. }) | (true, StabilityKind::NonExhaustive { .. }) => {
                FieldAccessor::Direct
            }
            (true, StabilityKind::Prefix(prefix)) => prefix.field_accessor(field),
        }
    }
}

impl<'a> StableAbiOptions<'a> {
    fn new(
        ds: &'a DataStructure<'a>,
        mut this: StableAbiAttrs<'a>,
        arenas: &'a Arenas,
    ) -> Result<Self, syn::Error> {
        let mut phantom_fields = Vec::<(&'a Ident, &'a Type)>::new();

        let repr = ReprAttr::new(this.repr)?;

        let mut errors = LinearResult::ok(());

        let kind = match this.kind {
            _ if repr.is_repr_transparent() => {
                // let field=&ds.variants[0].fields[0];

                // let accessor_bound=syn::parse_str::<WherePredicate>(
                //     &format!("({}):__StableAbi",(&field.mutated_ty).into_token_stream())
                // ).expect(concat!(file!(),"-",line!()));
                // this.extra_bounds.push(accessor_bound);

                StabilityKind::Value {
                    impl_prefix_stable_abi: false,
                }
            }
            UncheckedStabilityKind::Value {
                impl_prefix_stable_abi,
            } => StabilityKind::Value {
                impl_prefix_stable_abi,
            },
            UncheckedStabilityKind::Prefix(prefix) => PrefixKindCtor::<'a> {
                arenas,
                struct_name: ds.name,
                first_suffix_field: this.first_suffix_field,
                prefix_ref: prefix.prefix_ref,
                prefix_fields: prefix.prefix_fields,
                replacing_prefix_ref_docs: prefix.replacing_prefix_ref_docs,
                fields: mem::replace(&mut this.prefix_kind_fields, FieldMap::empty()).map(
                    |fi, pk_field| {
                        AccessorOrMaybe::new(
                            fi,
                            this.first_suffix_field,
                            pk_field,
                            this.default_on_missing_fields.unwrap_or_default(),
                        )
                    },
                ),
                prefix_bounds: this.prefix_bounds,
                accessor_bounds: this.accessor_bounds,
            }
            .make()
            .piped(StabilityKind::Prefix),
            UncheckedStabilityKind::NonExhaustive(nonexhaustive) => {
                let ne_variants = this.ne_variants;
                nonexhaustive
                    .piped(|x| NonExhaustive::new(x, ne_variants, ds, arenas))?
                    .piped(StabilityKind::NonExhaustive)
            }
        };

        match (repr.variant, ds.data_variant) {
            (Repr::Transparent, DataVariant::Struct) => {}
            (Repr::Transparent, _) => {
                errors.push_err(syn_err!(
                    *repr.span,
                    "\nAbiStable does not suport non-struct #[repr(transparent)] types.\n"
                ));
            }
            (Repr::Int { .. }, DataVariant::Enum) => {}
            (Repr::Int { .. }, _) => {
                errors.push_err(syn_err!(
                    *repr.span,
                    "AbiStable does not suport non-enum #[repr(<some_integer_type>)] types."
                ));
            }
            (Repr::C { .. }, _) => {}
        }

        let mod_refl_mode = match this.mod_refl_mode {
            Some(ModReflMode::Module) => ModReflMode::Module,
            Some(ModReflMode::Opaque) => ModReflMode::Opaque,
            Some(ModReflMode::DelegateDeref(())) => {
                let index = phantom_fields.len();
                let field_ty =
                    syn::parse_str::<Type>("<Self as __sabi_re::GetPointerKind >::PtrTarget")
                        .expect("BUG")
                        .piped(|x| arenas.alloc(x));

                let dt = arenas.alloc(parse_str_as_ident("deref_target"));
                phantom_fields.push((dt, field_ty));

                [
                    "Self: __sabi_re::GetPointerKind",
                    "<Self as __sabi_re::GetPointerKind>::Target: __StableAbi",
                ]
                .iter()
                .map(|x| syn::parse_str::<WherePredicate>(x).expect("BUG"))
                .extending(&mut this.extra_bounds);

                ModReflMode::DelegateDeref(index)
            }
            None if ds.has_public_fields() => ModReflMode::Module,
            None => ModReflMode::Opaque,
        };

        phantom_fields.extend(this.extra_phantom_fields);
        phantom_fields.extend(this.phantom_type_params.iter().cloned().enumerate().map(
            |(i, ty)| {
                let x = format!("_phantom_ty_param_{}", i);
                let name = arenas.alloc(parse_str_as_ident(&x));
                (name, ty)
            },
        ));

        let doc_hidden_attr = if this.is_hidden {
            Some(arenas.alloc(quote!(#[doc(hidden)])))
        } else {
            None
        };

        let const_idents = ConstIdents {
            strings: parse_str_as_ident(&format!("_SHARED_VARS_STRINGS_{}", ds.name)),
        };

        errors.into_result()?;

        Ok(StableAbiOptions {
            debug_print: this.debug_print,
            kind,
            repr,
            extra_bounds: this.extra_bounds,
            type_param_bounds: this.type_param_bounds,
            layout_ctor: this.layout_ctor,
            renamed_fields: this.renamed_fields,
            changed_types: this.changed_types,
            override_field_accessor: this.override_field_accessor,
            tags: this.tags,
            extra_checks: this.extra_checks,
            impl_interfacetype: this.impl_interfacetype,
            phantom_fields,
            phantom_type_params: this.phantom_type_params,
            phantom_const_params: this.phantom_const_params,
            allow_type_macros: this.allow_type_macros,
            with_field_indices: this.with_field_indices,
            const_idents,
            mod_refl_mode,
            doc_hidden_attr,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct StableAbiAttrs<'a> {
    debug_print: bool,
    kind: UncheckedStabilityKind<'a>,
    repr: UncheckedReprAttr,

    extra_bounds: Vec<WherePredicate>,

    tags: Option<syn::Expr>,
    extra_checks: Option<syn::Expr>,

    first_suffix_field: FirstSuffixField,
    default_on_missing_fields: Option<OnMissingField<'a>>,
    prefix_kind_fields: FieldMap<PrefixKindField<'a>>,

    prefix_bounds: Vec<WherePredicate>,

    type_param_bounds: TypeParamMap<'a, ASTypeParamBound>,

    layout_ctor: FieldMap<LayoutConstructor>,

    ne_variants: Vec<UncheckedNEVariant>,

    override_field_accessor: FieldMap<Option<FieldAccessor<'a>>>,

    renamed_fields: FieldMap<Option<&'a Ident>>,
    changed_types: FieldMap<Option<&'a Type>>,

    accessor_bounds: FieldMap<Vec<TypeParamBound>>,

    extra_phantom_fields: Vec<(&'a Ident, &'a Type)>,
    phantom_type_params: Vec<&'a Type>,
    phantom_const_params: Vec<&'a syn::Expr>,

    impl_interfacetype: Option<ImplInterfaceType>,

    mod_refl_mode: Option<ModReflMode<()>>,

    allow_type_macros: bool,
    with_field_indices: bool,
    is_hidden: bool,

    errors: LinearResult<()>,
}

#[derive(Clone)]
enum UncheckedStabilityKind<'a> {
    Value { impl_prefix_stable_abi: bool },
    Prefix(UncheckedPrefixKind<'a>),
    NonExhaustive(UncheckedNonExhaustive<'a>),
}

#[derive(Copy, Clone)]
struct UncheckedPrefixKind<'a> {
    prefix_ref: Option<&'a Ident>,
    prefix_fields: Option<&'a Ident>,
    replacing_prefix_ref_docs: &'a [syn::Expr],
}

impl<'a> Default for UncheckedStabilityKind<'a> {
    fn default() -> Self {
        UncheckedStabilityKind::Value {
            impl_prefix_stable_abi: false,
        }
    }
}

#[derive(Copy, Clone)]
enum ParseContext<'a> {
    TypeAttr,
    Variant {
        variant_index: usize,
    },
    Field {
        field_index: usize,
        field: &'a Field<'a>,
    },
}

/// Parses the attributes for the `StableAbi` derive macro.
pub(crate) fn parse_attrs_for_stable_abi<'a, I>(
    attrs: I,
    ds: &'a DataStructure<'a>,
    arenas: &'a Arenas,
) -> Result<StableAbiOptions<'a>, syn::Error>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    let mut this = StableAbiAttrs::default();

    this.layout_ctor = FieldMap::defaulted(ds);
    this.prefix_kind_fields = FieldMap::defaulted(ds);
    this.renamed_fields = FieldMap::defaulted(ds);
    this.override_field_accessor = FieldMap::defaulted(ds);
    this.accessor_bounds = FieldMap::defaulted(ds);
    this.changed_types = FieldMap::defaulted(ds);
    this.ne_variants.resize(
        ds.variants.len(),
        UncheckedNEVariant {
            constructor: None,
            is_hidden: false,
        },
    );

    this.type_param_bounds = TypeParamMap::defaulted(ds);

    parse_inner(&mut this, attrs, ParseContext::TypeAttr, arenas);

    for (variant_index, variant) in ds.variants.iter().enumerate() {
        parse_inner(
            &mut this,
            variant.attrs,
            ParseContext::Variant { variant_index },
            arenas,
        );
        for (field_index, field) in variant.fields.iter().enumerate() {
            parse_inner(
                &mut this,
                field.attrs,
                ParseContext::Field { field, field_index },
                arenas,
            );
        }
    }

    this.errors.take()?;

    StableAbiOptions::new(ds, this, arenas)
}

/// Parses an individual attribute
fn parse_inner<'a, I>(
    this: &mut StableAbiAttrs<'a>,
    attrs: I,
    pctx: ParseContext<'a>,
    arenas: &'a Arenas,
) where
    I: IntoIterator<Item = &'a Attribute>,
{
    for attr in attrs {
        if attr.path.is_ident("sabi") {
            attr.parse_args_with(|input: &'_ ParseBuffer<'_>| {
                input.for_each_separated(Token!(,), |input| {
                    parse_sabi_attr(this, pctx, input, arenas)
                })
            })
        } else if attr.path.is_ident("doc") {
            (|| -> syn::Result<()> {
                let is_hidden =
                    syn::parse::Parser::parse2(contains_doc_hidden, attr.tokens.clone())?;

                match pctx {
                    ParseContext::TypeAttr { .. } => {
                        this.is_hidden = is_hidden;
                    }
                    ParseContext::Variant { variant_index, .. } => {
                        this.ne_variants[variant_index].is_hidden = is_hidden;
                    }
                    ParseContext::Field { .. } => {}
                }

                Ok(())
            })()
        } else if attr.path.is_ident("repr") {
            fn parse_int_arg(input: &'_ ParseBuffer<'_>) -> Result<u32, syn::Error> {
                input.parse_paren_buffer()?.parse_int::<u32>()
            }

            attr.parse_args_with(|input: &'_ ParseBuffer<'_>| {
                input.for_each_separated(Token!(,), |input| {
                    let span = input.span();
                    if input.check_parse(kw::C)? {
                        this.repr.set_repr_kind(UncheckedReprKind::C, span)
                    } else if input.check_parse(kw::transparent)? {
                        this.repr
                            .set_repr_kind(UncheckedReprKind::Transparent, span)
                    } else if input.check_parse(kw::align)? {
                        this.repr.set_aligned(parse_int_arg(input)?)
                    } else if input.check_parse(kw::packed)? {
                        if input.peek(syn::token::Paren) {
                            this.repr.set_packed(Some(parse_int_arg(input)?))
                        } else {
                            this.repr.set_packed(None)
                        }
                    } else if let Some(dr) = DiscriminantRepr::from_parser(input) {
                        this.repr.set_discriminant_repr(dr, span)
                    } else {
                        Err(syn_err!(
                            span,
                            "repr attribute not currently recognized by this macro.{}",
                            REPR_ERROR_MSG
                        ))
                    }?;

                    Ok(())
                })
            })
        } else {
            Ok(())
        }
        .combine_into_err(&mut this.errors);
    }
}

/// Parses the contents of a `#[sabi( .. )]` attribute.
fn parse_sabi_attr<'a>(
    this: &mut StableAbiAttrs<'a>,
    pctx: ParseContext<'a>,
    input: &'_ ParseBuffer<'_>,
    arenas: &'a Arenas,
) -> Result<(), syn::Error> {
    fn make_err(input: &ParseBuffer) -> syn::Error {
        input.error("unrecognized attribute")
    }

    if let ParseContext::TypeAttr = pctx {
        fn parse_pred(input: &ParseBuffer) -> Result<WherePredicate, syn::Error> {
            let input = input.parse_paren_buffer()?;

            ret_err_on_peek! {input, syn::Lit, "where predicate", "literal"}

            input
                .parse::<WherePredicate>()
                .map_err(|e| e.prepend_msg("while parsing where predicate: "))
        }

        fn parse_preds(
            input: &ParseBuffer<'_>,
        ) -> Result<Punctuated<WherePredicate, Comma>, syn::Error> {
            let input = input.parse_paren_buffer()?;

            ret_err_on_peek! {input, syn::Lit, "where predicates", "literal"}

            match input.parse::<ParsePunctuated<WherePredicate, Comma>>() {
                Ok(x) => Ok(x.list),
                Err(e) => Err(e.prepend_msg("while parsing where predicates: ")),
            }
        }

        if input.check_parse(kw::bound)? {
            this.extra_bounds.push(parse_pred(input)?);
        } else if input.check_parse(kw::prefix_bound)? {
            this.prefix_bounds.push(parse_pred(input)?);
        } else if input.check_parse(kw::bounds)? {
            this.extra_bounds.extend(parse_preds(input)?);
        } else if input.check_parse(kw::prefix_bounds)? {
            this.prefix_bounds.extend(parse_preds(input)?);
        } else if input.check_parse(kw::phantom_field)? {
            input.parse_paren_with(|input| {
                let fname = arenas.alloc(input.parse::<Ident>()?);
                let _ = input.parse::<Token!(:)>()?;
                let ty = arenas.alloc(input.parse_type()?);
                this.extra_phantom_fields.push((fname, ty));
                Ok(())
            })?;
        } else if input.check_parse(kw::phantom_type_param)? {
            input.parse::<Token!(=)>()?;
            let ty = arenas.alloc(input.parse_type()?);
            this.phantom_type_params.push(ty);
        } else if input.check_parse(kw::phantom_const_param)? {
            input.parse::<Token!(=)>()?;
            let constant = arenas.alloc(input.parse_expr()?);
            ret_err_on! {
                matches!(constant, syn::Expr::Lit(ExprLit{lit: Lit::Str{..}, ..})),
                syn::spanned::Spanned::span(constant),
                "`impl StableAbi + Eq + Debug` expression",
                "string literal",
            }
            this.phantom_const_params.push(constant);
        } else if let Some(tag_token) = input.peek_parse(kw::tag)? {
            input.parse::<Token!(=)>()?;

            ret_err_on_peek! {
                input,
                syn::Lit,
                "`abi_stable::type_layout::Tag` expression",
                "literal",
            }

            let bound = input.parse_expr()?;
            if this.tags.is_some() {
                return_syn_err!(
                    tag_token.span,
                    "\
                    Cannot specify multiple tags,\
                    you must choose whether you want array or set semantics \
                    when adding more tags.\n\
                    \n\
                    For multiple elements you can do:\n\
                    \n\
                    - `tag![[ tag0,tag1 ]]` or `Tag::arr(&[ tag0,tag1 ])` :\n\
                        \tThis will require that the tags match exactly between \
                        interface and implementation.\n\
                    \n\
                    - `tag!{{ tag0,tag1 }}` or `Tag::set(&[ tag0,tag1 ])` :\n\
                        \tThis will require that the tags in the interface are \
                        a subset of the implementation.\n\
                    ",
                );
            }
            this.tags = Some(bound);
        } else if let Some(ec_token) = input.peek_parse(kw::extra_checks)? {
            input.parse::<Token!(=)>()?;

            ret_err_on_peek! {input, syn::Lit, "`impl ExtraChecks` expression", "literal"}

            let bound = input.parse_expr()?;
            if this.extra_checks.is_some() {
                return_syn_err!(
                    ec_token.span,
                    "cannot use the `#[sabi(extra_checks=\"\")]` \
                     attribute multiple times,\
                    "
                );
            }

            this.extra_checks = Some(bound);
        } else if input.check_parse(kw::missing_field)? {
            let on_missing = &mut this.default_on_missing_fields;
            if on_missing.is_some() {
                return Err(
                    input.error("Cannot use this attribute multiple times on the same type")
                );
            }
            *on_missing = Some(input.parse_paren_with(|i| parse_missing_field(i, arenas))?);
        } else if input.check_parse(kw::kind)? {
            input.parse_paren_with(|input| {
                if input.check_parse(kw::Value)? {
                    this.kind = UncheckedStabilityKind::Value {
                        impl_prefix_stable_abi: false,
                    };
                } else if input.check_parse(kw::Prefix)? {
                    if input.peek(syn::token::Paren) {
                        let prefix = input
                            .parse_paren_with(|input| parse_prefix_type_list(input, arenas))?;
                        this.kind = UncheckedStabilityKind::Prefix(prefix);
                    } else {
                        let prefix = parse_prefix_type_list(input, arenas)?;
                        this.kind = UncheckedStabilityKind::Prefix(prefix);
                    }
                } else if input.check_parse(kw::WithNonExhaustive)? {
                    let nonexhaustive =
                        input.parse_paren_with(|input| parse_non_exhaustive_list(input, arenas))?;
                    this.kind = UncheckedStabilityKind::NonExhaustive(nonexhaustive);
                } else {
                    return Err(input.error("invalid #[kind(..)] attribute"));
                }
                Ok(())
            })?;
        } else if input.check_parse(kw::debug_print)? {
            this.debug_print = true;
        } else if input.check_parse(kw::module_reflection)? {
            input
                .parse_paren_buffer()?
                .for_each_separated(Token!(,), |input| {
                    if this.mod_refl_mode.is_some() {
                        return Err(input.error("Cannot use this attribute multiple times"));
                    }

                    if input.check_parse(kw::Module)? {
                        this.mod_refl_mode = Some(ModReflMode::Module);
                    } else if input.check_parse(kw::Opaque)? {
                        this.mod_refl_mode = Some(ModReflMode::Opaque);
                    } else if input.check_parse(kw::Deref)? {
                        this.mod_refl_mode = Some(ModReflMode::DelegateDeref(()));
                    } else {
                        return Err(input.error("invalid #[module_reflection(..)] attribute."));
                    }

                    Ok(())
                })?;
        } else if input.check_parse(kw::not_stableabi)? {
            input
                .parse_paren_buffer()?
                .for_each_separated(Token!(,), |input| {
                    let type_param = input.parse::<Ident>().map_err(|_| {
                        input.error(
                            "\
                        invalid #[not_stableabi(..)] attribute\
                        (it must be the identifier of a type parameter).\
                    ",
                        )
                    })?;

                    *this.type_param_bounds.get_mut(&type_param)? =
                        ASTypeParamBound::GetStaticEquivalent;

                    Ok(())
                })?;
        } else if input.check_parse(kw::unsafe_unconstrained)? {
            input
                .parse_paren_buffer()?
                .for_each_separated(Token!(,), |input| {
                    let type_param = input.parse::<Ident>().map_err(|_| {
                        input.error(
                            "\
                        invalid #[unsafe_unconstrained(..)] attribute\
                        (it must be the identifier of a type parameter).\
                    ",
                        )
                    })?;

                    *this.type_param_bounds.get_mut(&type_param)? = ASTypeParamBound::NoBound;

                    Ok(())
                })?;
        } else if let Some(attr_ident) = input.peek_parse(kw::impl_InterfaceType)? {
            if this.impl_interfacetype.is_some() {
                return_spanned_err!(attr_ident, "Cannot use this attribute multiple times")
            }
            let content = input.parse_paren_buffer()?;
            this.impl_interfacetype = Some(parse_impl_interfacetype(&content)?);
        } else if input.check_parse(kw::with_constructor)? {
            this.ne_variants
                .iter_mut()
                .for_each(|x| x.constructor = Some(UncheckedVariantConstructor::Regular));
        } else if input.check_parse(kw::with_boxed_constructor)? {
            this.ne_variants
                .iter_mut()
                .for_each(|x| x.constructor = Some(UncheckedVariantConstructor::Boxed));
        } else if input.check_parse(kw::unsafe_opaque_fields)? {
            this.layout_ctor
                .iter_mut()
                .for_each(|(_, x)| *x = LayoutConstructor::Opaque);
        } else if input.check_parse(kw::unsafe_sabi_opaque_fields)? {
            this.layout_ctor
                .iter_mut()
                .for_each(|(_, x)| *x = LayoutConstructor::SabiOpaque);
        } else if input.check_parse(kw::unsafe_allow_type_macros)? {
            this.allow_type_macros = true;
        } else if input.check_parse(kw::with_field_indices)? {
            this.with_field_indices = true;
        } else if input.check_parse(kw::impl_prefix_stable_abi)? {
            this.kind = UncheckedStabilityKind::Value {
                impl_prefix_stable_abi: true,
            };
        } else {
            return Err(make_err(input));
        }
    } else if let ParseContext::Variant { variant_index } = pctx {
        if input.check_parse(kw::with_constructor)? {
            this.ne_variants[variant_index].constructor =
                Some(UncheckedVariantConstructor::Regular);
        } else if input.check_parse(kw::with_boxed_constructor)? {
            this.ne_variants[variant_index].constructor = Some(UncheckedVariantConstructor::Boxed);
        } else {
            return Err(make_err(input));
        }
    } else if let ParseContext::Field { field, field_index } = pctx {
        if input.check_parse(kw::unsafe_opaque_field)? {
            this.layout_ctor[field] = LayoutConstructor::Opaque;
        } else if input.check_parse(kw::unsafe_sabi_opaque_field)? {
            this.layout_ctor[field] = LayoutConstructor::SabiOpaque;
        } else if input.check_parse(kw::last_prefix_field)? {
            let field_pos = field_index + 1;
            this.first_suffix_field = FirstSuffixField { field_pos };
        } else if input.check_parse(kw::rename)? {
            input.parse::<Token!(=)>()?;
            let renamed = input.parse::<Ident>()?.piped(|x| arenas.alloc(x));
            this.renamed_fields.insert(field, Some(renamed));
        } else if input.check_parse(kw::unsafe_change_type)? {
            input.parse::<Token!(=)>()?;
            let changed_type = input.parse_type()?.piped(|x| arenas.alloc(x));
            this.changed_types.insert(field, Some(changed_type));
        } else if input.check_parse(kw::accessible_if)? {
            input.parse::<Token!(=)>()?;

            let expr = input.parse_expr()?;

            if let Expr::Lit(ExprLit { lit, .. }) = &expr {
                ret_err_on! {
                    !matches!(lit, Lit::Bool{..}),
                    syn::spanned::Spanned::span(&expr),
                    "`bool` expression",
                    "non-bool literal",
                }
            }

            let expr = arenas.alloc(expr);
            this.prefix_kind_fields[field].accessible_if = Some(expr);
        } else if input.check_parse(kw::accessor_bound)? {
            input.parse::<Token!(=)>()?;
            let bound = input.parse::<ParseBounds>()?.list;
            this.accessor_bounds[field].extend(bound);
        } else if input.check_parse(kw::bound)? {
            input.parse::<Token!(=)>()?;
            let bounds = input.parse::<ParseBounds>()?.list;
            let preds = where_predicate_from(field.ty.clone(), bounds);
            this.extra_bounds.push(preds);
        } else if input.check_parse(kw::missing_field)? {
            let on_missing_field =
                input.parse_paren_with(|input| parse_missing_field(input, arenas))?;
            let on_missing = &mut this.prefix_kind_fields[field].on_missing;
            if on_missing.is_some() {
                return Err(
                    input.error("Cannot use this attribute multiple times on the same field")
                );
            }
            *on_missing = Some(on_missing_field);
        } else if input.check_parse(kw::refl)? {
            input.parse_paren_with(|input| parse_refl_field(this, field, input, arenas))?;
        } else {
            return Err(make_err(input));
        }
    }
    Ok(())
}

/// Parses the `#[sabi(refl = ...)` attribute.
fn parse_refl_field<'a>(
    this: &mut StableAbiAttrs<'a>,
    field: &'a Field<'a>,
    input: &ParseBuffer,
    arenas: &'a Arenas,
) -> Result<(), syn::Error> {
    input.for_each_separated(Token!(,), |input| {
        if input.check_parse(kw::pub_getter)? {
            input.parse::<Token!(=)>()?;
            let function = arenas.alloc(input.parse::<Ident>()?);
            this.override_field_accessor[field] = Some(FieldAccessor::Method {
                name: Some(function),
            });
        } else {
            return Err(input.error("invalid #[sabi(refl(..))] attribute"));
        }
        Ok(())
    })
}

/// Parses the contents of #[sabi(missing_field( ... ))]
fn parse_missing_field<'a>(
    input: &ParseBuffer,
    arenas: &'a Arenas,
) -> Result<OnMissingField<'a>, syn::Error> {
    const ATTRIBUTE_MSG: &str = "

Valid Attributes:

    `#[sabi(missing_field(panic))]`
    This panics if the field doesn't exist.

    `#[sabi(missing_field(option))]`
    This returns Some(field_value) if the field exists,None if the field doesn't exist.
    This is the default.

    `#[sabi(missing_field(with=\"somefunction\"))]`
    This returns `somefunction()` if the field doesn't exist.
    
    `#[sabi(missing_field(value=\"some_expression\"))]`
    This returns `(some_expression)` if the field doesn't exist.
    
    `#[sabi(missing_field(default))]`
    This returns `Default::default` if the field doesn't exist.

";
    if input.check_parse(kw::option)? {
        Ok(OnMissingField::ReturnOption)
    } else if input.check_parse(kw::panic)? {
        Ok(OnMissingField::Panic)
    } else if input.check_parse(kw::default)? {
        Ok(OnMissingField::Default_)
    } else if input.check_parse(kw::with)? {
        input.parse::<Token!(=)>()?;
        let function = input.parse::<syn::Path>()?.piped(|i| arenas.alloc(i));
        Ok(OnMissingField::With { function })
    } else if input.check_parse(kw::value)? {
        input.parse::<Token!(=)>()?;
        let value = input.parse_expr()?.piped(|i| arenas.alloc(i));
        Ok(OnMissingField::Value { value })
    } else if input.is_empty() {
        Err(syn_err!(
            input.span(),
            "Error:Expected one attribute inside `missing_field(..)`\n{}",
            ATTRIBUTE_MSG
        ))
    } else {
        Err(syn_err!(
            input.span(),
            "Invalid attribute.\n{}",
            ATTRIBUTE_MSG,
        ))
    }
}

/// Parses the contents of #[sabi(kind(Prefix( ... )))]
fn parse_prefix_type_list<'a>(
    input: &ParseBuffer,
    arenas: &'a Arenas,
) -> Result<UncheckedPrefixKind<'a>, syn::Error> {
    let mut prefix_ref = None;
    let mut prefix_fields = None;
    let mut replacing_prefix_ref_docs = Vec::new();

    input.for_each_separated(Token!(,), |input| {
        if input.check_parse(kw::prefix_ref)? {
            input.parse::<Token!(=)>()?;
            prefix_ref = Some(arenas.alloc(input.parse::<Ident>()?));
        } else if input.check_parse(kw::prefix_ref_docs)? {
            input.parse::<Token!(=)>()?;
            replacing_prefix_ref_docs.push(input.parse_expr()?);
        } else if input.check_parse(kw::prefix_fields)? {
            input.parse::<Token!(=)>()?;
            prefix_fields = Some(arenas.alloc(input.parse::<Ident>()?));
        } else {
            return Err(input.error(
                "invalid #[sabi(kind(Prefix(  )))] attribute, it must be one of:\n\
                 - prefix_ref = NameOfPrefixPointerType\n\
                 - prefix_fields = NameOfPrefixFieldsStruct\n\
                ",
            ));
        }
        Ok(())
    })?;

    Ok(UncheckedPrefixKind {
        prefix_ref,
        prefix_fields,
        replacing_prefix_ref_docs: arenas.alloc(replacing_prefix_ref_docs),
    })
}

/// Parses the contents of #[sabi(kind(WithNonExhaustive( ... )))]
fn parse_non_exhaustive_list<'a>(
    input: &ParseBuffer,
    arenas: &'a Arenas,
) -> Result<UncheckedNonExhaustive<'a>, syn::Error> {
    let trait_set_strs = [
        "Clone",
        "Display",
        "Debug",
        "Eq",
        "PartialEq",
        "Ord",
        "PartialOrd",
        "Hash",
        "Deserialize",
        "Serialize",
        "Send",
        "Sync",
        "Error",
    ];

    let trait_set = trait_set_strs
        .iter()
        .map(|e| arenas.alloc(Ident::new(e, Span::call_site())))
        .collect::<HashSet<&'a Ident>>();

    let trait_err = |trait_ident: &Ident| -> syn::Error {
        spanned_err!(
            trait_ident,
            "Invalid trait in  #[sabi(kind(WithNonExhaustive(traits())))].\n\
             Valid traits:\n\t{}\
            ",
            trait_set_strs.join("\n\t")
        )
    };

    fn both_err(span: Span) -> syn::Error {
        syn_err!(
            span,
            "Cannot use both `interface=\"...\"` and `traits(...)`"
        )
    }

    let mut this = UncheckedNonExhaustive::default();

    let mut errors = LinearResult::ok(());

    input.for_each_separated(Token!(,), |input| {
        fn parse_expr_or_type<'a>(
            input: &ParseBuffer,
            arenas: &'a Arenas,
        ) -> Result<ExprOrType<'a>, syn::Error> {
            if input.peek(syn::LitInt) {
                Ok(ExprOrType::Int(input.parse_int::<usize>()?))
            } else if input.peek(syn::token::Brace) {
                let expr = input.parse::<syn::Expr>()?;
                Ok(ExprOrType::Expr(arenas.alloc(expr)))
            } else {
                ret_err_on_peek! {
                    input,
                    syn::LitStr,
                    "either integer literal or type",
                    "string literal",
                }

                Ok(ExprOrType::Type(arenas.alloc(input.parse_type()?)))
            }
        }

        if input.check_parse(kw::align)? {
            input.parse::<Token!(=)>()?;
            this.alignment = Some(parse_expr_or_type(input, arenas)?);
        } else if input.check_parse(kw::size)? {
            input.parse::<Token!(=)>()?;
            this.size = Some(parse_expr_or_type(input, arenas)?);
        } else if input.check_parse(kw::assert_nonexhaustive)? {
            if input.peek(syn::token::Paren) {
                input
                    .parse_paren_buffer()?
                    .for_each_separated(Token!(,), |input| {
                        let ty = arenas.alloc(input.parse_type()?);
                        this.assert_nonexh.push(ty);
                        Ok(())
                    })?;
            } else {
                input.parse::<Token!(=)>()?;
                let ty = arenas.alloc(input.parse_type()?);
                this.assert_nonexh.push(ty);
            }
        } else if let Some(in_token) = input.peek_parse(kw::interface)? {
            input.parse::<Token!(=)>()?;
            let ty = arenas.alloc(input.parse_type()?);
            if this.enum_interface.is_some() {
                return Err(both_err(in_token.span));
            }
            this.enum_interface = Some(EnumInterface::Old(ty));
        } else if let Some(traits_token) = input.peek_parse(kw::traits)? {
            let enum_interface = match &mut this.enum_interface {
                Some(EnumInterface::New(x)) => x,
                Some(EnumInterface::Old { .. }) => {
                    return Err(both_err(traits_token.span));
                }
                x @ None => {
                    *x = Some(EnumInterface::New(Default::default()));
                    match x {
                        Some(EnumInterface::New(x)) => x,
                        _ => unreachable!(),
                    }
                }
            };

            input
                .parse_paren_buffer()?
                .for_each_separated(Token!(,), |input| {
                    let ident = input.parse::<Ident>()?;

                    let is_impld = if input.check_parse(Token!(=))? {
                        input.parse::<syn::LitBool>()?.value
                    } else {
                        true
                    };

                    match trait_set.get(&ident) {
                        Some(&trait_ident) => {
                            if is_impld {
                                &mut enum_interface.impld
                            } else {
                                &mut enum_interface.unimpld
                            }
                            .push(trait_ident);
                        }
                        None => errors.push_err(trait_err(&ident)),
                    }

                    Ok(())
                })?;
        } else {
            return Err(input.error("invalid #[sabi(kind(WithNonExhaustive(....)))] attribute"));
        }
        Ok(())
    })?;

    errors.into_result().map(|_| this)
}

////////////////////////////////////////////////////////////////////////////////

fn where_predicate_from(
    ty: syn::Type,
    bounds: Punctuated<TypeParamBound, syn::token::Add>,
) -> syn::WherePredicate {
    let x = syn::PredicateType {
        lifetimes: None,
        bounded_ty: ty,
        colon_token: Default::default(),
        bounds,
    };
    syn::WherePredicate::Type(x)
}
