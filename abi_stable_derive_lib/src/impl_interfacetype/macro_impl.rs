use super::*;

use proc_macro2::TokenStream as TokenStream2;

use syn::{
    ItemImpl,
    ImplItem,
    ImplItemType,
    Visibility,
};


/// The implementation of the impl_InterfaceType!{}` proc macro.
///
/// This macro takes in an impl block for the InterfaceType trait,
/// and emulates defaulted associated types for the ones that weren't mentioned.
pub fn the_macro(mut impl_:ItemImpl)->TokenStream2{
    let interfacetype:syn::Ident=syn::parse_str("InterfaceType").unwrap();

    let mut const_name=(&impl_.self_ty).into_token_stream().to_string();
    const_name.retain(|c| c.is_alphanumeric() );
    const_name.insert_str(0,"_impl_InterfaceType");
    let const_name=parse_str_as_ident(&const_name);
    
    let interface_path_s=impl_.trait_.as_ref().map(|x| &x.1.segments );
    let is_interface_type=interface_path_s
        .and_then(|x| x.last() )
        .map_or(false,|path_| path_.value().ident==interfacetype );

    if !is_interface_type {
        panic!(
            "expected 'impl<...> InterfaceType for {} ' ",
            (&impl_.self_ty).into_token_stream()
        );
    }
    
    // The default value for each associated type.
    let mut default_map=TRAIT_LIST
        .iter()
        .map(|ut|{
            ( parse_str_as_ident(ut.name) , DefaultVal::from(ut.which_trait.default_value()) ) 
        })
        .collect::<HashMap<_,_>>();

    // Processed the items in the impl block,
    // removing them from the defaulted associated type map,
    // and converting the value of the associated type to 
    // either `Implemented<trait_marker::AssocTyName>`
    // or `Unimplemented<trait_marker::AssocTyName>` 
    for item in &mut impl_.items {
        match item {
            ImplItem::Type(assoc_ty)=>{
                assert_ne!(
                    assoc_ty.ident,
                    "define_this_in_the_impl_InterfaceType_macro",
                    "you are not supposed to define\n\t\
                     the 'define_this_in_the_impl_InterfaceType_macro' associated type yourself"
                );
                default_map.remove(&assoc_ty.ident);

                let old_ty=&assoc_ty.ty;
                let name=&assoc_ty.ident;
                let span=name.span();

                assoc_ty.ty=type_from_token_stream(
                    quote_spanned!(span=> ImplFrom<#old_ty, trait_marker::#name> )
                );
            }
            _=>{}
        }
    }

    default_map.insert(private_associated_type(),DefaultVal::Hidden);

    // Converts the defaulted associated types to the syn datastructure,
    // and then adds them to the list of items inside the impl block.
    for (key,default_) in default_map {
        let mut attrs=Vec::<syn::Attribute>::new();

        let span=key.span();

        let ty=match default_ {
            DefaultVal::Unimplemented=>quote_spanned!(span=> Unimplemented<trait_marker::#key> ),
            DefaultVal::Implemented=>quote_spanned!(span=> Implemented<trait_marker::#key> ),
            DefaultVal::Hidden=>{
                attrs.extend(parse_syn_attributes("#[doc(hidden)]"));
                quote_spanned!(span=> () )
            },
        }.piped(type_from_token_stream);

        let defaulted=ImplItemType{
            attrs,
            vis: Visibility::Inherited,
            defaultness: None,
            type_token: Default::default(),
            ident:key,
            generics: Default::default(),
            eq_token: Default::default(),
            ty,
            semi_token: Default::default(),
        };
        impl_.items.push(ImplItem::Type(defaulted))
    }

    quote!(
        const #const_name:()={
            use ::abi_stable::derive_macro_reexports::{
                Implemented,
                Unimplemented,
                ImplFrom,
                trait_marker,
            };

            #impl_
        };
    )
}


fn type_from_token_stream(tts:TokenStream2)->syn::Type{
    let x=syn::TypeVerbatim{tts};
    syn::Type::Verbatim(x)
}



/// Parses an inner attribute `#[]` from a string.
///
/// inner attribute as opposed to an outer attribute `#![]`.
pub fn parse_syn_attributes(str_: &str) -> Vec<syn::Attribute> {
    syn::parse_str::<ParseOuter>(str_).unwrap().attributes
}


struct ParseOuter {
    attributes: Vec<syn::Attribute>,
}

impl syn::parse::Parse for ParseOuter {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        Ok(Self {
            attributes: syn::Attribute::parse_outer(input)?,
        })
    }
}