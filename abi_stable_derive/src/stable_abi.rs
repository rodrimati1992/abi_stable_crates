use crate::*;

use crate::{
    composite_collections::{SmallCompositeVec as CompositeVec, SmallStartLen as StartLen},
    impl_interfacetype::impl_interfacetype_tokenizer,
    lifetimes::LifetimeIndex,
    literals_constructors::{rslice_tokenizer, rstr_tokenizer},
};

use as_derive_utils::{
    datastructure::{DataStructure, DataVariant},
    gen_params_in::{GenParamsIn, InWhat},
    return_spanned_err, return_syn_err, to_stream,
    to_token_fn::ToTokenFnMut,
};

use syn::Ident;

use proc_macro2::{Span, TokenStream as TokenStream2};

use core_extensions::{IteratorExt, SelfOps};

#[doc(hidden)]
pub mod reflection;

mod attribute_parsing;

mod common_tokens;

mod generic_params;

mod nonexhaustive;

mod prefix_types;

mod repr_attrs;

mod tl_function;

mod tl_field;

mod tl_multi_tl;

mod shared_vars;

#[cfg(test)]
mod tests;

use self::{
    attribute_parsing::{
        parse_attrs_for_stable_abi, ASTypeParamBound, ConstIdents, LayoutConstructor,
        StabilityKind, StableAbiOptions,
    },
    common_tokens::CommonTokens,
    nonexhaustive::{tokenize_enum_info, tokenize_nonexhaustive_items},
    prefix_types::{prefix_type_tokenizer, PrefixTypeTokens},
    reflection::ModReflMode,
    shared_vars::SharedVars,
    tl_field::CompTLField,
    tl_function::{CompTLFunction, VisitedFieldMap},
    tl_multi_tl::TypeLayoutRange,
};

pub(crate) fn derive(mut data: DeriveInput) -> Result<TokenStream2, syn::Error> {
    data.generics.make_where_clause();

    // println!("\nderiving for {}",data.ident);

    let arenas = Arenas::default();
    let arenas = &arenas;
    let ctokens = CommonTokens::new(arenas);
    let ctokens = &ctokens;
    let ds = &DataStructure::new(&data);
    let config = &parse_attrs_for_stable_abi(ds.attrs, ds, arenas)?;
    let shared_vars = &mut SharedVars::new(arenas, &config.const_idents, ctokens);
    let generics = ds.generics;
    let name = ds.name;

    let doc_hidden_attr = config.doc_hidden_attr;

    // This has to come before the `VisitedFieldMap`.
    let generic_params_tokens =
        generic_params::GenericParams::new(ds, shared_vars, config, ctokens);

    if generics.lifetimes().count() > LifetimeIndex::MAX_LIFETIME_PARAM + 1 {
        return_syn_err!(
            Span::call_site(),
            "Cannot have more than {} lifetime parameter.",
            LifetimeIndex::MAX_LIFETIME_PARAM + 1
        );
    }

    let visited_fields = &VisitedFieldMap::new(ds, config, shared_vars, ctokens);
    shared_vars.extract_errs()?;

    let mono_type_layout = &Ident::new(&format!("_MONO_LAYOUT_{}", name), Span::call_site());

    let (_, _, where_clause) = generics.split_for_impl();
    let where_clause = (&where_clause.expect("BUG").predicates).into_iter();
    let where_clause_b = where_clause.clone();

    let ty_generics = GenParamsIn::new(generics, InWhat::ItemUse);

    let impld_stable_abi_trait = match &config.kind {
        StabilityKind::Value {
            impl_prefix_stable_abi: false,
        }
        | StabilityKind::NonExhaustive { .. } => Ident::new("StableAbi", Span::call_site()),
        StabilityKind::Value {
            impl_prefix_stable_abi: true,
        }
        | StabilityKind::Prefix { .. } => Ident::new("PrefixStableAbi", Span::call_site()),
    };

    // The type that implements StableAbi
    let impl_ty = match &config.kind {
        StabilityKind::Value { .. } => quote!(#name <#ty_generics> ),
        StabilityKind::Prefix(prefix) => {
            let n = &prefix.prefix_fields_struct;
            quote!(#n <#ty_generics> )
        }
        StabilityKind::NonExhaustive(nonexhaustive) => {
            let marker = nonexhaustive.nonexhaustive_marker;
            quote!(#marker < #name  <#ty_generics> , __Storage > )
        }
    };

    let mut prefix_type_trait_bound = None;
    let mut prefix_bounds: &[_] = &[];

    // The type whose size and alignment that is stored in the type layout.
    let size_align_for = match &config.kind {
        StabilityKind::NonExhaustive(_) => {
            quote!(__Storage)
        }
        StabilityKind::Prefix(prefix) => {
            let prefix_fields_struct = prefix.prefix_fields_struct;

            prefix_type_trait_bound = Some(quote!(
                #name <#ty_generics>:__sabi_re::PrefixTypeTrait,
            ));
            prefix_bounds = &prefix.prefix_bounds;

            quote!( #prefix_fields_struct <#ty_generics> )
        }
        StabilityKind::Value { .. } => quote!(Self),
    };

    let repr = config.repr;

    let is_transparent = config.repr.is_repr_transparent();
    let is_enum = ds.data_variant == DataVariant::Enum;
    let prefix = match &config.kind {
        StabilityKind::Prefix(prefix) => Some(prefix),
        _ => None,
    };
    let nonexh_opt = match &config.kind {
        StabilityKind::NonExhaustive(nonexhaustive) => Some(nonexhaustive),
        _ => None,
    };

    let tags_const;
    let tags_arg;
    // tokenizes the `Tag` data structure associated with this type.
    match &config.tags {
        Some(tag) => {
            tags_const = quote!( const __SABI_TAG: &'static __sabi_re::Tag = &#tag; );
            tags_arg = quote!(Some(Self::__SABI_TAG));
        }
        None => {
            tags_const = TokenStream2::new();
            tags_arg = quote!(None);
        }
    }

    let extra_checks_const;
    let extra_checks_arg;
    match &config.extra_checks {
        Some(extra_checks) => {
            extra_checks_const = quote!(
                const __SABI_EXTRA_CHECKS:
                    &'static ::std::mem::ManuallyDrop<__sabi_re::StoredExtraChecks>
                =
                    &std::mem::ManuallyDrop::new(
                        __sabi_re::StoredExtraChecks::from_const(
                            &#extra_checks,
                            __sabi_re::TD_Opaque,
                        )
                    );
            );

            extra_checks_arg = quote!(Some(Self::__SABI_EXTRA_CHECKS));
        }
        None => {
            extra_checks_const = TokenStream2::new();
            extra_checks_arg = quote!(None);
        }
    };

    let variant_names_start_len = if is_enum {
        let mut variant_names = String::new();
        for variant in &ds.variants {
            use std::fmt::Write;
            let _ = write!(variant_names, "{};", variant.name);
        }
        shared_vars.push_str(&variant_names, None)
    } else {
        StartLen::EMPTY
    };

    // tokenizes the items for nonexhaustive enums outside of the module this generates.
    let nonexhaustive_items = tokenize_nonexhaustive_items(ds, config, ctokens);

    // tokenizes the items for nonexhaustive enums inside of the module this generates.
    let nonexhaustive_tokens = tokenize_enum_info(ds, variant_names_start_len, config, ctokens)?;

    let is_nonzero = if is_transparent && !visited_fields.map.is_empty() {
        let visited_field = &visited_fields.map[0];

        let is_opaque_field = visited_field.layout_ctor.is_opaque();
        if visited_field.comp_field.is_function() {
            quote!(__sabi_re::True)
        } else if is_opaque_field {
            quote!(__sabi_re::False)
        } else {
            let ty = visited_field.comp_field.type_(shared_vars);
            quote!( <#ty as __StableAbi>::IsNonZeroType )
        }
    } else {
        quote!(__sabi_re::False)
    };

    let ct = ctokens;
    // The tokens for the MonoTLData stored in the TypeLayout
    let mono_tl_data;
    // The tokens for the GenericTLData stored in the TypeLayout
    let generic_tl_data;

    match (is_enum, prefix) {
        (false, None) => {
            mono_tl_data = {
                let fields = fields_tokenizer(ds, visited_fields, ct);
                match ds.data_variant {
                    DataVariant::Struct => quote!( __sabi_re::MonoTLData::derive_struct(#fields) ),
                    DataVariant::Union => quote!( __sabi_re::MonoTLData::derive_union(#fields) ),
                    DataVariant::Enum => unreachable!(),
                }
            };
            generic_tl_data = {
                match ds.data_variant {
                    DataVariant::Struct => quote!(__sabi_re::GenericTLData::Struct),
                    DataVariant::Union => quote!(__sabi_re::GenericTLData::Union),
                    DataVariant::Enum => unreachable!(),
                }
            };
        }
        (true, None) => {
            let vn_sl = variant_names_start_len;
            mono_tl_data = {
                let mono_enum_tokenizer =
                    tokenize_mono_enum(ds, vn_sl, nonexh_opt, config, visited_fields, shared_vars);
                quote!( __sabi_re::MonoTLData::Enum(#mono_enum_tokenizer) )
            };
            generic_tl_data = {
                let generic_enum_tokenizer =
                    tokenize_generic_enum(ds, vn_sl, nonexh_opt, config, visited_fields, ct);
                quote!( __sabi_re::GenericTLData::Enum(#generic_enum_tokenizer) )
            };
        }
        (false, Some(prefix)) => {
            if is_transparent {
                return_spanned_err!(name, "repr(transparent) prefix types not supported")
            }

            mono_tl_data = {
                let first_suffix_field = prefix.first_suffix_field.field_pos;
                let fields = fields_tokenizer(ds, visited_fields, ct);
                let prefix_field_conditionality_mask = prefix.prefix_field_conditionality_mask;
                quote!(
                    __sabi_re::MonoTLData::prefix_type_derive(
                        #first_suffix_field,
                        #prefix_field_conditionality_mask,
                        #fields
                    )
                )
            };
            generic_tl_data = {
                quote!(
                    __sabi_re::GenericTLData::prefix_type_derive(
                        <#name <#ty_generics> as
                            __sabi_re::PrefixTypeTrait
                        >::PT_FIELD_ACCESSIBILITY,
                    )
                )
            };
        }
        (true, Some(_)) => {
            return_spanned_err!(name, "enum prefix types not supported");
        }
    };

    let lifetimes = &generics
        .lifetimes()
        .map(|x| &x.lifetime)
        .collect::<Vec<_>>();
    let type_params = &generics.type_params().map(|x| &x.ident).collect::<Vec<_>>();
    let const_params = &generics
        .const_params()
        .map(|x| &x.ident)
        .collect::<Vec<_>>();

    // For `type StaticEquivalent= ... ;`
    let lifetimes_s = lifetimes.iter().map(|_| &ctokens.static_lt);
    let type_params_s = ToTokenFnMut::new(|ts| {
        let ct = ctokens;

        for (ty_param, bounds) in config.type_param_bounds.iter() {
            match bounds {
                ASTypeParamBound::NoBound => {
                    ct.empty_tuple.to_tokens(ts);
                }
                ASTypeParamBound::GetStaticEquivalent | ASTypeParamBound::StableAbi => {
                    to_stream!(ts; ct.static_equivalent, ct.lt, ty_param, ct.gt);
                }
            }
            ct.comma.to_tokens(ts);
        }
    });
    let const_params_s = &const_params;

    // The name of the struct this generates,
    // to use as the `GetStaticEquivalent_::StaticEquivalent` associated type.
    let static_struct_name = Ident::new(&format!("_static_{}", name), Span::call_site());

    let item_info_const = Ident::new(&format!("_item_info_const_{}", name), Span::call_site());

    let static_struct_decl = {
        let const_param_name = generics.const_params().map(|c| &c.ident);
        let const_param_type = generics.const_params().map(|c| &c.ty);

        let lifetimes_a = lifetimes;

        let type_params_a = type_params;

        quote! {
            #doc_hidden_attr
            pub struct #static_struct_name<
                #(#lifetimes_a,)*
                #(#type_params_a:?Sized,)*
                #(const #const_param_name:#const_param_type,)*
            >(
                #(& #lifetimes_a (),)*
                extern "C" fn(#(&#type_params_a,)*)
            );
        }
    };

    // if the `#[sabi(impl_InterfaceType())]` attribute was used:
    // tokenizes the implementation of `InterfaceType` for `#name #ty_params`
    let interfacetype_tokenizer =
        impl_interfacetype_tokenizer(ds.name, ds.generics, config.impl_interfacetype.as_ref());

    let stringified_name = rstr_tokenizer(name.to_string());

    let mut stable_abi_bounded = Vec::new();
    let mut static_equiv_bounded = Vec::new();

    for (ident, bounds) in config.type_param_bounds.iter() {
        let list = match bounds {
            ASTypeParamBound::NoBound => None,
            ASTypeParamBound::GetStaticEquivalent => Some(&mut static_equiv_bounded),
            ASTypeParamBound::StableAbi => Some(&mut stable_abi_bounded),
        };
        if let Some(list) = list {
            list.push(ident);
        }
    }

    let stable_abi_bounded = &stable_abi_bounded;
    let static_equiv_bounded = &static_equiv_bounded;

    let extra_bounds = &config.extra_bounds;

    let PrefixTypeTokens {
        prefixref_types,
        prefixref_impls,
    } = prefix_type_tokenizer(mono_type_layout, ds, config, ctokens)?;

    let mod_refl_mode = match config.mod_refl_mode {
        ModReflMode::Module => quote!(__ModReflMode::Module),
        ModReflMode::Opaque => quote!(__ModReflMode::Opaque),
        ModReflMode::DelegateDeref(field_index) => {
            quote!(
                __ModReflMode::DelegateDeref{
                    phantom_field_index:#field_index
                }
            )
        }
    };

    let phantom_field_tys = config.phantom_fields.iter().map(|x| x.1);

    // This has to be collected into a Vec ahead of time,
    // so that the names and types are stored in SharedVars.
    let phantom_fields = config
        .phantom_fields
        .iter()
        .map(|(name, ty)| {
            CompTLField::from_expanded_std_field(
                name,
                std::iter::empty(),
                shared_vars.push_type(LayoutConstructor::Regular, ty),
                shared_vars,
            )
        })
        .collect::<Vec<CompTLField>>();
    let phantom_fields = rslice_tokenizer(&phantom_fields);

    // The storage type parameter that is added if this is a nonexhaustive enum.
    let storage_opt = nonexh_opt.map(|_| &ctokens.und_storage);
    let generics_header =
        GenParamsIn::with_after_types(ds.generics, InWhat::ImplHeader, storage_opt);

    shared_vars.extract_errs()?;

    let mono_shared_vars_tokenizer = shared_vars.mono_shared_vars_tokenizer();

    let strings_const = &config.const_idents.strings;
    let strings = shared_vars.strings().piped(rstr_tokenizer);

    let shared_vars_tokenizer = shared_vars.shared_vars_tokenizer(mono_type_layout);

    // drop(_measure_time0);

    let shared_where_preds = quote!(
        #(#where_clause_b,)*
        #(#stable_abi_bounded:__StableAbi,)*
        #(#static_equiv_bounded:__GetStaticEquivalent_,)*
        #(#extra_bounds,)*
        #(#prefix_bounds,)*
        #prefix_type_trait_bound
    );

    let stable_abi_where_preds = shared_where_preds.clone().mutated(|ts| {
        ts.extend(quote!(
            #(#phantom_field_tys:__StableAbi,)*
        ))
    });

    let prefix_ref_impls = if let StabilityKind::Prefix(prefix) = &config.kind {
        let prefix_ref = &prefix.prefix_ref;
        let prefix_fields_struct = &prefix.prefix_fields_struct;
        let lifetimes_s = lifetimes_s.clone();

        quote!(
            unsafe impl<#generics_header> __sabi_re::GetStaticEquivalent_
            for #prefix_ref <#ty_generics>
            where
                #shared_where_preds
            {
                type StaticEquivalent =
                    __sabi_re::PrefixRef<
                        #static_struct_name <
                            #(#lifetimes_s,)*
                            #type_params_s
                            #({#const_params_s}),*
                        >
                    >;
            }

            unsafe impl<#generics_header> __sabi_re::StableAbi for #prefix_ref <#ty_generics>
            where
                #stable_abi_where_preds
            {
                type IsNonZeroType = __sabi_re::True;

                const LAYOUT: &'static __sabi_re::TypeLayout =
                    <__sabi_re::PrefixRef<#prefix_fields_struct <#ty_generics>>
                        as __sabi_re::StableAbi
                    >::LAYOUT;
            }
        )
    } else {
        TokenStream2::new()
    };

    quote!(
        #prefixref_types

        #nonexhaustive_items

        const _: () = {
            use ::abi_stable;

            #[allow(unused_imports)]
            use ::abi_stable::pmr::{
                self as __sabi_re,
                renamed::*,
            };

            #prefixref_impls

            const #item_info_const: abi_stable::type_layout::ItemInfo=
                abi_stable::make_item_info!();

            const #strings_const: ::abi_stable::std_types::RStr<'static>=#strings;


            #static_struct_decl

            #nonexhaustive_tokens

            #interfacetype_tokenizer

            #prefix_ref_impls

            unsafe impl <#generics_header> __GetStaticEquivalent_ for #impl_ty
            where
                #shared_where_preds
            {
                type StaticEquivalent=#static_struct_name <
                    #(#lifetimes_s,)*
                    #type_params_s
                    #({#const_params_s}),*
                >;
            }

            #[doc(hidden)]
            const #mono_type_layout:&'static __sabi_re::MonoTypeLayout=
                &__sabi_re::MonoTypeLayout::from_derive(
                    __sabi_re::_private_MonoTypeLayoutDerive{
                        name: #stringified_name,
                        item_info: #item_info_const,
                        data: #mono_tl_data,
                        generics: #generic_params_tokens,
                        mod_refl_mode:#mod_refl_mode,
                        repr_attr:#repr,
                        phantom_fields:#phantom_fields,
                        shared_vars: #mono_shared_vars_tokenizer,
                    }
                );

            impl <#generics_header> #impl_ty
            where
                #stable_abi_where_preds
            {
                #shared_vars_tokenizer

                #extra_checks_const

                #tags_const
            }

            unsafe impl <#generics_header> __sabi_re::#impld_stable_abi_trait for #impl_ty
            where
                #stable_abi_where_preds

            {
                type IsNonZeroType=#is_nonzero;

                const LAYOUT: &'static __sabi_re::TypeLayout = {
                    &__sabi_re::TypeLayout::from_derive::<#size_align_for>(
                        __sabi_re::_private_TypeLayoutDerive {
                            shared_vars: Self::__SABI_SHARED_VARS,
                            mono:#mono_type_layout,
                            abi_consts: Self::ABI_CONSTS,
                            data:#generic_tl_data,
                            tag: #tags_arg,
                            extra_checks: #extra_checks_arg,
                        }
                    )
                };
            }

        };
    )
    .observe(|tokens| {
        // drop(_measure_time1);
        if config.debug_print {
            panic!("\n\n\n{}\n\n\n", tokens);
        }
    })
    .piped(Ok)
}

// Tokenizes a `MonoTLEnum{ .. }`
fn tokenize_mono_enum<'a>(
    ds: &'a DataStructure<'a>,
    variant_names_start_len: StartLen,
    _nonexhaustive_opt: Option<&'a nonexhaustive::NonExhaustive<'a>>,
    _config: &'a StableAbiOptions<'a>,
    visited_fields: &'a VisitedFieldMap<'a>,
    shared_vars: &mut SharedVars<'a>,
) -> impl ToTokens + 'a {
    let ct = shared_vars.ctokens();

    ToTokenFnMut::new(move |ts| {
        let variant_names_start_len = variant_names_start_len.tokenizer(ct.as_ref());

        let variant_lengths = ds.variants.iter().map(|x| {
            assert!(
                x.fields.len() < 256,
                "variant '{}' has more than 255 fields.",
                x.name
            );
            x.fields.len() as u8
        });

        let fields = fields_tokenizer(ds, visited_fields, ct);

        quote!(
            __sabi_re::MonoTLEnum::new(
                #variant_names_start_len,
                abi_stable::rslice![#( #variant_lengths ),*],
                #fields,
            )
        )
        .to_tokens(ts);
    })
}

// Tokenizes a `GenericTLEnum{ .. }`
fn tokenize_generic_enum<'a>(
    ds: &'a DataStructure<'a>,
    _variant_names_start_len: StartLen,
    nonexhaustive_opt: Option<&'a nonexhaustive::NonExhaustive<'a>>,
    config: &'a StableAbiOptions<'a>,
    _visited_fields: &'a VisitedFieldMap<'a>,
    ct: &'a CommonTokens<'a>,
) -> impl ToTokens + 'a {
    ToTokenFnMut::new(move |ts| {
        let is_exhaustive = match nonexhaustive_opt {
            Some(_) => {
                let name = ds.name;

                let ty_generics = GenParamsIn::new(ds.generics, InWhat::ItemUse);
                // let (_, ty_generics,_) = ds.generics.split_for_impl();
                quote!(nonexhaustive(
                    &__sabi_re::MakeTLNonExhaustive::< #name <#ty_generics> >::NEW
                ))
            }
            None => quote!(exhaustive()),
        };

        let discriminants = ds.variants.iter().map(|x| x.discriminant);
        let discriminants = config.repr.tokenize_discriminant_exprs(discriminants, ct);

        quote!(
            __sabi_re::GenericTLEnum::new(
                __IsExhaustive::#is_exhaustive,
                #discriminants,
            )
        )
        .to_tokens(ts);
    })
}

/// Tokenizes a TLFields,
fn fields_tokenizer<'a>(
    ds: &'a DataStructure<'a>,
    visited_fields: &'a VisitedFieldMap<'a>,
    ctokens: &'a CommonTokens<'a>,
) -> impl ToTokens + 'a {
    ToTokenFnMut::new(move |ts| {
        to_stream!(ts;ctokens.comp_tl_fields,ctokens.colon2,ctokens.new);
        ctokens.paren.surround(ts, |ts| {
            fields_tokenizer_inner(ds, visited_fields, ctokens, ts);
        });
    })
}

fn fields_tokenizer_inner<'a>(
    ds: &'a DataStructure<'a>,
    visited_fields: &'a VisitedFieldMap<'a>,
    ct: &'a CommonTokens<'a>,
    ts: &mut TokenStream2,
) {
    let iter = visited_fields.map.iter().map(|field| field.comp_field);
    rslice_tokenizer(iter).to_tokens(ts);

    ct.comma.to_tokens(ts);

    if visited_fields.fn_ptr_count == 0 {
        ct.none.to_tokens(ts);
    } else {
        to_stream!(ts;ct.some);
        ct.paren.surround(ts, |ts| {
            ct.and_.to_tokens(ts);
            tokenize_tl_functions(ds, visited_fields, ct, ts);
        });
    }
    to_stream! {ts; ct.comma };
}

/// Tokenizes a TLFunctions
fn tokenize_tl_functions<'a>(
    ds: &'a DataStructure<'a>,
    visited_fields: &'a VisitedFieldMap<'a>,
    _ct: &'a CommonTokens<'a>,
    ts: &mut TokenStream2,
) {
    let mut functions =
        CompositeVec::<&'a CompTLFunction>::with_capacity(visited_fields.fn_ptr_count);
    let mut field_fn_ranges = Vec::<StartLen>::with_capacity(ds.field_count);

    visited_fields
        .map
        .iter()
        .map(|field| functions.extend(&field.functions))
        .extending(&mut field_fn_ranges);

    let functions = functions.into_inner();

    let field_fn_ranges = field_fn_ranges.into_iter().map(|sl| sl.to_u32());

    quote!({
        const TLF_A: &[__CompTLFunction] = &[#(#functions),*];
        const TLF_B: __TLFunctions = __TLFunctions::new(
            __sabi_re::RSlice::from_slice(TLF_A),
            abi_stable::rslice![#(#field_fn_ranges),*],
        );
        TLF_B
    })
    .to_tokens(ts);
}
