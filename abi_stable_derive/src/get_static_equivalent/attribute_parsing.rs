/*!
For parsing the helper attributess for `#[derive(GetStaticEquivalent)]`.
*/

use std::marker::PhantomData;

use as_derive_utils::return_spanned_err;

use syn::{Attribute, Meta, MetaList};

use crate::{
    attribute_parsing::with_nested_meta,
    impl_interfacetype::{parse_impl_interfacetype, ImplInterfaceType},
    utils::SynPathExt,
};

/// This is derived from the helper attributes of the `#[derive(GetStaticEquivalent)]` macrp.
#[derive(Default)]
pub(super) struct GetStaticEquivAttrs<'a> {
    pub(super) impl_interfacetype: Option<ImplInterfaceType>,
    pub(super) debug_print: bool,
    _marker: PhantomData<&'a ()>,
}

/// Parses the helper attributes of the `#[derive(GetStaticEquivalent)]` macrp.
pub(super) fn parse_attrs_for_get_static_equiv<'a, I>(
    attrs: I,
) -> Result<GetStaticEquivAttrs<'a>, syn::Error>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    let mut this = GetStaticEquivAttrs::default();

    for attr in attrs {
        if let Meta::List(list) = attr.parse_meta()? {
            parse_attr_list(&mut this, list)?;
        }
    }

    Ok(this)
}

// Helper function of `parse_attrs_for_get_static_equiv`.
fn parse_attr_list(this: &mut GetStaticEquivAttrs<'_>, list: MetaList) -> Result<(), syn::Error> {
    if list.path.equals_str("sabi") {
        with_nested_meta("sabi", list.nested, |attr| parse_gse_attr(this, attr))?;
    }
    Ok(())
}

// Helper function of `parse_attrs_for_get_static_equiv`.
fn parse_gse_attr(this: &mut GetStaticEquivAttrs<'_>, attr: Meta) -> Result<(), syn::Error> {
    match attr {
        Meta::List(list) => {
            if list.path.equals_str("impl_InterfaceType") {
                this.impl_interfacetype = Some(parse_impl_interfacetype(&list.nested)?);
            } else {
                return_spanned_err!(list, "Unrecodnized #[sabi(..)] attribute.");
            }
        }
        Meta::Path(ref word) if word.equals_str("debug_print") => {
            this.debug_print = true;
        }
        x => return_spanned_err!(x, "Unrecodnized #[sabi(..)] attribute."),
    }
    Ok(())
}
