//! Code generation for prefix-types.

use abi_stable_shared::const_utils::low_bit_mask_u64;

use core_extensions::{matches, SelfOps};

use syn::{punctuated::Punctuated, Ident, TypeParamBound, Visibility, WherePredicate};

use quote::{quote_spanned, ToTokens, TokenStreamExt};

use proc_macro2::Span;

use as_derive_utils::{
    datastructure::{DataStructure, Field, FieldIndex, FieldMap},
    return_spanned_err,
    to_token_fn::ToTokenFnMut,
};

use crate::*;

use crate::{literals_constructors::rstr_tokenizer, parse_utils::parse_str_as_ident};

use super::{
    attribute_parsing::{StabilityKind, StableAbiOptions},
    reflection::FieldAccessor,
    CommonTokens,
};

/// Configuration for code generation related to prefix-types.
pub(crate) struct PrefixKind<'a> {
    pub(crate) first_suffix_field: FirstSuffixField,
    pub(crate) prefix_ref: &'a Ident,
    pub(crate) prefix_fields_struct: &'a Ident,
    pub(crate) replacing_prefix_ref_docs: &'a [syn::Expr],
    pub(crate) prefix_bounds: Vec<WherePredicate>,
    pub(crate) fields: FieldMap<AccessorOrMaybe<'a>>,
    pub(crate) accessor_bounds: FieldMap<Vec<TypeParamBound>>,
    pub(crate) cond_field_indices: Vec<usize>,
    pub(crate) enable_field_if: Vec<&'a syn::Expr>,
    pub(crate) unconditional_bit_mask: u64,
    pub(crate) prefix_field_conditionality_mask: u64,
}

pub(crate) struct PrefixKindCtor<'a> {
    pub(crate) arenas: &'a Arenas,
    #[allow(dead_code)]
    pub(crate) struct_name: &'a Ident,
    pub(crate) first_suffix_field: FirstSuffixField,
    pub(crate) prefix_ref: Option<&'a Ident>,
    pub(crate) prefix_fields: Option<&'a Ident>,
    pub(crate) replacing_prefix_ref_docs: &'a [syn::Expr],
    pub(crate) prefix_bounds: Vec<WherePredicate>,
    pub(crate) fields: FieldMap<AccessorOrMaybe<'a>>,
    pub(crate) accessor_bounds: FieldMap<Vec<TypeParamBound>>,
}

impl<'a> PrefixKindCtor<'a> {
    pub fn make(self) -> PrefixKind<'a> {
        let ctor = self;
        let mut cond_field_indices = Vec::<usize>::new();
        let mut enable_field_if = Vec::<&syn::Expr>::new();
        let mut unconditional_bit_mask = 0u64;
        let mut conditional_bit_mask = 0u64;

        for (index, field) in ctor.fields.iter() {
            let field_i = index.pos;
            match (|| field.to_maybe_accessor()?.accessible_if)() {
                Some(cond) => {
                    cond_field_indices.push(field_i);
                    enable_field_if.push(cond);
                    conditional_bit_mask |= 1u64 << field_i;
                }
                None => {
                    unconditional_bit_mask |= 1u64 << field_i;
                }
            }
        }

        let prefix_field_conditionality_mask =
            conditional_bit_mask & low_bit_mask_u64(ctor.first_suffix_field.field_pos as u32);

        PrefixKind {
            first_suffix_field: ctor.first_suffix_field,
            prefix_ref: ctor.prefix_ref.unwrap_or_else(|| {
                let ident = format!("{}_Ref", ctor.struct_name);
                ctor.arenas.alloc(parse_str_as_ident(&ident))
            }),
            prefix_fields_struct: ctor.prefix_fields.unwrap_or_else(|| {
                let ident = format!("{}_Prefix", ctor.struct_name);
                ctor.arenas.alloc(parse_str_as_ident(&ident))
            }),
            replacing_prefix_ref_docs: ctor.replacing_prefix_ref_docs,
            prefix_bounds: ctor.prefix_bounds,
            fields: ctor.fields,
            accessor_bounds: ctor.accessor_bounds,
            cond_field_indices,
            enable_field_if,
            unconditional_bit_mask,
            prefix_field_conditionality_mask,
        }
    }
}

/// Used while parsing the prefix-type-related attributes on fields.
#[derive(Copy, Default, Clone)]
pub(crate) struct PrefixKindField<'a> {
    pub(crate) accessible_if: Option<&'a syn::Expr>,
    pub(crate) on_missing: Option<OnMissingField<'a>>,
}

/// The different types of prefix-type accessors.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AccessorOrMaybe<'a> {
    /// Unconditionally returns the field.
    Accessor,
    /// Either optionally returns the field,or it does some action when it's missing.
    Maybe(MaybeAccessor<'a>),
}

/// Describes a field accessor which is either optional or
/// does some action when the field is missing.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct MaybeAccessor<'a> {
    /// If Some,it uses a bool constant to determine whether a field is accessible.
    accessible_if: Option<&'a syn::Expr>,
    /// What the accessor method does when the field is missing.
    on_missing: OnMissingField<'a>,
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub(crate) struct FirstSuffixField {
    pub(crate) field_pos: usize,
}

/// What happens in a Prefix-type field getter if the field does not exist.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum OnMissingField<'a> {
    /// Returns an `Option<FieldType>`,where it returns None if the field is absent.
    ReturnOption,
    /// Panics with a default message.
    Panic,
    /// Evaluates `function()`,and returns the return value of the function.
    With { function: &'a syn::Path },
    /// Returns `some_expression`.
    Value { value: &'a syn::Expr },
    /// Returns Default::default
    Default_,
}

impl<'a> Default for OnMissingField<'a> {
    fn default() -> Self {
        OnMissingField::ReturnOption
    }
}

impl<'a> AccessorOrMaybe<'a> {
    pub(crate) fn new(
        field_i: FieldIndex,
        first_suffix_field: FirstSuffixField,
        pkf: PrefixKindField<'a>,
        default_omf: OnMissingField<'a>,
    ) -> Self {
        if field_i.pos < first_suffix_field.field_pos
            && pkf.accessible_if.is_none()
            && pkf.on_missing != Some(OnMissingField::ReturnOption)
        {
            AccessorOrMaybe::Accessor
        } else {
            AccessorOrMaybe::Maybe(MaybeAccessor {
                accessible_if: pkf.accessible_if,
                on_missing: pkf.on_missing.unwrap_or(default_omf),
            })
        }
    }

    #[allow(dead_code)]
    pub(crate) fn is_conditional(&self) -> bool {
        self.to_maybe_accessor()
            .map_or(false, |x| x.accessible_if.is_some())
    }

    /// Converts this to a MaybeAccessor,returning None if it is not the `Maybe` variant.
    pub(crate) fn to_maybe_accessor(self) -> Option<MaybeAccessor<'a>> {
        match self {
            AccessorOrMaybe::Maybe(x) => Some(x),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn is_maybe_accessor(&self) -> bool {
        matches!(self, AccessorOrMaybe::Maybe { .. })
    }
}

impl<'a> PrefixKind<'a> {
    /// Gets the accessibility for a field,used for (very basic) runtime reflection.
    pub(crate) fn field_accessor(&self, field: &Field<'a>) -> FieldAccessor<'a> {
        use self::OnMissingField as OMF;

        match self.fields[field] {
            AccessorOrMaybe::Accessor => FieldAccessor::Method { name: None },
            AccessorOrMaybe::Maybe(MaybeAccessor { on_missing, .. }) => match on_missing {
                OMF::ReturnOption => FieldAccessor::MethodOption,
                OMF::Panic { .. } | OMF::With { .. } | OMF::Value { .. } | OMF::Default_ => {
                    FieldAccessor::Method { name: None }
                }
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/////                 Code generation
////////////////////////////////////////////////////////////////////////////////

pub struct PrefixTypeTokens {
    pub prefixref_types: TokenStream2,
    pub prefixref_impls: TokenStream2,
}

/// Returns a value which for a prefix-type .
pub(crate) fn prefix_type_tokenizer<'a>(
    mono_type_layout: &'a Ident,
    ds: &'a DataStructure<'a>,
    config: &'a StableAbiOptions<'a>,
    _ctokens: &'a CommonTokens<'a>,
) -> Result<PrefixTypeTokens, syn::Error> {
    if matches!(config.kind, StabilityKind::Prefix { .. }) {
        if ds
            .variants
            .get(0)
            .map_or(false, |struct_| struct_.fields.len() > 64)
        {
            return_spanned_err!(
                ds.name,
                "`#[sabi(kind(Prefix(..)))]` structs cannot have more than 64 fields."
            );
        }

        if config.repr.is_packed.is_some() {
            return_spanned_err!(
                ds.name,
                "`#[sabi(kind(Prefix(..)))]` structs cannot be `#[repr(C, packed)]`"
            );
        }
    }

    Ok({
        fn default_prefixtype_tokens() -> PrefixTypeTokens {
            PrefixTypeTokens {
                prefixref_types: TokenStream2::new(),
                prefixref_impls: TokenStream2::new(),
            }
        }

        let struct_ = match ds.variants.get(0) {
            Some(x) => x,
            None => return Ok(default_prefixtype_tokens()),
        };

        let prefix = match &config.kind {
            StabilityKind::Prefix(prefix) => prefix,
            _ => return Ok(default_prefixtype_tokens()),
        };

        let first_suffix_field = prefix.first_suffix_field.field_pos;
        let prefix_fields_struct = &prefix.prefix_fields_struct;

        let doc_hidden_attr = config.doc_hidden_attr;

        let deriving_name = ds.name;
        let (ref impl_generics, ref ty_generics, ref where_clause) = ds.generics.split_for_impl();

        let empty_preds = Punctuated::new();
        let where_preds = where_clause
            .as_ref()
            .map_or(&empty_preds, |x| &x.predicates)
            .into_iter();
        let where_preds_a = where_preds.clone();
        let where_preds_b = where_preds.clone();
        let where_preds_c = where_preds.clone();
        // let where_preds_d0=where_preds.clone();
        // let where_preds_d1=where_preds.clone();
        let where_preds_e = where_preds.clone();
        let where_preds_f = where_preds.clone();
        let where_preds_rl = where_preds.clone();
        let where_preds_r2 = where_preds.clone();
        let prefix_bounds = &prefix.prefix_bounds;

        let stringified_deriving_name = deriving_name.to_string();

        let stringified_generics = (&ty_generics).into_token_stream().to_string();
        let stringified_generics_tokenizer = rstr_tokenizer(&stringified_generics);

        let is_ds_pub = matches!(ds.vis, Visibility::Public { .. }) && doc_hidden_attr.is_none();
        let has_replaced_docs = !prefix.replacing_prefix_ref_docs.is_empty();

        let prefix_ref_docs = ToTokenFnMut::new(|ts| {
            if !is_ds_pub {
                return;
            }

            if has_replaced_docs {
                let iter = prefix.replacing_prefix_ref_docs.iter();
                ts.append_all(quote!(#(#[doc = #iter])*));
            } else {
                let single_docs = format!(
                    "\
                        This is the pointer to the prefix of \n\
                        [`{deriving_name}{generics}`](struct@{deriving_name}).\n\
                        \n\
                        **This is automatically generated documentation,\
                        by the StableAbi derive macro**.\n\
                        \n\
                        ### Creating a compiletime-constant\n\
                        \n\
                        You can look at the docs in [`::abi_stable::docs::prefix_types`] \
                        to see how you\n\
                        can construct and use this and similar types.<br>\n\
                        More specifically in the \n\
                        [\"constructing a module\" example\n\
                        ](::abi_stable::docs::prefix_types#module_construction) or the\n\
                        [\"Constructing a vtable\" example\n\
                        ](::abi_stable::docs::prefix_types#vtable_construction)\n\
                    ",
                    deriving_name = stringified_deriving_name,
                    generics = stringified_generics,
                );

                ts.append_all(quote!(#[doc = #single_docs]));
            }
        });

        let prefix_fields_docs = if is_ds_pub {
            format!(
                "\
This is the prefix fields of 
[`{deriving_name}{generics}`](struct@{deriving_name}),
accessible through [`{prefix_name}`](struct@{prefix_name}), with `.0.prefix()`.

**This is automatically generated documentation,by the StableAbi derive macro**.
                ",
                prefix_name = prefix.prefix_ref,
                deriving_name = stringified_deriving_name,
                generics = stringified_generics,
            )
        } else {
            String::new()
        };

        let prefix_ref = prefix.prefix_ref;

        // Generating the `<prefix_ref>` struct
        let generated_types = {
            let vis = ds.vis;
            let generics = ds.generics;

            let alignemnt = if let Some(alignemnt) = config.repr.is_aligned {
                let alignemnt = as_derive_utils::utils::uint_lit(alignemnt.into());
                quote!(, align(#alignemnt))
            } else {
                quote!()
            };

            let prefix_field_iter = struct_.fields[..first_suffix_field].iter();
            // This uses `field_<NUMBER>` for tuple fields
            let prefix_field_vis = prefix_field_iter.clone().map(|f| {
                ToTokenFnMut::new(move |ts| {
                    let vis = f.vis;
                    if AccessorOrMaybe::Accessor == prefix.fields[f] {
                        vis.to_tokens(ts);
                    }
                })
            });
            let prefix_field = prefix_field_iter.clone().map(|f| f.pat_ident());
            let prefix_field_ty = prefix_field_iter.clone().map(|f| f.ty);

            let (_, ty_generics, where_clause) = generics.split_for_impl();

            quote!(
                #doc_hidden_attr
                #prefix_ref_docs
                #[repr(transparent)]
                #vis struct #prefix_ref #generics (
                    #vis ::abi_stable::pmr::PrefixRef<
                        #prefix_fields_struct #ty_generics,
                    >
                )#where_clause;

                #doc_hidden_attr
                #[doc=#prefix_fields_docs]
                // A struct with all the prefix fields in the deriving type
                //
                // A field being in the prefix doesn't mean that it's
                // unconditionally accessible, it just means that it won't cause a SEGFAULT.
                #[repr(C #alignemnt)]
                #vis struct #prefix_fields_struct #generics
                #where_clause
                {
                    #(
                        ///
                        #prefix_field_vis #prefix_field: #prefix_field_ty,
                    )*
                    // Using this to ensure:
                    // - That all the generic arguments are used
                    // - That the struct has the same alignemnt as the deriving struct.
                    __sabi_pt_prefix_alignment: [#deriving_name #ty_generics  ;0],
                    // Using this to ensure that the struct has at least the alignment of usize,
                    // so that adding pointer fields is not an ABI breaking change.
                    __sabi_usize_alignment: [usize; 0],
                    __sabi_pt_unbounds: ::abi_stable::pmr::NotCopyNotClone,
                }
            )
        };

        let mut accessor_buffer = String::new();

        let mut uncond_acc_docs = Vec::<String>::new();
        let mut cond_acc_docs = Vec::<String>::new();
        let mut field_index_for = Vec::new();
        let offset_consts: &[syn::Ident] = &struct_
            .fields
            .iter()
            .map(|f| parse_str_as_ident(&format!("__sabi_offset_for_{}", f.pat_ident())))
            .collect::<Vec<Ident>>();

        // Creates the docs for the accessor functions.
        // Creates the identifiers for constants associated with each field.
        for field in struct_.fields.iter() {
            use self::AccessorOrMaybe as AOM;
            use std::fmt::Write;

            let mut acc_doc_buffer = String::new();
            let acc_on_missing = prefix.fields[field];
            if is_ds_pub {
                let _ = write!(
                    acc_doc_buffer,
                    "Accessor method for the `{deriving_name}::{field_name}` field.",
                    deriving_name = deriving_name,
                    field_name = field.pat_ident(),
                );

                match acc_on_missing {
                    AOM::Accessor => {
                        acc_doc_buffer.push_str("This is for a field which always exists.")
                    }
                    AOM::Maybe(MaybeAccessor {
                        on_missing: OnMissingField::ReturnOption,
                        ..
                    }) => acc_doc_buffer.push_str(
                        "Returns `Some(field_value)` if the field exists,\
                             `None` if it does not.\
                            ",
                    ),
                    AOM::Maybe(MaybeAccessor {
                        on_missing: OnMissingField::Panic,
                        ..
                    }) => acc_doc_buffer
                        .push_str("\n\n# Panic\n\nPanics if the field does not exist."),
                    AOM::Maybe(MaybeAccessor {
                        on_missing: OnMissingField::With { function },
                        ..
                    }) => write!(
                        acc_doc_buffer,
                        "Returns `{function}()` if the field does not exist.",
                        function = (&function).into_token_stream()
                    )
                    .drop_(),
                    AOM::Maybe(MaybeAccessor {
                        on_missing: OnMissingField::Value { .. },
                        ..
                    }) => acc_doc_buffer.push_str(
                        "\
                            Returns a default value (not Default::default()) \
                            if the field does not exist.\
                        ",
                    ),
                    AOM::Maybe(MaybeAccessor {
                        on_missing: OnMissingField::Default_,
                        ..
                    }) => acc_doc_buffer
                        .push_str("Returns `Default::default()` if the field does not exist."),
                };
            }

            if config.with_field_indices {
                let field_name = field.pat_ident();
                let mut new_ident = parse_str_as_ident(&format!("field_index_for_{}", field_name));
                new_ident.set_span(field_name.span());
                field_index_for.push(new_ident);
            }

            match acc_on_missing {
                AOM::Accessor => {
                    uncond_acc_docs.push(acc_doc_buffer);
                }
                AOM::Maybe { .. } => {
                    cond_acc_docs.push(acc_doc_buffer);
                }
            }
        }

        let mut unconditional_accessors = Vec::new();
        let mut conditional_accessors = Vec::new();

        // Creates TokenStreams for each accessor function.
        for (field_i, field) in struct_.fields.iter().enumerate() {
            use std::fmt::Write;
            accessor_buffer.clear();
            write!(accessor_buffer, "{}", field.pat_ident()).drop_();
            let vis = field.vis;
            let mut getter_name = syn::parse_str::<Ident>(&accessor_buffer).expect("BUG");
            getter_name.set_span(field.pat_ident().span());
            let field_name = field.pat_ident();
            let field_span = field_name.span();
            let ty = field.ty;

            let accessor_bounds = &prefix.accessor_bounds[field];

            let field_where_clause = if accessor_bounds.is_empty() {
                None
            } else {
                Some(quote!(where #ty:))
            };

            match prefix.fields[field] {
                AccessorOrMaybe::Accessor => {
                    unconditional_accessors.push(quote_spanned! {field_span=>
                        #[allow(clippy::missing_const_for_fn)]
                        #vis fn #getter_name(&self)->#ty
                        #field_where_clause #( #accessor_bounds+ )*
                        {
                            self.0.prefix().#field_name
                        }
                    })
                }
                AccessorOrMaybe::Maybe(maybe_accessor) => {
                    let field_offset = &offset_consts[field_i];
                    let on_missing_field = maybe_accessor.on_missing;
                    let is_optional = on_missing_field == OnMissingField::ReturnOption;

                    let return_ty = if is_optional {
                        quote!( Option< #ty > )
                    } else {
                        quote!( #ty)
                    };

                    let else_ = match on_missing_field {
                        OnMissingField::ReturnOption => quote_spanned! {field_span=>
                            return None
                        },
                        OnMissingField::Panic => quote_spanned!(field_span=>
                            __sabi_re::panic_on_missing_field_ty::<
                                #deriving_name #ty_generics
                            >(
                                #field_i,
                                self._prefix_type_layout(),
                            )
                        ),
                        OnMissingField::With { function } => quote_spanned! {field_span=>
                            #function()
                        },
                        OnMissingField::Value { value } => quote_spanned! {field_span=>
                            (#value)
                        },
                        OnMissingField::Default_ => quote_spanned! {field_span=>
                            Default::default()
                        },
                    };

                    let val_var = syn::Ident::new("val", Span::mixed_site());

                    let with_val = if is_optional {
                        quote_spanned!(field_span=> Some(#val_var) )
                    } else {
                        val_var.to_token_stream()
                    };

                    conditional_accessors.push(quote_spanned! {field_span=>
                        #[allow(clippy::missing_const_for_fn)]
                        #vis fn #getter_name(&self)->#return_ty
                        #field_where_clause #( #accessor_bounds+ )*
                        {
                            let acc_bits=self.0.field_accessibility().bits();
                            let #val_var=if (1u64<<#field_i & Self::__SABI_PTT_FAM & acc_bits)==0 {
                                #else_
                            }else{
                                unsafe{
                                    *((self.0.to_raw_ptr() as *const u8)
                                        .offset(Self::#field_offset as isize)
                                        as *const #ty)
                                }
                            };
                            #with_val
                        }
                    });
                }
            }
        }

        let cond_field_indices = &prefix.cond_field_indices;
        let enable_field_if = &prefix.enable_field_if;
        let unconditional_bit_mask = &prefix.unconditional_bit_mask;

        let cond_field_indices = cond_field_indices.iter();
        let enable_field_if = enable_field_if.iter();

        let field_i_a = 0u8..;

        let mut pt_layout_ident = parse_str_as_ident(&format!("__sabi_PT_LAYOUT{}", deriving_name));
        pt_layout_ident.set_span(deriving_name.span());

        let mut generated_impls = quote!(
            #[allow(non_upper_case_globals)]
            const #pt_layout_ident:&'static __sabi_re::PTStructLayout ={
                &__sabi_re::PTStructLayout::new(
                    #stringified_generics_tokenizer,
                    #mono_type_layout,
                )
            };

            unsafe impl #impl_generics
                __sabi_re::PrefixTypeTrait
            for #deriving_name #ty_generics
            where
                #(#where_preds_a,)*
                #(#prefix_bounds,)*
            {
                // Describes the accessibility of all the fields,
                // used to initialize the `WithMetadata<Self>::_prefix_type_field_acc` field.
                const PT_FIELD_ACCESSIBILITY:__sabi_re::FieldAccessibility={
                    __sabi_re::FieldAccessibility::from_u64(
                        #unconditional_bit_mask
                        #(
                            |(((#enable_field_if)as u64) << #cond_field_indices)
                        )*
                    )
                };
                // A description of the struct used for error messages.
                const PT_LAYOUT:&'static __sabi_re::PTStructLayout =#pt_layout_ident;

                type PrefixFields = #prefix_fields_struct #ty_generics;
                type PrefixRef = #prefix_ref #ty_generics;
            }

            #[allow(non_upper_case_globals, clippy::needless_lifetimes, clippy::new_ret_no_self)]
            impl #impl_generics #prefix_ref #ty_generics
            where
                #(#where_preds_b,)*
            {
                #(
                    #[doc=#uncond_acc_docs]
                    #unconditional_accessors
                )*

                #(
                    // This is the field index,starting with 0,from the top field.
                    const #field_index_for:u8=
                        #field_i_a;
                )*
            }
        );

        let first_offset = if let Some(constant) = offset_consts.first() {
            quote!(
                const #constant: usize = {
                    __sabi_re::WithMetadata_::<
                        #prefix_fields_struct #ty_generics,
                        #prefix_fields_struct #ty_generics,
                    >::__VALUE_OFFSET
                };
            )
        } else {
            quote!()
        };

        let prev_offsets = offset_consts.iter();
        let curr_offsets = prev_offsets.clone().skip(1);
        let prev_tys = struct_.fields.iter().map(|f| f.ty);
        let curr_tys = prev_tys.clone().skip(1);

        generated_impls.append_all(quote!(
            #[allow(
                clippy::ptr_offset_with_cast,
                clippy::needless_lifetimes,
                clippy::new_ret_no_self,
                non_upper_case_globals,
            )]
            impl #impl_generics #prefix_ref #ty_generics
            where
                #(#where_preds_c,)*
                #(#prefix_bounds,)*
            {
                #first_offset
                #(
                    const #curr_offsets: usize =
                        __sabi_re::next_field_offset::<
                            #deriving_name #ty_generics,
                            #prev_tys,
                            #curr_tys,
                        >(Self::#prev_offsets);
                )*

                // The accessibility of all fields,
                // used below to initialize the mask for each individual field.
                //
                // If the nth bit is:
                //    0:the field is inaccessible.
                //    1:the field is accessible.
                const __SABI_PTT_FAM:u64=
                    <#deriving_name #ty_generics as
                        __sabi_re::PrefixTypeTrait
                    >::PT_FIELD_ACCESSIBILITY.bits();

                /// Accessor to get the layout of the type,used for error messages.
                #[inline(always)]
                pub fn _prefix_type_layout(self)-> &'static __sabi_re::PTStructLayout {
                    self.0.type_layout()
                }

                #(
                    #[doc=#cond_acc_docs]
                    #conditional_accessors
                )*
            }

            unsafe impl #impl_generics __sabi_re::GetPointerKind for #prefix_ref #ty_generics
            where
                #(#where_preds_rl,)*
            {
                type PtrTarget = __sabi_re::WithMetadata_<
                    #prefix_fields_struct #ty_generics,
                    #prefix_fields_struct #ty_generics,
                >;

                type Kind = __sabi_re::PK_Reference;
            }

            unsafe impl #impl_generics __sabi_re::PrefixRefTrait for #prefix_ref #ty_generics
            where
                #(#where_preds_r2,)*
            {
                type PrefixFields = #prefix_fields_struct #ty_generics;
            }

            impl #impl_generics Copy for #prefix_ref #ty_generics
            where
                #(#where_preds_e,)*
            {}

            impl #impl_generics Clone for #prefix_ref #ty_generics
            where
                #(#where_preds_f,)*
            {
                fn clone(&self) -> Self {
                    *self
                }
            }

        ));

        PrefixTypeTokens {
            prefixref_types: generated_types,
            prefixref_impls: generated_impls,
        }
    })
}
