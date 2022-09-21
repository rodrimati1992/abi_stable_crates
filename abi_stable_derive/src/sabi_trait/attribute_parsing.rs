use super::{TraitDefinition, *};

use as_derive_utils::parse_utils::ParseBufferExt;

use syn::{parse::ParseBuffer, Attribute, ItemTrait, TraitItem, TraitItemMethod};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::{arenas::Arenas, attribute_parsing::contains_doc_hidden, utils::LinearResult};

/// Configuration parsed from the helper attributes of `#[sabi_trait]`
pub(crate) struct SabiTraitOptions<'a> {
    /// Whether the output of the proc-macro is printed with println.
    pub(crate) debug_print_trait: bool,
    pub(crate) debug_output_tokens: bool,
    pub(crate) doc_hidden_attr: Option<&'a TokenStream2>,
    pub(crate) trait_definition: TraitDefinition<'a>,
}

impl<'a> SabiTraitOptions<'a> {
    fn new(
        trait_: &'a ItemTrait,
        this: SabiTraitAttrs<'a>,
        arenas: &'a Arenas,
        ctokens: &'a CommonTokens,
    ) -> Result<Self, syn::Error> {
        let doc_hidden_attr = if this.is_hidden {
            Some(arenas.alloc(quote!(#[doc(hidden)])))
        } else {
            None
        };

        Ok(Self {
            debug_print_trait: this.debug_print_trait,
            debug_output_tokens: this.debug_output_tokens,
            doc_hidden_attr,
            trait_definition: TraitDefinition::new(trait_, this, arenas, ctokens)?,
        })
    }
}

mod kw {
    syn::custom_keyword! {no_default_fallback}
    syn::custom_keyword! {debug_print_trait}
    syn::custom_keyword! {debug_output_tokens}
    syn::custom_keyword! {use_dyntrait}
    syn::custom_keyword! {use_dyn_trait}
    syn::custom_keyword! {no_trait_impl}
}

////////////////////////////////////////////////////////////////////////////////

/// The attributes used in the vtable,and the trait.
#[derive(Debug, Clone, Default)]
pub(crate) struct OwnedDeriveAndOtherAttrs {
    /// The attributes used in the vtable.
    pub(crate) derive_attrs: Vec<Attribute>,
    /// The attributes used in the trait.
    pub(crate) other_attrs: Vec<Attribute>,
}

////////////////////////////////////////////////////////////////////////////////

/// The `syn` type for methods,as well as its attributes split by where they are used.
#[derive(Debug, Clone)]
pub(crate) struct MethodWithAttrs<'a> {
    /// The attributes used in the vtable,and the trait.
    pub(crate) attrs: OwnedDeriveAndOtherAttrs,
    pub(crate) item: &'a TraitItemMethod,
}

impl<'a> MethodWithAttrs<'a> {
    /// Constructs a `MethodWithAttrs` with no attributes.
    fn new(item: &'a TraitItemMethod) -> Self {
        Self {
            attrs: OwnedDeriveAndOtherAttrs {
                derive_attrs: Vec::new(),
                other_attrs: Vec::new(),
            },
            item,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A datastructure used while parsing the helper attributes of #[sabi_trait].
#[derive(Default)]
pub(super) struct SabiTraitAttrs<'a> {
    /// Whether the output of the proc-macro is printed with println.
    pub(super) debug_print_trait: bool,
    /// The attributes used in the vtable,and the trait.
    pub(super) attrs: OwnedDeriveAndOtherAttrs,
    /// The `syn` type for methods,as well as their attributes split by where they are used.
    pub(super) methods_with_attrs: Vec<MethodWithAttrs<'a>>,
    /// Which type to use as the underlying implementation of the trait object,
    /// either DynTrait or RObject.
    pub(super) which_object: WhichObject,
    /// If true,removes the `impl Trait for Trait_TO`
    pub(super) disable_trait_impl: bool,
    /// If true,doesn't use the default implementation of methods when
    /// the vtable entry is absent.
    pub(super) disable_inherent_default: Vec<bool>,

    pub(super) is_hidden: bool,
    pub(super) debug_output_tokens: bool,

    pub(super) errors: LinearResult<()>,
}

/// Used as context while parsing helper attributes of #[sabi_trait].
#[derive(Debug, Copy, Clone)]
enum ParseContext {
    TraitAttr,
    Method { index: usize },
}

/// Parses the helper attributes for `#[sabi_trait]`.
pub(crate) fn parse_attrs_for_sabi_trait<'a>(
    trait_: &'a ItemTrait,
    arenas: &'a Arenas,
    ctokens: &'a CommonTokens,
) -> Result<SabiTraitOptions<'a>, syn::Error> {
    let mut this = SabiTraitAttrs::default();

    let assoc_fns: Vec<&'a TraitItemMethod> = trait_
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Method(x) => Some(x),
            _ => None,
        })
        .collect();

    this.methods_with_attrs.reserve(assoc_fns.len());

    this.disable_inherent_default.resize(assoc_fns.len(), false);

    parse_inner(&mut this, &*trait_.attrs, ParseContext::TraitAttr, arenas)?;

    for (index, assoc_fn) in assoc_fns.iter().cloned().enumerate() {
        this.methods_with_attrs.push(MethodWithAttrs::new(assoc_fn));

        parse_inner(
            &mut this,
            &*assoc_fn.attrs,
            ParseContext::Method { index },
            arenas,
        )?;
    }

    this.errors.take()?;

    SabiTraitOptions::new(trait_, this, arenas, ctokens)
}

/// Parses all the attributes on an item.
fn parse_inner<'a, I>(
    this: &mut SabiTraitAttrs<'a>,
    attrs: I,
    pctx: ParseContext,
    arenas: &'a Arenas,
) -> Result<(), syn::Error>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    for attr in attrs {
        if attr.path.is_ident("sabi") {
            attr.parse_args_with(|input: &ParseBuffer<'_>| {
                parse_sabi_trait_attr(this, pctx, input, attr, arenas)
            })?;
        } else if attr.path.is_ident("doc")
            && matches!(pctx, ParseContext::TraitAttr)
            && syn::parse::Parser::parse2(contains_doc_hidden, attr.tokens.clone())?
        {
            this.is_hidden = true;
        } else if let ParseContext::TraitAttr = pctx {
            this.attrs.other_attrs.push(attr.clone());
        } else if let ParseContext::Method { .. } = pctx {
            this.methods_with_attrs
                .last_mut()
                .unwrap()
                .attrs
                .other_attrs
                .push(attr.clone());
        }
    }
    Ok(())
}

/// Parses the `#[sabi()]` attributes on an item.
fn parse_sabi_trait_attr<'a>(
    this: &mut SabiTraitAttrs<'a>,
    pctx: ParseContext,
    input: &ParseBuffer<'_>,
    attr: &Attribute,
    _arenas: &'a Arenas,
) -> Result<(), syn::Error> {
    fn push_attr(
        this: &mut SabiTraitAttrs<'_>,
        pctx: ParseContext,
        input: &ParseBuffer<'_>,
        attr: Attribute,
    ) {
        input.ignore_rest();
        match pctx {
            ParseContext::Method { .. } => {
                this.methods_with_attrs
                    .last_mut()
                    .unwrap()
                    .attrs
                    .derive_attrs
                    .push(attr);
            }
            ParseContext::TraitAttr => {
                this.attrs.derive_attrs.push(attr);
            }
        }
    }

    if input.check_parse(kw::no_default_fallback)? {
        match pctx {
            ParseContext::TraitAttr => {
                for is_disabled in &mut this.disable_inherent_default {
                    *is_disabled = true;
                }
            }
            ParseContext::Method { index } => {
                this.disable_inherent_default[index] = true;
            }
        }
    } else if input.check_parse(kw::debug_print_trait)? {
        this.debug_print_trait = true;
    } else if input.check_parse(kw::debug_output_tokens)? {
        this.debug_output_tokens = true;
    } else if let ParseContext::TraitAttr = pctx {
        if input.check_parse(kw::use_dyntrait)? || input.check_parse(kw::use_dyn_trait)? {
            this.which_object = WhichObject::DynTrait;
        } else if input.check_parse(kw::no_trait_impl)? {
            this.disable_trait_impl = true;
        } else {
            push_attr(this, pctx, input, attr.clone());
        }
    } else {
        push_attr(this, pctx, input, attr.clone())
    }
    Ok(())
}
