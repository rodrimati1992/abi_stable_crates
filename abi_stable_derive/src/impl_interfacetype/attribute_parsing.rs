use std::{
    collections::HashMap,
};

use syn::{
    Meta, NestedMeta, Ident, 
    punctuated::Punctuated,
    token::Comma,
};

use crate::{
    impl_interfacetype::{TRAIT_LIST,TraitStruct,WhichTrait},
    parse_utils::parse_str_as_ident,
};


/// The traits that are (un)implemented,parsed from the `#[sabi(impl_InterfaceType())]`
/// helper attribute.
pub(crate) struct ImplInterfaceType{
    pub(crate) impld  :Vec<Ident>,
    pub(crate) unimpld:Vec<Ident>,
}

/// Parses the `#[sabi(impl_InterfaceType())]` helper attribute.
pub(crate) fn parse_impl_interfacetype<'a>(
    list: &Punctuated<NestedMeta, Comma>
)-> Result<ImplInterfaceType,syn::Error> {
    let trait_map=TRAIT_LIST.iter()
        .map(|t|{
            let ident=parse_str_as_ident(t.name);
            (ident,t.which_trait)
        })
        .collect::<HashMap<Ident,WhichTrait>>();

    let mut impld_struct=TraitStruct::TRAITS.map(|_,_|false);

    let mut impld  =Vec::new();
    let mut unimpld=Vec::new();

    let valid_traits=||->String{
        trait_map.keys().map(|x|x.to_string()).collect::<Vec<String>>().join("\n    ")
    };

    for subelem in list {
        let trait_ident=match subelem {
            NestedMeta::Meta(Meta::Path(ident))=>ident.get_ident(),
            _ => None,
        }.ok_or_else(||{
            spanned_err!(
                subelem,
                "invalid attribute inside #[sabi(impl_InterfaceType(  ))].\n\
                 Valid traits:\n    {}\n\
                ",
                valid_traits()
            )
        })?;

        match trait_map.get(trait_ident) {
            Some(&which_trait) => {
                impld_struct[which_trait]=true;

                match which_trait {
                    WhichTrait::Iterator|WhichTrait::DoubleEndedIterator=>{
                        impld_struct.iterator=true;
                    }
                    WhichTrait::Eq|WhichTrait::PartialOrd=>{
                        impld_struct.partial_eq=true;
                    }
                    WhichTrait::Ord=>{
                        impld_struct.partial_eq=true;
                        impld_struct.eq=true;
                        impld_struct.partial_ord=true;
                    }
                    WhichTrait::IoBufRead=>{
                        impld_struct.io_read=true;
                    }
                    WhichTrait::Error=>{
                        impld_struct.display=true;
                        impld_struct.debug=true;
                    }
                    _=>{}
                }
            },
            None =>return_spanned_err!(
                trait_ident,
                "invalid trait inside #[sabi(impl_InterfaceType(  ))]:\n\
                 Valid traits:\n    {}\n", 
                valid_traits(),
            ),
        }
    }

    for (trait_,which_trait) in trait_map {
        if impld_struct[which_trait] {
            &mut impld
        }else{
            &mut unimpld
        }.push(trait_);
    }

    Ok(ImplInterfaceType{
        impld,
        unimpld,
    })
}
