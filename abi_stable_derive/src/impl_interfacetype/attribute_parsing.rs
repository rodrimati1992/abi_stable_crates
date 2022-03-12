use std::collections::HashMap;

use as_derive_utils::{parse_utils::ParseBufferExt, return_syn_err, syn_err};

use syn::{parse::ParseBuffer, Ident, Token};

use crate::{
    impl_interfacetype::{TraitStruct, WhichTrait, TRAIT_LIST},
    parse_utils::parse_str_as_ident,
};

/// The traits that are (un)implemented,parsed from the `#[sabi(impl_InterfaceType())]`
/// helper attribute.
pub(crate) struct ImplInterfaceType {
    pub(crate) impld: Vec<Ident>,
    pub(crate) unimpld: Vec<Ident>,
}

/// Parses the `#[sabi(impl_InterfaceType())]` helper attribute.
pub(crate) fn parse_impl_interfacetype(
    input: &ParseBuffer<'_>,
) -> Result<ImplInterfaceType, syn::Error> {
    let trait_map = TRAIT_LIST
        .iter()
        .map(|t| {
            let ident = parse_str_as_ident(t.name);
            (ident, t.which_trait)
        })
        .collect::<HashMap<Ident, WhichTrait>>();

    let mut impld_struct = TraitStruct::TRAITS.map(|_, _| false);

    let mut impld = Vec::new();
    let mut unimpld = Vec::new();

    let valid_traits = || -> String {
        trait_map
            .keys()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\n    ")
    };

    input.for_each_separated(Token!(,), |input| {
        let trait_ident = input.parse::<Ident>().map_err(|e| {
            syn_err!(
                e.span(),
                "invalid attribute inside #[sabi(impl_InterfaceType(  ))].\n\
                     Valid traits:\n    {}\n\
                    ",
                valid_traits()
            )
        })?;

        match trait_map.get(&trait_ident) {
            Some(&which_trait) => {
                impld_struct[which_trait] = true;

                match which_trait {
                    WhichTrait::Iterator | WhichTrait::DoubleEndedIterator => {
                        impld_struct.iterator = true;
                    }
                    WhichTrait::Eq | WhichTrait::PartialOrd => {
                        impld_struct.partial_eq = true;
                    }
                    WhichTrait::Ord => {
                        impld_struct.partial_eq = true;
                        impld_struct.eq = true;
                        impld_struct.partial_ord = true;
                    }
                    WhichTrait::IoBufRead => {
                        impld_struct.io_read = true;
                    }
                    WhichTrait::Error => {
                        impld_struct.display = true;
                        impld_struct.debug = true;
                    }
                    _ => {}
                }
            }
            None => return_syn_err!(
                trait_ident.span(),
                "invalid trait inside #[sabi(impl_InterfaceType(  ))]:\n\
                 Valid traits:\n    {}\n",
                valid_traits(),
            ),
        }
        Ok(())
    })?;

    for (trait_, which_trait) in trait_map {
        if impld_struct[which_trait] {
            &mut impld
        } else {
            &mut unimpld
        }
        .push(trait_);
    }

    Ok(ImplInterfaceType { impld, unimpld })
}
