use std::marker::PhantomData;

use quote::ToTokens;

use syn::{
    Attribute, Meta, MetaList,
};



use crate::{
    attribute_parsing::{with_nested_meta},
    impl_interfacetype::{ImplInterfaceType,parse_impl_interfacetype},
};



#[derive(Default)]
pub(super) struct GetStaticEquivAttrs<'a>{
    pub(super) impl_interfacetype:Option<ImplInterfaceType>,
    pub(super) debug_print:bool,
    _marker:PhantomData<&'a ()>,
}


pub(super) fn parse_attrs_for_get_static_equiv<'a,I>(
    attrs: I,
) -> GetStaticEquivAttrs<'a>
where
    I:IntoIterator<Item=&'a Attribute>
{
    let mut this=GetStaticEquivAttrs::default();

    for attr in attrs {
        match attr.parse_meta().unwrap() {
            Meta::List(list) => {
                parse_attr_list(&mut this,list);
            }
            _ => {}
        }
    }

    this
}

fn parse_attr_list<'a>(
    this: &mut GetStaticEquivAttrs<'a>,
    list: MetaList, 
) {
    if list.ident == "sabi" {
        with_nested_meta("sabi", list.nested, |attr| {
            parse_gse_attr(this, attr)
        });
    }
}

fn parse_gse_attr<'a>(
    this: &mut GetStaticEquivAttrs<'a>,
    attr: Meta, 
) {
    match attr {
        Meta::List(list)=>{
            if list.ident == "impl_InterfaceType" {
                this.impl_interfacetype=Some(parse_impl_interfacetype(&list.nested));
            }else{
                panic!(
                    "Unrecodnized #[sabi(..)] attribute:\n{}",
                    list.into_token_stream(),
                );
            }
        }
        Meta::Word(ref word) if word=="debug_print"=>{
            this.debug_print=true;
        }
        x =>{
            panic!(
                "not allowed inside the #[sabi(...)] attribute:\n{}",
                x.into_token_stream()
            );
        }
    }
}


