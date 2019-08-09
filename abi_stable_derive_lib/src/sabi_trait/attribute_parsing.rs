use super::{
    *,
    TraitDefinition,
};

use std::{iter,mem};

use syn::{
    Attribute, Ident, Meta, MetaList, NestedMeta,
    ItemTrait,TraitItem,TraitItemMethod,
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    attribute_parsing::with_nested_meta,
    arenas::Arenas,
};


/// Configuration parsed from the helper attributes of `#[sabi_trait]`
pub(crate) struct SabiTraitOptions<'a> {
    /// Whether the output of the proc-macro is printed with println.
    pub(crate) debug_print_trait:bool,
    pub(crate) trait_definition:TraitDefinition<'a>,
}


impl<'a> SabiTraitOptions<'a> {
    fn new(
        trait_: &'a ItemTrait, 
        this: SabiTraitAttrs<'a>,
        arenas: &'a Arenas,
        ctokens:&'a CommonTokens,
    ) -> Self {
        Self{
            debug_print_trait:this.debug_print_trait,
            trait_definition:
                TraitDefinition::new(
                    trait_,
                    this.attrs,
                    this.methods_with_attrs,
                    this.which_object,
                    arenas,
                    ctokens,
                ),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////

/// The attributes used in the vtable,and the trait.
#[derive(Debug, Clone,Default)]
pub(crate) struct OwnedDeriveAndOtherAttrs{
    /// The attributes used in the vtable.
    pub(crate) derive_attrs:Vec<Meta>,
    /// The attributes used in the trait.
    pub(crate) other_attrs:Vec<Meta>,
}


////////////////////////////////////////////////////////////////////////////////


/// The `syn` type for methods,as well as its attributes split by where they are used.
#[derive(Debug, Clone)]
pub(crate) struct MethodWithAttrs<'a>{
    /// The attributes used in the vtable,and the trait.
    pub(crate) attrs:OwnedDeriveAndOtherAttrs,
    pub(crate) item:&'a TraitItemMethod,
}


impl<'a> MethodWithAttrs<'a>{
    /// Constructs a `MethodWithAttrs` with no attributes.
    fn new(item:&'a TraitItemMethod)->Self{
        Self{
            attrs:OwnedDeriveAndOtherAttrs{
                derive_attrs:Vec::new(),
                other_attrs:Vec::new(),
            },
            item,
        }
    }
}


////////////////////////////////////////////////////////////////////////////////


/// A datastructure used while parsing the helper attributes of #[sabi_trait].
#[derive(Default)]
struct SabiTraitAttrs<'a> {
    /// Whether the output of the proc-macro is printed with println.
    debug_print_trait:bool,
    /// The attributes used in the vtable,and the trait.
    attrs:OwnedDeriveAndOtherAttrs,
    /// The `syn` type for methods,as well as their attributes split by where they are used.
    methods_with_attrs:Vec<MethodWithAttrs<'a>>,
    /// Which type to use as the underlying implementation of the trait object,
    /// either DynTrait or RObject.
    which_object:WhichObject,
}


/// Used as context while parsing helper attributes of #[sabi_trait].
#[derive(Debug, Copy, Clone)]
enum ParseContext<'a> {
    TraitAttr{
        name:&'a Ident,
    },
    Method,
}


/// Parses the helper attributes for `#[sabi_trait]`.
pub(crate) fn parse_attrs_for_sabi_trait<'a>(
    trait_:&'a ItemTrait,
    arenas: &'a Arenas,
    ctokens:&'a CommonTokens,
)->SabiTraitOptions<'a> {
    let mut this=SabiTraitAttrs::default();

    let assoc_fns:Vec<&'a TraitItemMethod>=
        trait_.items
        .iter()
        .filter_map(|item|{
            match item {
                TraitItem::Method(x)=>Some(x),
                _=>None,
            }
        })
        .collect();

    this.methods_with_attrs.reserve(assoc_fns.len());

    parse_inner(
        &mut this,
        &*trait_.attrs,
        ParseContext::TraitAttr{name:&trait_.ident},
        arenas,
    );

    for assoc_fn in assoc_fns.iter().cloned() {
        this.methods_with_attrs.push(MethodWithAttrs::new(assoc_fn));

        parse_inner(
            &mut this,
            &*assoc_fn.attrs,
            ParseContext::Method,
            arenas,
        );

        let last_fn=this.methods_with_attrs.last_mut().unwrap();

        if !last_fn.attrs.derive_attrs.is_empty() {
            wrap_attrs_in_sabi_list(&mut last_fn.attrs.derive_attrs)
        }
    }


    if !this.attrs.derive_attrs.is_empty() {
        wrap_attrs_in_sabi_list(&mut this.attrs.derive_attrs)
    }



    SabiTraitOptions::new(trait_,this,arenas,ctokens)
}

/// Parses all the attributes on an item.
fn parse_inner<'a,I>(
    this: &mut SabiTraitAttrs<'a>,
    attrs: I,
    pctx: ParseContext<'a>,
    arenas: &'a Arenas,
) where
    I:IntoIterator<Item=&'a Attribute>
{
    for attr in attrs {
        match attr.parse_meta().unwrap() {
            Meta::List(list) => {
                parse_attr_list(this,pctx, list, arenas);
            }
            other_attr => {
                match pctx {
                    ParseContext::TraitAttr{..}=>{
                        this.attrs.other_attrs.push(other_attr);
                    }
                    ParseContext::Method=>{
                        this.methods_with_attrs.last_mut().unwrap()
                            .attrs.other_attrs
                            .push(other_attr);
                    }
                }
            }
        }
    }
}

/// Parses the list attributes on an item.
fn parse_attr_list<'a>(
    this: &mut SabiTraitAttrs<'a>,
    pctx: ParseContext<'a>,
    list: MetaList, 
    arenas: &'a Arenas
) {
    if list.ident == "sabi" {
        with_nested_meta("sabi", list.nested, |attr| {
            parse_sabi_trait_attr(this,pctx, attr, arenas)
        });
    }else if let ParseContext::Method=pctx {
        this.methods_with_attrs
            .last_mut().unwrap()
            .attrs.other_attrs
            .push(Meta::List(list));
    }
}


/// Parses the `#[sabi()]` attributes on an item.
fn parse_sabi_trait_attr<'a>(
    this: &mut SabiTraitAttrs<'a>,
    pctx: ParseContext<'a>, 
    attr: Meta, 
    _arenas: &'a Arenas
) {
    match (pctx, attr) {

        (ParseContext::Method, attr) => {
            this.methods_with_attrs
                .last_mut().unwrap()
                .attrs
                .derive_attrs
                .push(attr);
        }
        (ParseContext::TraitAttr{..}, Meta::Word(ref word))if word=="debug_print_trait" => {
            this.debug_print_trait=true;
        }
        (ParseContext::TraitAttr{..}, Meta::Word(ref word))
        if word=="use_dyntrait"||word=="use_dyn_trait" => {
            this.which_object=WhichObject::DynTrait;
        }
        (ParseContext::TraitAttr{..}, attr) => {
            this.attrs.derive_attrs.push(attr);
        }
    }
}


/// Wraps a list of Meta with `#[sabi(  )]`
fn wrap_attrs_in_sabi_list<A>(attrs:&mut A)
where
    A:Default+Extend<Meta>+IntoIterator<Item=Meta>,
{
    let older_attrs=mem::replace(attrs,Default::default());

    let list=Meta::List(MetaList{
        ident:parse_str_as_ident("sabi"),
        paren_token:Default::default(),
        nested:older_attrs.into_iter().map(NestedMeta::Meta).collect(),
    });

    attrs.extend(iter::once(list));
}
