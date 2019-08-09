/*!
Contains the MethodsTokenizer type,
which is used to print the definition of the method in different places.

Where this is used is determined by WhichItem:

- `WhichItem::Trait`: 
    outputs the method in the trait definition.

- `WhichItem::TraitImpl`: 
    outputs the method in the trait implemetation for the generated trait object.

- `WhichItem::TraitObjectImpl`:
    outputs the methods in the inherent implemetation of the generated trait object.

- `WhichItem::VtableDecl`: 
    outputs the fields of the trait object vtable.

- `WhichItem::VtableImpl`: 
    outputs the methods used to construct the vtable.


*/

use super::*;

use crate::to_token_fn::ToTokenFnMut;

#[derive(Debug,Copy,Clone)]
pub struct MethodsTokenizer<'a>{
    pub(crate) trait_def:&'a TraitDefinition<'a>,
    pub(crate) which_item:WhichItem,
}


#[derive(Debug,Copy,Clone)]
pub struct MethodTokenizer<'a>{
    trait_def:&'a TraitDefinition<'a>,
    method:&'a TraitMethod<'a>,
    which_item:WhichItem,
}


impl<'a> ToTokens for MethodsTokenizer<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        for method in &self.trait_def.methods {
            MethodTokenizer{
                trait_def:self.trait_def,
                method,
                which_item:self.which_item,
            }.to_tokens(ts);
        }
    }
}
        
impl<'a> ToTokens for MethodTokenizer<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let which_item=self.which_item;
        let method=self.method;
        let trait_def=self.trait_def;
        let ctokens=trait_def.ctokens;
        // is_method: Whether this is a method,instead of an associated function or field.
        //
        // vis: the visibility of the generated method,
        //      None if it's implicit,Some(_) if it's explicit.
        let (is_method,vis)=match which_item {
            WhichItem::Trait|WhichItem::TraitImpl=>{
                (true,None)
            }
            WhichItem::TraitObjectImpl=>{
                (true,Some(trait_def.submod_vis))
            }
            WhichItem::VtableDecl|WhichItem::VtableImpl=>{
                (false,Some(trait_def.submod_vis))
            }
        };
        
        // The default implementation block used both by:
        // - the trait definition.
        // - the trait object inherent impl
        //      (for the case where the method doesn't exist in the vtable).
        let default_=method.default.as_ref();

        let lifetimes=Some(&method.lifetimes).filter(|l| !l.is_empty() );

        let method_name=method.name;
        let method_span=method_name.span();

        let self_param=match (is_method,&method.self_param) {
            (true,SelfParam::ByRef{lifetime,is_mutable:false})=>{
                quote_spanned!(method_span=> & #lifetime self)
            }
            (true,SelfParam::ByRef{lifetime,is_mutable:true})=>{
                quote_spanned!(method_span=> & #lifetime mut self)
            }
            (true,SelfParam::ByVal)=>{
                quote_spanned!(method_span=> self)
            }
            (false,SelfParam::ByRef{lifetime,is_mutable:false})=>{
                quote_spanned!(method_span=> _self:& #lifetime __ErasedObject<_Self>)
            }
            (false,SelfParam::ByRef{lifetime,is_mutable:true})=>{
                quote_spanned!(method_span=> _self:& #lifetime mut __ErasedObject<_Self>)
            }
            (false,SelfParam::ByVal)=>{
                quote_spanned!(method_span=> _self:__sabi_re::MovePtr<'_,_Self>)
            }
        };

        let param_names_a=method.params.iter()
            .map(move|param|ToTokenFnMut::new(move|ts|{
                match which_item {
                    WhichItem::Trait=>{
                        param.pattern.to_tokens(ts);
                    }
                    _=>{
                        param.name.to_tokens(ts);
                    }
                }
            }));
        let param_ty     =method.params.iter().map(|param| &param.ty   );
        let param_names_c=param_names_a.clone();
        let param_names_d=param_names_a.clone();
        let param_names_e=method.params.iter().map(|x| x.pattern );
        let return_ty=&method.output;
        
        let self_is_sized_bound=Some(&ctokens.self_sized)
            .filter(|_| is_method&&method.self_param==SelfParam::ByVal );

        let abi=match which_item {
             WhichItem::VtableImpl=>Some(&ctokens.extern_c),
            _=>method.abi,
        };

        let user_where_clause=method.where_clause.get_tokenizer(ctokens);

        let other_attrs=if which_item==WhichItem::Trait { 
            method.other_attrs
        }else{ 
            &[] 
        };

        if WhichItem::VtableDecl==which_item {
            let optional_field=method.default.as_ref().map(|_| &ctokens.missing_field_option );
            let derive_attrs=method.derive_attrs;
            quote_spanned!( method_span=>
                #(#[#derive_attrs])*
                #optional_field
                #vis #method_name:
                    #(for< #(#lifetimes,)* >)*
                    unsafe extern "C" fn(
                        #self_param,
                        #( #param_names_a:#param_ty ,)* 
                    ) #(-> #return_ty )*
            )
        }else{
            let unsafety=match which_item {
                WhichItem::VtableImpl=>Some(&ctokens.unsafe_),
                _=>method.unsafety
            };

            quote_spanned!(method_span=>
                #(#[#other_attrs])*
                #vis #unsafety #abi fn #method_name #(< #(#lifetimes,)* >)* (
                    #self_param, 
                    #( #param_names_a:#param_ty ,)* 
                ) #(-> #return_ty )*
                where
                    #self_is_sized_bound
                    #user_where_clause
            )
        }.to_tokens(ts);

        let ptr_constraint=match &method.self_param {
            SelfParam::ByRef{is_mutable:false,..}=>
                &ctokens.ptr_ref_bound,
            SelfParam::ByRef{is_mutable:true,..}=>
                &ctokens.ptr_mut_bound,
            SelfParam::ByVal=>
                &ctokens.ptr_val_bound,
        };

        match (which_item,&method.self_param) {
            (WhichItem::Trait,_)=>{
                method.default.as_ref().map(|x|x.block).to_tokens(ts);
                method.semicolon.to_tokens(ts);
            }
            (WhichItem::TraitImpl,_)=>{
                quote_spanned!(method_span=>{
                    self.#method_name(#(#param_names_c,)*)
                }).to_tokens(ts);
            }
            (WhichItem::TraitObjectImpl,_)=>{
                let method_call=match &method.self_param {
                    SelfParam::ByRef{is_mutable:false,..}=>{
                        quote_spanned!(method_span=> 
                            __method(self.obj.sabi_erased_ref(),#(#param_names_c,)*) 
                        )
                    }
                    SelfParam::ByRef{is_mutable:true,..}=>{
                        quote_spanned!(method_span=> 
                            __method(self.obj.sabi_erased_mut(),#(#param_names_c,)*) 
                        )
                    }
                    SelfParam::ByVal=>{
                        quote_spanned!(method_span=>
                            self.obj.sabi_with_value(
                                move|_self|__method(_self,#(#param_names_c,)*)
                            )
                        )
                    }
                };

                match default_ {
                    Some(default_)=>{
                        let block=&default_.block;
                        quote_spanned!(method_span=>
                                #ptr_constraint
                            {
                                match self.obj.sabi_et_vtable().#method_name() {
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
                        ).to_tokens(ts);
                    }
                    None=>{
                        quote_spanned!(method_span=>
                                #ptr_constraint
                            {
                                let __method=self.obj.sabi_et_vtable().#method_name();
                                unsafe{
                                    #method_call
                                }
                            }
                        ).to_tokens(ts);
                    }
                }
            }
            (WhichItem::VtableDecl,_)=>{
                quote_spanned!(method_span=> , ).to_tokens(ts);
            
            }
            (WhichItem::VtableImpl,SelfParam::ByRef{is_mutable:false,..})=>{
                quote_spanned!(method_span=>{
                    __sabi_re::sabi_from_ref(
                        _self,
                        move|_self| 
                            __Trait::#method_name(_self,#(#param_names_c,)*)
                    )
                }).to_tokens(ts);
            }
            (WhichItem::VtableImpl,SelfParam::ByRef{is_mutable:true,..})=>{
                quote_spanned!(method_span=>{
                    __sabi_re::sabi_from_mut(
                        _self,
                        move|_self| 
                            __Trait::#method_name(_self,#(#param_names_c,)*)
                    )
                }).to_tokens(ts);
            }
            (WhichItem::VtableImpl,SelfParam::ByVal)=>{
                quote_spanned!(method_span=>{
                    ::abi_stable::extern_fn_panic_handling!{no_early_return;
                        __Trait::#method_name(
                            _self.into_inner(),#(#param_names_c,)*
                        )
                    }
                }).to_tokens(ts);
            }
        }
    }
}