//! Contains types related to the type layout of function pointers.

use super::*;

use crate::{
    composite_collections::SmallStartLen as StartLen, fn_pointer_extractor::Function,
    fn_pointer_extractor::TypeVisitor, lifetimes::LifetimeRange,
};

use std::marker::PhantomData;

use syn::Type;

///////////////////////////////////////////////////////////////////////////////

/// Associates extra information related to function pointers to a type declaration.
#[allow(dead_code)]
pub(crate) struct VisitedFieldMap<'a> {
    pub(crate) map: Vec<VisitedField<'a>>,
    pub(crate) fn_ptr_count: usize,
    priv_: (),
}

impl<'a> VisitedFieldMap<'a> {
    pub(crate) fn new(
        ds: &'a DataStructure<'a>,
        config: &'a StableAbiOptions<'a>,
        shared_vars: &mut SharedVars<'a>,
        ctokens: &'a CommonTokens<'a>,
    ) -> Self {
        let arenas = shared_vars.arenas();
        let mut tv = TypeVisitor::new(arenas, ctokens.as_ref(), ds.generics);
        if config.allow_type_macros {
            tv.allow_type_macros();
        }

        let mut fn_ptr_count = 0;

        let map = ds
            .variants
            .iter()
            .flat_map(|x| &x.fields)
            .map(|field| {
                // The type used to get the TypeLayout of the field.
                // This has all parameter and return types of function pointers removed.
                // Extracted into the `functions` field of this struct.
                let mut mutated_ty = config.changed_types[field].unwrap_or(field.ty).clone();
                let layout_ctor = config.layout_ctor[field];
                let is_opaque = layout_ctor.is_opaque();

                let is_function = match mutated_ty {
                    Type::BareFn { .. } => !is_opaque,
                    _ => false,
                };

                let visit_info = tv.visit_field(&mut mutated_ty);

                let mutated_ty = arenas.alloc(mutated_ty);

                let field_accessor = config.override_field_accessor[field]
                    .unwrap_or_else(|| config.kind.field_accessor(config.mod_refl_mode, field));

                let name = config.renamed_fields[field].unwrap_or_else(|| field.pat_ident());

                let comp_field = CompTLField::from_expanded(
                    name,
                    visit_info.referenced_lifetimes.iter().cloned(),
                    field_accessor,
                    shared_vars.push_type(layout_ctor, mutated_ty),
                    is_function,
                    shared_vars,
                );

                let iterated_functions = if is_opaque {
                    Vec::new()
                } else {
                    visit_info.functions
                };

                let functions = iterated_functions
                    .iter()
                    .enumerate()
                    .map(|(fn_i, func): (usize, &Function<'_>)| {
                        let name_span = name.span();
                        let name_start_len = if is_function || iterated_functions.len() == 1 {
                            comp_field.name_start_len()
                        } else {
                            shared_vars.push_str(&format!("fn_{}", fn_i), Some(name_span))
                        };

                        shared_vars.combine_err(name_start_len.check_ident_length(name_span));

                        let bound_lifetimes_start_len = shared_vars
                            .extend_with_idents(",", func.named_bound_lts.iter().cloned());

                        let params_iter = func.params.iter().map(|p| match p.name {
                            Some(pname) => (pname as &dyn std::fmt::Display, pname.span()),
                            None => (&"" as &dyn std::fmt::Display, Span::call_site()),
                        });
                        let param_names_len = shared_vars.extend_with_display(",", params_iter).len;

                        let param_type_layouts =
                            TypeLayoutRange::compress_params(&func.params, shared_vars);

                        let paramret_lifetime_range = shared_vars.extend_with_lifetime_indices(
                            func.params
                                .iter()
                                .chain(&func.returns)
                                .flat_map(|p| p.lifetime_refs.iter().cloned()),
                        );

                        let return_type_layout = match &func.returns {
                            Some(ret) => shared_vars.push_type(layout_ctor, ret.ty).to_u10(),
                            None => !0,
                        };

                        CompTLFunction {
                            name: name_start_len,
                            contiguous_strings_offset: bound_lifetimes_start_len.start,
                            bound_lifetimes_len: bound_lifetimes_start_len.len,
                            param_names_len,
                            param_type_layouts,
                            paramret_lifetime_range,
                            return_type_layout,
                            is_unsafe: func.is_unsafe,
                        }
                    })
                    .collect::<Vec<CompTLFunction>>();

                fn_ptr_count += functions.len();

                VisitedField {
                    comp_field,
                    layout_ctor,
                    functions,
                    _marker: PhantomData,
                }
            })
            .collect::<Vec<VisitedField<'a>>>();

        shared_vars.combine_err(tv.get_errors());

        Self {
            map,
            fn_ptr_count,
            priv_: (),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A `Field<'a>` with extra information.
#[allow(dead_code)]
pub struct VisitedField<'a> {
    pub(crate) comp_field: CompTLField,
    pub(crate) layout_ctor: LayoutConstructor,
    /// The function pointers from this field.
    pub(crate) functions: Vec<CompTLFunction>,
    _marker: PhantomData<&'a ()>,
}

///////////////////////////////////////////////////////////////////////////////

/// This is how a function pointer is stored,
/// in which every field is a range into `TLFunctions`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct CompTLFunction {
    name: StartLen,
    contiguous_strings_offset: u16,
    bound_lifetimes_len: u16,
    param_names_len: u16,
    /// Stores `!0` if the return type is `()`.
    return_type_layout: u16,
    paramret_lifetime_range: LifetimeRange,
    param_type_layouts: TypeLayoutRange,
    is_unsafe: bool,
}

impl ToTokens for CompTLFunction {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let name = self.name.to_u32();

        let contiguous_strings_offset = self.contiguous_strings_offset;
        let bound_lifetimes_len = self.bound_lifetimes_len;
        let param_names_len = self.param_names_len;
        let return_type_layout = self.return_type_layout;
        let paramret_lifetime_range = self.paramret_lifetime_range.to_u21();
        let param_type_layouts = self.param_type_layouts.to_u64();
        let is_unsafe = if self.is_unsafe {
            quote!( .set_unsafe() )
        } else {
            TokenStream2::new()
        };

        quote!(
            __CompTLFunction::new(
                #name,
                #contiguous_strings_offset,
                #bound_lifetimes_len,
                #param_names_len,
                #return_type_layout,
                #paramret_lifetime_range,
                #param_type_layouts,
                __TLFunctionQualifiers::NEW
                    #is_unsafe,
            )
        )
        .to_tokens(ts);
    }
}
