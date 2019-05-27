use syn::{
    Attribute, Ident, Meta, MetaList, 
    ItemTrait,TraitItem,TraitItemMethod,
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    *,
    attribute_parsing::with_nested_meta,
    arenas::Arenas,
};


pub(crate) struct SabiTraitOptions<'a> {
    derive_attrs:Vec<&'a Meta>,
    method_derive_attrs:Vec<Vec<&'a Meta>>,
}


impl<'a> SabiTraitOptions<'a> {
    fn new(
        _trait_: &'a ItemTrait, 
        this: SabiTraitAttrs<'a>,
        _arenas: &'a Arenas,
    ) -> Self {
        Self{
            derive_attrs:this.derive_attrs,
            method_derive_attrs:this.method_derive_attrs,
        }
    }
}


////////////////////////////////////////////////////////////////////////////////


#[derive(Default)]
struct SabiTraitAttrs<'a> {
    derive_attrs:Vec<&'a Meta>,
    method_derive_attrs:Vec<Vec<&'a Meta>>,
}


#[derive(Copy, Clone)]
enum ParseContext<'a> {
    TraitAttr{
        name:&'a Ident,
    },
    Item{
        item_index:usize,
    },
}


pub(crate) fn parse_attrs_for_sabi_trait<'a,I>(
    trait_:&'a ItemTrait,
    arenas: &'a Arenas
)->SabiTraitOptions<'a>
where
    I:IntoIterator<Item=&'a Attribute>
{
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

    this.method_derive_attrs.resize(assoc_fns.len(),Vec::new());

    parse_inner(
        &mut this,
        &*trait_.attrs,
        ParseContext::TraitAttr{name:&trait_.ident},
        arenas,
    );

    for (item_index,assoc_fn) in assoc_fns.iter().cloned().enumerate() {
        parse_inner(
            &mut this,
            &*assoc_fn.attrs,
            ParseContext::Item{item_index},
            arenas,
        )
    }


    SabiTraitOptions::new(trait_,this,arenas)
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
            _ => {}
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
    }
}


fn parse_sabi_trait_attr<'a>(
    this: &mut SabiTraitAttrs<'a>,
    pctx: ParseContext<'a>, 
    attr: Meta, 
    arenas: &'a Arenas
) {
    let attr=arenas.alloc(attr);
    match (pctx, attr) {
        (ParseContext::Item{item_index}, attr) => {
            this.method_derive_attrs[item_index].push(attr);
        }
        (ParseContext::TraitAttr{..}, attr) => {
            this.derive_attrs.push(attr);
        }
    }
}