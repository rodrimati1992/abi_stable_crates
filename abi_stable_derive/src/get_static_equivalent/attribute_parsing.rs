//! For parsing the helper attributess for `#[derive(GetStaticEquivalent)]`.

use std::marker::PhantomData;

use as_derive_utils::parse_utils::ParseBufferExt;

use syn::{parse::ParseBuffer, Attribute};

use crate::impl_interfacetype::{parse_impl_interfacetype, ImplInterfaceType};

/// This is derived from the helper attributes of the `#[derive(GetStaticEquivalent)]` macrp.
#[derive(Default)]
pub(super) struct GetStaticEquivAttrs<'a> {
    pub(super) impl_interfacetype: Option<ImplInterfaceType>,
    pub(super) debug_print: bool,
    _marker: PhantomData<&'a ()>,
}

mod kw {
    syn::custom_keyword! {debug_print}
    syn::custom_keyword! {sabi}
    syn::custom_keyword! {impl_InterfaceType}
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
        if attr.path.is_ident("sabi") {
            attr.parse_args_with(|input: &ParseBuffer<'_>| parse_gse_attr(&mut this, input))?;
        }
    }

    Ok(this)
}

// Helper function of `parse_attrs_for_get_static_equiv`.
fn parse_gse_attr(
    this: &mut GetStaticEquivAttrs<'_>,
    input: &ParseBuffer<'_>,
) -> Result<(), syn::Error> {
    if input.check_parse(kw::impl_InterfaceType)? {
        let content = input.parse_paren_buffer()?;
        this.impl_interfacetype = Some(parse_impl_interfacetype(&content)?);
    } else if input.check_parse(kw::debug_print)? {
        this.debug_print = true;
    } else {
        return Err(input.error("Unrecodnized #[sabi(..)] attribute."));
    }

    Ok(())
}
