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


pub(crate) struct SabiTraitOptions<'a> {
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
                    arenas,
                    ctokens,
                ),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub(crate) struct DeriveAndOtherAttrs<'a>{
    pub(crate) derive_attrs:&'a [Meta],
    pub(crate) other_attrs:&'a [Meta],
}


impl<'a> DeriveAndOtherAttrs<'a>{
    fn new(owned:OwnedDeriveAndOtherAttrs,arenas:&'a Arenas)->Self{
        Self{
            derive_attrs:arenas.alloc(owned.derive_attrs),
            other_attrs:arenas.alloc(owned.other_attrs),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////


#[derive(Debug, Clone,Default)]
pub(crate) struct OwnedDeriveAndOtherAttrs{
    pub(crate) derive_attrs:Vec<Meta>,
    pub(crate) other_attrs:Vec<Meta>,
}


////////////////////////////////////////////////////////////////////////////////



#[derive(Debug, Clone)]
pub(crate) struct MethodWithAttrs<'a>{
    pub(crate) attrs:OwnedDeriveAndOtherAttrs,
    pub(crate) item:&'a TraitItemMethod,
}


impl<'a> MethodWithAttrs<'a>{
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


#[derive(Default)]
struct SabiTraitAttrs<'a> {
    debug_print_trait:bool,
    attrs:OwnedDeriveAndOtherAttrs,
    methods_with_attrs:Vec<MethodWithAttrs<'a>>,
}


#[derive(Debug, Copy, Clone)]
enum ParseContext<'a> {
    TraitAttr{
        name:&'a Ident,
    },
    Method,
}


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
        (ParseContext::TraitAttr{..}, attr) => {
            this.attrs.derive_attrs.push(attr);
        }
    }
}


fn wrap_attrs_in_sabi_list<A>(mut attrs:&mut A)
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
