//! Stuff related to the `GetStaticEquivalent` derive macro.

use proc_macro2::{Span, TokenStream as TokenStream2};

use quote::{quote, ToTokens};

use syn::{punctuated::Punctuated, DeriveInput, Generics, Ident};

use as_derive_utils::{
    gen_params_in::{GenParamsIn, InWhat},
    to_stream,
    to_token_fn::ToTokenFnMut,
};

use crate::{impl_interfacetype::impl_interfacetype_tokenizer, parse_utils::parse_str_as_ident};

mod attribute_parsing;

/// The implementation of the `GetStaticEquivalent` derive macro.
pub(crate) fn derive(data: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let name = &data.ident;
    let generics = &data.generics;
    let config = self::attribute_parsing::parse_attrs_for_get_static_equiv(&data.attrs)?;

    let impl_it = impl_interfacetype_tokenizer(name, generics, config.impl_interfacetype.as_ref());
    let gse_equiv_impl = get_static_equiv_tokenizer(name, generics, quote!());

    let ret = quote!(
        #impl_it

        #gse_equiv_impl
    );

    if config.debug_print {
        panic!("\n\n\n{}\n\n\n", ret);
    }

    Ok(ret)
}

/// Tokenizes the `GetStaticEquivalent_` implementation for some type.
fn get_static_equiv_tokenizer<'a>(
    name: &'a Ident,
    generics: &'a Generics,
    extra_bounds: TokenStream2,
) -> impl ToTokens + 'a {
    ToTokenFnMut::new(move |ts| {
        let lifetimes = &generics
            .lifetimes()
            .map(|x| &x.lifetime)
            .collect::<Vec<_>>();
        let type_params = &generics.type_params().map(|x| &x.ident).collect::<Vec<_>>();
        let const_params = &generics
            .const_params()
            .map(|x| &x.ident)
            .collect::<Vec<_>>();

        let ct_gt = syn::token::Gt::default();
        let ct_lt = syn::token::Lt::default();
        let ct_static_equivalent = parse_str_as_ident("__GetStaticEquivalent");
        let ct_comma = syn::token::Comma::default();
        let ct_static_lt = syn::parse_str::<syn::Lifetime>("'static").expect("BUG");

        let lifetimes_s = lifetimes.iter().map(|_| &ct_static_lt);
        let type_params_s = ToTokenFnMut::new(|ts| {
            for ty in type_params {
                to_stream!(ts; ct_static_equivalent, ct_lt, ty, ct_gt, ct_comma);
            }
        });
        let const_params_s = &const_params;

        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let ty_generics = GenParamsIn::new(generics, InWhat::ItemUse);

        let static_struct_name = Ident::new(&format!("_static_{}", name), Span::call_site());

        let empty_preds = Punctuated::new();
        let where_preds = where_clause
            .as_ref()
            .map_or(&empty_preds, |x| &x.predicates)
            .into_iter();

        let lifetimes_a = lifetimes;
        let type_params_a = type_params;
        let const_param_name = generics.const_params().map(|c| &c.ident);
        let const_param_type = generics.const_params().map(|c| &c.ty);

        let const_ident =
            parse_str_as_ident(&format!("_impl_get_static_equivalent_constant_{}", name,));

        quote!(
            const #const_ident:()={
                use ::abi_stable::derive_macro_reexports::renamed::{
                    __GetStaticEquivalent_,
                    __GetStaticEquivalent,
                };

                pub struct #static_struct_name<
                    #(#lifetimes_a,)*
                    #(#type_params_a:?Sized,)*
                    #(const #const_param_name:#const_param_type,)*
                >(
                    #(& #lifetimes_a (),)*
                    extern "C" fn(#(&#type_params_a,)*)
                );

                unsafe impl #impl_generics  __GetStaticEquivalent_ for #name <#ty_generics>
                where
                    #(#where_preds,)*
                    #(#type_params_a:__GetStaticEquivalent_,)*
                    #extra_bounds
                {
                    type StaticEquivalent=#static_struct_name <
                        #(#lifetimes_s,)*
                        #type_params_s
                        #({#const_params_s}),*
                    >;
                }
            };
        )
        .to_tokens(ts);
    })
}
