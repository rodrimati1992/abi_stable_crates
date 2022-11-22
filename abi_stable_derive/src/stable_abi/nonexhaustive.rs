use std::collections::HashMap;

use core_extensions::SelfOps;

use syn::{visit_mut::VisitMut, Ident};

use quote::{quote, ToTokens};

use proc_macro2::{Span, TokenStream as TokenStream2};

use as_derive_utils::{
    datastructure::DataStructure,
    gen_params_in::{GenParamsIn, InWhat},
    return_spanned_err, spanned_err,
    to_token_fn::ToTokenFnMut,
};

use super::{
    attribute_parsing::{StabilityKind, StableAbiOptions},
    common_tokens::CommonTokens,
    StartLen,
};

use crate::{
    arenas::{AllocMethods, Arenas},
    impl_interfacetype::{private_associated_type, UsableTrait, TRAIT_LIST},
    literals_constructors::rstr_tokenizer,
    parse_utils::{parse_str_as_ident, parse_str_as_path},
    set_span_visitor::SetSpanVisitor,
    utils::{LinearResult, SynResultExt},
};

/// Used while parsing the `#[sabi(kind(WithNonExhaustive(...)))]` attribute.
#[derive(Clone, Default)]
pub(crate) struct UncheckedNonExhaustive<'a> {
    pub(crate) alignment: Option<ExprOrType<'a>>,
    pub(crate) size: Option<ExprOrType<'a>>,
    pub(crate) enum_interface: Option<EnumInterface<'a>>,
    pub(crate) assert_nonexh: Vec<&'a syn::Type>,
}

/// The configuration for code generation related to nonexhaustive enums.
#[derive(Clone)]
pub(crate) struct NonExhaustive<'a> {
    pub(crate) nonexhaustive_alias: &'a Ident,
    /// The identifier for the interface parameter of `NonExhaustive<>`(the third one).
    pub(crate) nonexhaustive_marker: &'a Ident,
    /// The identifier for the storage space used to store the enum within `NonExhaustive<>`
    pub(crate) enum_storage: &'a Ident,
    /// The alignment of `#enum_storage`
    pub(crate) alignment: ExprOrType<'a>,
    /// The size of `#enum_storage`
    pub(crate) size: ExprOrType<'a>,
    /// The InterfaceType-implementing marker struct this will generate,
    /// if this is:
    ///     - `Some(EnumInterface::New{..})`:it will use a new struct as the InterfaceType.
    ///     - `Some(EnumInterface::Old{..})`:it will use an existing type as the InterfaceType.
    ///     - `None`: it will use `()` as the InterfaceType.
    /// rather than generate one.
    pub(crate) enum_interface: Option<EnumInterface<'a>>,
    /// The identifier of the InterfaceType-implementing struct this will generate.
    pub(crate) new_interface: Option<&'a Ident>,
    /// The type used as the InterfaceType parameter of `NonExhaustive<>` by default.
    pub(crate) default_interface: TokenStream2,
    /// The types that will be tested as being compatible with their storage and interface.
    pub(crate) assert_nonexh: Vec<&'a syn::Type>,
    /// This is a trait aliasing the constraints required when
    /// wrapping the enum inside `NonExhaustive<>`.
    /// This is None when the enum uses a pre-existing InterfaceType as
    /// its interface (the third type parameter of `NonExhaustive<>`)
    pub(crate) bounds_trait: Option<BoundsTrait<'a>>,
    /// The constructor functions generated for each variant,
    /// with the enum wrapped inside `NonExhaustive<>`
    pub(crate) ne_variants: Vec<NEVariant<'a>>,
}

/// The configuration required to generate a trait aliasing the constraints required when
/// wrapping the enum inside `NonExhaustive<>`.
#[derive(Clone)]
pub struct BoundsTrait<'a> {
    ident: &'a Ident,
    bounds: Vec<&'a syn::Path>,
}

#[derive(Clone)]
pub struct UncheckedNEVariant {
    pub(crate) constructor: Option<UncheckedVariantConstructor>,
    pub(crate) is_hidden: bool,
}

#[derive(Clone)]
pub struct NEVariant<'a> {
    pub(crate) constructor: Option<VariantConstructor<'a>>,
    pub(crate) is_hidden: bool,
}

/// How a NonExhaustive<Enum,...> is constructed
#[derive(Clone)]
pub enum UncheckedVariantConstructor {
    /// Constructs an enum variant using a function
    /// with parameters of the same type as the fields.
    Regular,
    /// Constructs an enum variant containing a pointer,
    /// using a function taking the referent of the pointer.
    Boxed,
}

/// How variant(s) of the enum wrapped inside `NonExhaustive<>` is constructed.
#[derive(Clone)]
pub enum VariantConstructor<'a> {
    /// Constructs an enum variant using a function
    /// with parameters of the same type as the fields.
    Regular,
    /// Constructs an enum variant containing a pointer,
    /// using a function taking the referent of the pointer.
    ///
    /// The type of the referent is extracted from the first type parameter
    /// in the type of the only field of a variant.
    Boxed {
        referent: Option<&'a syn::Type>,
        pointer: &'a syn::Type,
    },
}

/// The InterfaceType of the enum,used as the third type parameter of NonExhaustive.
#[derive(Clone)]
pub(crate) enum EnumInterface<'a> {
    New(NewEnumInterface<'a>),
    Old(&'a syn::Type),
}

/// The traits that are specified in `impl InterfaceType for Enum_Interface`,
/// which specifies the traits that are required when wrapping the enum in `NonExhaustive<>`,
/// and are then available when using it.
#[derive(Default, Clone)]
pub(crate) struct NewEnumInterface<'a> {
    pub(crate) impld: Vec<&'a Ident>,
    pub(crate) unimpld: Vec<&'a Ident>,
}

impl<'a> NonExhaustive<'a> {
    pub fn new(
        mut unchecked: UncheckedNonExhaustive<'a>,
        ne_variants: Vec<UncheckedNEVariant>,
        ds: &'a DataStructure<'a>,
        arenas: &'a Arenas,
    ) -> Result<Self, syn::Error> {
        let name = ds.name;

        let alignment = unchecked.alignment.unwrap_or(ExprOrType::Usize);

        let parse_ident = move |s: &str, span: Option<Span>| -> &'a Ident {
            let mut ident = parse_str_as_ident(s);
            if let Some(span) = span {
                ident.set_span(span)
            }
            arenas.alloc(ident)
        };

        let mut errors = LinearResult::ok(());

        let size = unchecked.size.unwrap_or_else(|| {
            errors.push_err(spanned_err!(
                name,
                "\n\
                You must specify the size of the enum storage in NonExhaustive<> using \
                the `size=integer literal` or `size=\"type\"` argument inside of \
                the `#[sabi(kind(WithNonExhaustive(...)))]` helper attribute.\n\
                "
            ));
            ExprOrType::Int(0)
        });

        let mut bounds_trait = None::<BoundsTrait<'a>>;

        if let Some(EnumInterface::New(enum_interface)) = &mut unchecked.enum_interface {
            let mut trait_map = TRAIT_LIST
                .iter()
                .map(|x| (parse_ident(x.name, None), x))
                .collect::<HashMap<&'a syn::Ident, &'static UsableTrait>>();

            let mut bounds_trait_inner = Vec::<&'a syn::Path>::new();

            for &trait_ in &enum_interface.impld {
                match trait_map.remove(trait_) {
                    Some(ut) => {
                        use crate::impl_interfacetype::WhichTrait as WT;
                        if let WT::Deserialize = ut.which_trait {
                            continue;
                        }
                        let mut full_path = parse_str_as_path(ut.full_path)?;

                        SetSpanVisitor::new(trait_.span()).visit_path_mut(&mut full_path);

                        bounds_trait_inner.push(arenas.alloc(full_path));
                    }
                    None => {
                        // This is an internal error.
                        panic!("Trait {} was not in TRAIT_LIST.", trait_)
                    }
                }
            }

            bounds_trait = Some(BoundsTrait {
                ident: parse_ident(&format!("{}_Bounds", name), None),
                bounds: bounds_trait_inner,
            });

            for &trait_ in &enum_interface.unimpld {
                // This is an internal error.
                assert!(
                    trait_map.remove(trait_).is_some(),
                    "Trait {} was not in TRAIT_LIST.",
                    trait_
                );
            }

            for (trait_, _) in trait_map {
                enum_interface.unimpld.push(trait_);
            }
        }

        let (default_interface, new_interface) = match unchecked.enum_interface {
            Some(EnumInterface::New { .. }) => {
                let name = parse_ident(&format!("{}_Interface", name), None);
                (name.into_token_stream(), Some(name))
            }
            Some(EnumInterface::Old(ty)) => ((&ty).into_token_stream(), None),
            None => (quote!(()), None),
        };

        let ne_variants = ne_variants
            .into_iter()
            .zip(&ds.variants)
            .map(|(vc, variant)| {
                let constructor = match vc.constructor {
                    Some(UncheckedVariantConstructor::Regular) => Some(VariantConstructor::Regular),
                    Some(UncheckedVariantConstructor::Boxed) => match variant.fields.first() {
                        Some(first_field) => Some(VariantConstructor::Boxed {
                            referent: extract_first_type_param(first_field.ty),
                            pointer: first_field.ty,
                        }),
                        None => Some(VariantConstructor::Regular),
                    },
                    None => None,
                };

                NEVariant {
                    constructor,
                    is_hidden: vc.is_hidden,
                }
            })
            .collect();

        errors.into_result()?;

        Ok(Self {
            nonexhaustive_alias: parse_ident(&format!("{}_NE", name), None),
            nonexhaustive_marker: parse_ident(&format!("{}_NEMarker", name), None),
            enum_storage: parse_ident(&format!("{}_Storage", name), None),
            alignment,
            size,
            enum_interface: unchecked.enum_interface,
            default_interface,
            new_interface,
            assert_nonexh: unchecked.assert_nonexh,
            bounds_trait,
            ne_variants,
        })
    }
}

#[derive(Copy, Clone)]
pub enum ExprOrType<'a> {
    Int(usize),
    Expr(&'a syn::Expr),
    Type(&'a syn::Type),
    Usize,
}

/// Extracts the first type parameter of a generic type.
fn extract_first_type_param(ty: &syn::Type) -> Option<&syn::Type> {
    match ty {
        syn::Type::Path(path) => {
            if path.qself.is_some() {
                return None;
            }
            let args = &path.path.segments.last()?.arguments;
            let args = match args {
                syn::PathArguments::AngleBracketed(x) => x,
                _ => return None,
            };
            args.args.iter().find_map(|arg| match arg {
                syn::GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
        }
        _ => None,
    }
}

fn hide_docs_if<T, F>(opt: &Option<T>, func: F) -> String
where
    F: FnOnce() -> String,
{
    if opt.is_some() {
        String::new()
    } else {
        func()
    }
}

/// Outputs the nonexhausitve-enum-related items,
/// outside the module generated by StableAbi.
pub(crate) fn tokenize_nonexhaustive_items<'a>(
    ds: &'a DataStructure<'a>,
    config: &'a StableAbiOptions<'a>,
    _ct: &'a CommonTokens<'a>,
) -> impl ToTokens + 'a {
    ToTokenFnMut::new(move |ts| {
        let this = match &config.kind {
            StabilityKind::NonExhaustive(x) => x,
            _ => return,
        };
        let doc_hidden_attr = config.doc_hidden_attr;
        let vis = ds.vis;
        let nonexhaustive_alias = this.nonexhaustive_alias;
        let nonexhaustive_marker = this.nonexhaustive_marker;
        let enum_storage = this.enum_storage;

        let (aligner_attribute, aligner_field) = match this.alignment {
            ExprOrType::Int(bytes) => {
                let bytes = crate::utils::expr_from_int(bytes as _);
                (Some(quote!(#[repr(align(#bytes))])), None)
            }
            ExprOrType::Expr(expr) => (
                None,
                Some(quote!(
                    __aligner: [
                        ::abi_stable::pmr::GetAlignerFor<::abi_stable::pmr::u8, #expr>;
                        0
                    ]
                )),
            ),
            ExprOrType::Type(ty) => (None, Some(quote!(__aligner:[#ty;0],))),
            ExprOrType::Usize => (
                None,
                Some(quote!(__aligner: [::abi_stable::pmr::usize; 0],)),
            ),
        };

        let aligner_size = match this.size {
            ExprOrType::Int(size) => quote!( #size ),
            ExprOrType::Expr(expr) => quote!( (#expr) ),
            ExprOrType::Type(ty) => quote!( ::std::mem::size_of::<#ty>() ),
            ExprOrType::Usize => quote!(::std::mem::size_of::<::abi_stable::pmr::usize>()),
        };

        let name = ds.name;

        let generics_header = GenParamsIn::new(ds.generics, InWhat::ImplHeader);

        let mut type_generics_decl = GenParamsIn::new(ds.generics, InWhat::ImplHeader);
        type_generics_decl.set_no_bounds();

        let type_generics_use = GenParamsIn::new(ds.generics, InWhat::ItemUse);

        let mut storage_docs = String::new();
        let mut alias_docs = String::new();
        let mut marker_docs = String::new();

        if doc_hidden_attr.is_none() {
            storage_docs = format!(
                "The default InlineStorage that `NonExhaustive` uses for \
                 [`{E}`](./enum.{E}.html).",
                E = name
            );
            alias_docs = format!(
                "An alias for `NonExhaustive` wrapping a [`{E}`](./enum.{E}.html).",
                E = name
            );
            marker_docs = format!(
                "A marker type which implements StableAbi with the layout of \
                 [`{E}`](./enum.{E}.html),\
                 used as a phantom field of NonExhaustive.",
                E = name
            );
        }

        let default_interface = &this.default_interface;

        quote!(
            #[doc=#storage_docs]
            #[repr(C)]
            #[derive(::abi_stable::StableAbi)]
            #aligner_attribute
            #vis struct #enum_storage{
                #[sabi(unsafe_opaque_field)]
                _filler:[u8; #aligner_size ],
                #aligner_field
            }

            #[doc=#alias_docs]
            #vis type #nonexhaustive_alias<#type_generics_decl>=
                ::abi_stable::pmr::NonExhaustive<
                    #name<#type_generics_use>,
                    #enum_storage,
                    #default_interface,
                >;

            unsafe impl ::abi_stable::pmr::InlineStorage for #enum_storage{}

            #[doc=#marker_docs]
            #vis struct #nonexhaustive_marker<T,S>(
                std::marker::PhantomData<T>,
                std::marker::PhantomData<S>,
            );
        )
        .to_tokens(ts);

        if let Some(BoundsTrait { ident, bounds }) = &this.bounds_trait {
            let trait_docs = hide_docs_if(&doc_hidden_attr, || {
                format!(
                    "An alias for the traits that \
                    `NonExhaustive<{E},_,_>` requires to be constructed,\
                    and implements afterwards.",
                    E = name
                )
            });

            quote!(
                #[doc=#trait_docs]
                #vis trait #ident:#(#bounds+)*{}

                impl<This> #ident for This
                where
                    This:#(#bounds+)*
                {}
            )
            .to_tokens(ts);
        }

        if let Some(new_interface) = this.new_interface {
            let interface_docs = hide_docs_if(&doc_hidden_attr, || {
                format!(
                    "Describes the traits required when constructing a \
                     `NonExhaustive<>` from [`{E}`](./enum.{E}.html),\
                     by implementing `InterfaceType`.",
                    E = name
                )
            });

            quote!(
                #[doc=#interface_docs]
                #[repr(C)]
                #[derive(::abi_stable::StableAbi)]
                #vis struct #new_interface;
            )
            .to_tokens(ts);
        }

        if this.ne_variants.iter().any(|x| x.constructor.is_some()) {
            let constructors = this
                .ne_variants
                .iter()
                .cloned()
                .zip(&ds.variants)
                .filter_map(|(vc, variant)| {
                    let constructor = vc.constructor.as_ref()?;
                    let variant_ident = variant.name;
                    let mut method_name = parse_str_as_ident(&format!("{}_NE", variant.name));
                    method_name.set_span(variant.name.span());

                    let method_docs = if vc.is_hidden {
                        quote!(#[doc(hidden)])
                    } else {
                        let v_doc = format!(
                            "Constructs the `{}::{}` variant inside a `NonExhaustive`.",
                            ds.name, variant.name,
                        );
                        quote!(#[doc= #v_doc])
                    };

                    match constructor {
                        VariantConstructor::Regular => {
                            let field_names_a = variant.fields.iter().map(|x| x.pat_ident());
                            let field_names_b = field_names_a.clone();
                            let field_names_c = variant.fields.iter().map(|x| &x.ident);
                            let field_types = variant.fields.iter().map(|x| x.ty);
                            quote! {
                                #method_docs
                                #vis fn #method_name(
                                    #( #field_names_a : #field_types ,)*
                                )->#nonexhaustive_alias<#type_generics_use> {
                                    let x=#name::#variant_ident{
                                        #( #field_names_c:#field_names_b, )*
                                    };
                                    #nonexhaustive_alias::new(x)
                                }
                            }
                        }
                        VariantConstructor::Boxed { referent, pointer } => {
                            let ptr_field_ident = &variant.fields[0].ident;
                            let type_param = ToTokenFnMut::new(|ts| match referent {
                                Some(x) => x.to_tokens(ts),
                                None => {
                                    quote!( <#pointer as ::abi_stable::pmr::GetPointerKind>::PtrTarget )
                                        .to_tokens(ts)
                                }
                            });

                            quote! {
                                #method_docs
                                #vis fn #method_name(
                                    value:#type_param,
                                )->#nonexhaustive_alias<#type_generics_use> {
                                    let x=<#pointer>::new(value);
                                    let x=#name::#variant_ident{
                                        #ptr_field_ident:x,
                                    };
                                    #nonexhaustive_alias::new(x)
                                }
                            }
                        }
                    }
                    .piped(Some)
                });

            let preds = ds.generics.where_clause.as_ref().map(|w| &w.predicates);

            let bound = match &this.bounds_trait {
                Some(BoundsTrait { ident, .. }) => quote!(#ident),
                None => quote!(
                    ::abi_stable::pmr::NonExhaustiveMarkerVTable<
                        #enum_storage,
                        #default_interface,
                    >
                ),
            };

            quote!(
                #[allow(non_snake_case)]
                impl<#generics_header> #name<#type_generics_use>
                where
                    Self: #bound ,
                    #preds
                {
                    #(#constructors)*
                }
            )
            .to_tokens(ts);
        }
    })
}

/// Outputs the nonexhausitve-enum-related impl blocks,
/// inside the module generated by StableAbi.
pub(crate) fn tokenize_enum_info<'a>(
    ds: &'a DataStructure<'a>,
    variant_names_start_len: StartLen,
    config: &'a StableAbiOptions<'a>,
    ct: &'a CommonTokens<'a>,
) -> Result<impl ToTokens + 'a, syn::Error> {
    let opt_type_ident = config.repr.type_ident();
    if let (StabilityKind::NonExhaustive { .. }, None) = (&config.kind, &opt_type_ident) {
        return_spanned_err!(
            ds.name,
            "Attempted to get type of discriminant for this representation:\n\t{:?}",
            config.repr
        );
    }

    Ok(ToTokenFnMut::new(move |ts| {
        let this = match &config.kind {
            StabilityKind::NonExhaustive(x) => x,
            _ => return,
        };

        let name = ds.name;
        let name_str = rstr_tokenizer(ds.name.to_string());

        let strings_const = &config.const_idents.strings;

        let discriminants = ds
            .variants
            .iter()
            .map(|x| x.discriminant)
            .collect::<Vec<Option<&'a syn::Expr>>>();

        let discriminant_tokens = config
            .repr
            .tokenize_discriminant_slice(discriminants.iter().cloned(), ct);

        let discriminant_type = match &opt_type_ident {
            Some(x) => x,
            None => unreachable!(),
        };

        let vn_start = variant_names_start_len.start;
        let vn_len = variant_names_start_len.len;

        let nonexhaustive_marker = this.nonexhaustive_marker;
        let enum_storage = this.enum_storage;

        let mut start_discrs = Vec::new();
        let mut end_discrs = Vec::new();
        if !discriminants.is_empty() {
            let mut first_index = 0;

            for (mut i, discr) in discriminants[1..].iter().cloned().enumerate() {
                i += 1;
                if discr.is_some() {
                    start_discrs.push(first_index);
                    end_discrs.push(i - 1);
                    first_index = i;
                }
            }

            start_discrs.push(first_index);
            end_discrs.push(discriminants.len() - 1);
        }

        let generics_header =
            GenParamsIn::with_after_types(ds.generics, InWhat::ImplHeader, &ct.und_storage);

        let generics_use = GenParamsIn::new(ds.generics, InWhat::ImplHeader);

        let default_interface = &this.default_interface;

        let (impl_generics, ty_generics, where_clause) = ds.generics.split_for_impl();

        let preds = where_clause.as_ref().map(|w| &w.predicates);

        quote!(

            unsafe impl #impl_generics __sabi_re::GetStaticEquivalent_ for #name #ty_generics
            where
                #nonexhaustive_marker <Self,#enum_storage> :
                    __sabi_re::GetStaticEquivalent_,
                #preds
            {
                type StaticEquivalent=__sabi_re::GetStaticEquivalent<
                    #nonexhaustive_marker <Self,#enum_storage>
                >;
            }

            unsafe impl #impl_generics __sabi_re::GetEnumInfo for #name #ty_generics
            #where_clause
            {
                type Discriminant=#discriminant_type;

                type DefaultStorage=#enum_storage;

                type DefaultInterface=#default_interface;

                const ENUM_INFO:&'static __sabi_re::EnumInfo=
                    &__sabi_re::EnumInfo::_for_derive(
                        #name_str,
                        #strings_const,
                        ::abi_stable::type_layout::StartLen::new(#vn_start,#vn_len),
                    );

                const DISCRIMINANTS: &'static[#discriminant_type]=
                    #discriminant_tokens;

                fn is_valid_discriminant(discriminant:#discriminant_type)->bool{
                    #(
                        (
                            <Self as __sabi_re::GetEnumInfo>::DISCRIMINANTS[#start_discrs]
                            <= discriminant &&
                            discriminant <=
                            <Self as __sabi_re::GetEnumInfo>::DISCRIMINANTS[#end_discrs]
                        )||
                    )*
                    false
                }
            }


            unsafe impl<#generics_header>
                __sabi_re::NonExhaustiveMarker<__Storage>
            for #name <#generics_use>
            #where_clause
            {
                type Marker = #nonexhaustive_marker<Self,__Storage>;
            }


        )
        .to_tokens(ts);

        let self_type: syn::Type;
        let self_type_buf: Vec<&syn::Type>;
        let assert_nonexh = if this.assert_nonexh.is_empty() && ds.generics.params.is_empty() {
            let name = ds.name;
            self_type = syn::parse_quote!(#name);
            self_type_buf = vec![&self_type];
            &self_type_buf
        } else {
            &this.assert_nonexh
        };

        if !assert_nonexh.is_empty() {
            let assertions = assert_nonexh.iter().cloned();
            let assertions_str = assert_nonexh
                .iter()
                .map(|x| x.to_token_stream().to_string());
            let enum_storage_str = enum_storage.to_string();
            quote!(
                #(
                    const _: () = ::abi_stable::pmr::assert_correct_storage::<#assertions, #enum_storage>(
                        ::abi_stable::pmr::AssertCsArgs{
                            enum_ty: #assertions_str,
                            storage_ty: #enum_storage_str,
                        }
                    );
                )*
            )
            .to_tokens(ts);
        }

        match &this.enum_interface {
            Some(EnumInterface::New(NewEnumInterface { impld, unimpld })) => {
                let enum_interface = parse_str_as_ident(&format!("{}_Interface", name));

                let priv_assocty = private_associated_type();

                let impld_a = impld.iter();
                let impld_b = impld.iter();

                let unimpld_a = unimpld.iter();
                let unimpld_b = unimpld.iter();

                let const_ident =
                    parse_str_as_ident(&format!("_impl_InterfaceType_constant_{}", name,));

                quote!(
                    const #const_ident:()={
                        use abi_stable::{
                            InterfaceType,
                            type_level::{
                                impl_enum::{Implemented,Unimplemented},
                                trait_marker,
                            },
                        };
                        impl InterfaceType for #enum_interface {
                            #( type #impld_a=Implemented<trait_marker::#impld_b>; )*
                            #( type #unimpld_a=Unimplemented<trait_marker::#unimpld_b>; )*
                            type #priv_assocty=();
                        }
                    };
                )
                .to_tokens(ts);
            }
            Some(EnumInterface::Old { .. }) => {}
            None => {}
        }
    }))
}
