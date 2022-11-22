//! Contains the MethodsTokenizer type,
//! which is used to print the definition of the method in different places.
//!
//! Where this is used is determined by WhichItem:
//!
//! - `WhichItem::Trait`:
//!     outputs the method in the trait definition.
//!
//! - `WhichItem::TraitImpl`:
//!     outputs the method in the trait implemetation for the generated trait object.
//!
//! - `WhichItem::TraitObjectImpl`:
//!     outputs the methods in the inherent implemetation of the generated trait object.
//!
//! - `WhichItem::VtableDecl`:
//!     outputs the fields of the trait object vtable.
//!
//! - `WhichItem::VtableImpl`:
//!     outputs the methods used to construct the vtable.
//!
//!

use super::{lifetime_unelider::BorrowKind, *};

use as_derive_utils::to_token_fn::ToTokenFnMut;

use quote::TokenStreamExt;

#[derive(Debug, Copy, Clone)]
pub struct MethodsTokenizer<'a> {
    pub(crate) trait_def: &'a TraitDefinition<'a>,
    pub(crate) which_item: WhichItem,
}

#[derive(Debug, Copy, Clone)]
pub struct MethodTokenizer<'a> {
    trait_def: &'a TraitDefinition<'a>,
    method: &'a TraitMethod<'a>,
    which_item: WhichItem,
}

impl<'a> ToTokens for MethodsTokenizer<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        for method in &self.trait_def.methods {
            MethodTokenizer {
                trait_def: self.trait_def,
                method,
                which_item: self.which_item,
            }
            .to_tokens(ts);
        }
    }
}

impl<'a> ToTokens for MethodTokenizer<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let which_item = self.which_item;
        let method = self.method;
        let trait_def = self.trait_def;
        let ctokens = trait_def.ctokens;
        // is_method: Whether this is a method,instead of an associated function or field.
        //
        // vis: the visibility of the generated method,
        //      None if it's implicit,Some(_) if it's explicit.
        let (is_method, vis) = match which_item {
            WhichItem::Trait | WhichItem::TraitImpl => (true, None),
            WhichItem::TraitObjectImpl => (true, Some(trait_def.submod_vis)),
            WhichItem::VtableDecl | WhichItem::VtableImpl => (false, Some(trait_def.submod_vis)),
        };

        // The default implementation block used both by:
        // - the trait definition.
        // - the trait object inherent impl
        //      (for the case where the method doesn't exist in the vtable).
        let default_ = method
            .default
            .as_ref()
            .filter(|_| !method.disable_inherent_default);

        let lifetimes = Some(&method.lifetimes)
            .filter(|l| !l.is_empty())
            .into_iter();

        let method_name = method.name;
        let method_span = method_name.span();

        let self_ty = if is_method {
            quote_spanned!(method_span=> Self)
        } else {
            quote_spanned!(method_span=> _Self)
        };

        struct WriteLifetime<'a>(Option<&'a syn::Lifetime>);

        impl ToTokens for WriteLifetime<'_> {
            fn to_tokens(&self, ts: &mut TokenStream2) {
                if let Some(lt) = self.0 {
                    lt.to_tokens(ts)
                } else {
                    ts.append_all(quote!('_))
                }
            }
        }

        let self_param = match (is_method, &method.self_param) {
            (
                true,
                SelfParam::ByRef {
                    lifetime,
                    is_mutable: false,
                },
            ) => {
                quote_spanned!(method_span=> & #lifetime self)
            }
            (
                true,
                SelfParam::ByRef {
                    lifetime,
                    is_mutable: true,
                },
            ) => {
                quote_spanned!(method_span=> & #lifetime mut self)
            }
            (true, SelfParam::ByVal) => {
                quote_spanned!(method_span=> self)
            }
            (
                false,
                SelfParam::ByRef {
                    lifetime,
                    is_mutable: false,
                },
            ) => {
                let lifetime = WriteLifetime(*lifetime);
                quote_spanned!(method_span=> _self: __sabi_re::RRef<#lifetime, ()>)
            }
            (
                false,
                SelfParam::ByRef {
                    lifetime,
                    is_mutable: true,
                },
            ) => {
                let lifetime = WriteLifetime(*lifetime);
                quote_spanned!(method_span=> _self: __sabi_re::RMut<#lifetime, ()>)
            }
            (false, SelfParam::ByVal) => {
                quote_spanned!(method_span=> _self:*mut ())
            }
        };

        let param_names_a = method.params.iter().map(move |param| {
            ToTokenFnMut::new(move |ts| match which_item {
                WhichItem::Trait => {
                    param.pattern.to_tokens(ts);
                }
                _ => {
                    param.name.to_tokens(ts);
                }
            })
        });
        let param_ty = method.params.iter().map(|param| &param.ty);
        let param_names_c = param_names_a.clone();
        let param_names_d = param_names_a.clone();
        let param_names_e = method.params.iter().map(|x| x.pattern);
        let return_ty = method.output.iter();

        let self_is_sized_bound = Some(&ctokens.self_sized)
            .filter(|_| is_method && method.self_param == SelfParam::ByVal);

        let abi = match which_item {
            WhichItem::VtableImpl => Some(&ctokens.extern_c),
            _ => method.abi,
        };

        let user_where_clause = method.where_clause.get_tokenizer(ctokens);

        let other_attrs = if which_item == WhichItem::Trait {
            method.other_attrs
        } else {
            &[]
        };

        if WhichItem::VtableImpl == which_item {
            ts.append_all(quote_spanned!(method_span=> #[doc(hidden)] ));
        }

        if WhichItem::VtableDecl == which_item {
            let optional_field = default_.as_ref().map(|_| &ctokens.missing_field_option);
            let derive_attrs = method.derive_attrs;

            quote_spanned!( method_span=>
                #optional_field
                #(#derive_attrs)*
                #vis #method_name:
                    #(for< #(#lifetimes,)* >)*
                    unsafe extern "C" fn(
                        #self_param,
                        #( #param_names_a:#param_ty ,)*
                    ) #(-> #return_ty )*
            )
        } else {
            let inherent_method_docs = ToTokenFnMut::new(|ts| {
                if WhichItem::TraitObjectImpl != which_item {
                    return;
                }
                let trait_name = trait_def.name;
                let m_docs = format!(
                    "This is the inherent equivalent of \
                     [the trait method of the same name](./trait.{TN}.html#tymethod.{TM})\
                    ",
                    TN = trait_name,
                    TM = method_name,
                );

                ts.append_all(quote!(#[doc = #m_docs]));
            });

            let unsafety = match which_item {
                WhichItem::VtableImpl => Some(&ctokens.unsafe_),
                _ => method.unsafety,
            };

            quote_spanned!(method_span=>
                #[allow(clippy::let_and_return)]
                #(#other_attrs)*
                #inherent_method_docs
                #vis #unsafety #abi fn #method_name #(< #(#lifetimes,)* >)* (
                    #self_param,
                    #( #param_names_a:#param_ty ,)*
                ) #(-> #return_ty )*
                where
                    #self_is_sized_bound
                    #user_where_clause
            )
        }
        .to_tokens(ts);

        let ptr_constraint = match &method.self_param {
            SelfParam::ByRef {
                is_mutable: false, ..
            } => &ctokens.ptr_ref_bound,
            SelfParam::ByRef {
                is_mutable: true, ..
            } => &ctokens.ptr_mut_bound,
            SelfParam::ByVal => &ctokens.ptr_val_bound,
        };

        let output_safety = |output: &mut TokenStream2, input: TokenStream2| {
            output.append_all(if let Some(safety) = method.unsafety {
                quote_spanned!(safety.span => { #safety{ #input } })
            } else {
                quote_spanned!(method_span => { #input })
            });
        };

        match (which_item, &method.self_param) {
            (WhichItem::Trait, _) => {
                method.default.as_ref().map(|x| x.block).to_tokens(ts);
                method.semicolon.to_tokens(ts);
            }
            (WhichItem::TraitImpl, _) => {
                output_safety(
                    ts,
                    quote_spanned!(method_span =>
                        self.#method_name(#(#param_names_c,)*)
                    ),
                );
            }
            (WhichItem::TraitObjectImpl, _) => {
                let method_call = match &method.self_param {
                    SelfParam::ByRef {
                        is_mutable: false, ..
                    } => {
                        quote_spanned!(method_span=>
                            __method(self.obj.sabi_as_rref(),#(#param_names_c,)*)
                        )
                    }
                    SelfParam::ByRef {
                        is_mutable: true, ..
                    } => {
                        quote_spanned!(method_span=>
                            __method(self.obj.sabi_as_rmut(),#(#param_names_c,)*)
                        )
                    }
                    SelfParam::ByVal => {
                        quote_spanned!(method_span=>
                            self.obj.sabi_with_value(
                                move|_self|__method(
                                    __sabi_re::MovePtr::into_raw(_self) as *mut (),
                                    #(#param_names_c,)*
                                )
                            )
                        )
                    }
                };

                match default_ {
                    Some(default_) => {
                        let block = &default_.block;
                        ts.append_all(quote_spanned!(method_span=>
                                #ptr_constraint
                            {
                                match self.sabi_vtable().#method_name() {
                                    Some(__method)=>{
                                        unsafe{
                                            #method_call
                                        }
                                    }
                                    None=>{
                                        #(
                                            let #param_names_e=#param_names_d;
                                        )*
                                        #block
                                    }
                                }
                            }
                        ));
                    }
                    None => {
                        ts.append_all(quote_spanned!(method_span=>
                                #ptr_constraint
                            {
                                let __method=self.sabi_vtable().#method_name();
                                unsafe{
                                    #method_call
                                }
                            }
                        ));
                    }
                }
            }
            (WhichItem::VtableDecl, _) => {
                quote_spanned!(method_span=> , ).to_tokens(ts);
            }
            (WhichItem::VtableImpl, SelfParam::ByRef { is_mutable, .. }) => {
                let mut_token = ToTokenFnMut::new(|ts| {
                    if *is_mutable {
                        syn::token::Mut { span: method_span }.to_tokens(ts);
                    }
                });

                let ret = syn::Ident::new("ret", proc_macro2::Span::call_site());

                let transmute_ret = match method.return_borrow_kind {
                    Some(BorrowKind::Reference) => {
                        quote_spanned!(method_span=> ::std::mem::transmute(#ret) )
                    }
                    Some(BorrowKind::MutReference) => {
                        quote_spanned!(method_span=> ::std::mem::transmute(#ret) )
                    }
                    Some(BorrowKind::Other) => {
                        // Motivation:
                        // We need to use this transmute to return a borrow from `_self`,
                        // without adding a `_Self: '_self` bound,
                        // which causes compilation errors due to how HRTB are handled.
                        // The correctness of the lifetime is guaranteed by the trait definition.
                        quote_spanned!(method_span=> __sabi_re::transmute_ignore_size(#ret) )
                    }
                    None => quote_spanned!(method_span=> #ret ),
                };

                ts.append_all(quote_spanned!(method_span=>{
                    unsafe{
                        let #ret = ::abi_stable::extern_fn_panic_handling!{no_early_return;
                            __Trait::#method_name(
                                &#mut_token *_self.transmute_into_raw::<#self_ty>(),
                                #(#param_names_c,)*
                            )
                        };

                        #transmute_ret
                    }
                }));
            }
            (WhichItem::VtableImpl, SelfParam::ByVal) => {
                ts.append_all(quote_spanned!(method_span=>{
                    ::abi_stable::extern_fn_panic_handling!{no_early_return; unsafe{
                        __Trait::#method_name(
                            (_self as *mut #self_ty).read(),#(#param_names_c,)*
                        )
                    }}
                }));
            }
        }
    }
}
