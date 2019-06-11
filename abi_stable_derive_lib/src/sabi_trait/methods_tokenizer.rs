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
                method:method,
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
        let (is_trait_method,vis)=match which_item {
            WhichItem::Trait
            |WhichItem::TraitImpl
            |WhichItem::TraitMethodsImpl
            |WhichItem::TraitMethodsDecl=>
                (true,None),
             WhichItem::VtableDecl
            |WhichItem::VtableImpl=>
                (false,Some(trait_def.vis)),
        };
        

        let lifetimes=Some(&method.lifetimes).filter(|l| !l.is_empty() );

        // The name of the method in the __Method trait.
        let name_method=method.name_method;
        // The name of the method in the __Trait trait.
        let method_name=method.name;
        let used_name=match which_item {
            WhichItem::Trait=>method.name,
            WhichItem::TraitImpl=>method.name,
            WhichItem::TraitMethodsImpl=>method.name_method,
            WhichItem::TraitMethodsDecl=>method.name_method,
            WhichItem::VtableDecl=>method.name,
            WhichItem::VtableImpl=>method.name,
        };
        let self_param=match (is_trait_method,&method.self_param) {
            (true,SelfParam::ByRef{lifetime,is_mutable:false})=>
                quote!(& #lifetime self),
            (true,SelfParam::ByRef{lifetime,is_mutable:true})=>
                quote!(& #lifetime mut self),
            (true,SelfParam::ByVal)=>
                quote!(self),
            (false,SelfParam::ByRef{lifetime,is_mutable:false})=>
                quote!(_self:& #lifetime __ErasedObject<_Self>),
            (false,SelfParam::ByRef{lifetime,is_mutable:true})=>
                quote!(_self:& #lifetime mut __ErasedObject<_Self>),
            (false,SelfParam::ByVal)=>
                quote!(_self:__sabi_re::MovePtr<'_,_Self>),
        };

        let param_names_a=method.params.iter()
            .map(move|param|ToTokenFnMut::new(move|ts|{
                if which_item==WhichItem::Trait {
                    param.pattern.to_tokens(ts);
                }else{
                    param.name.to_tokens(ts);
                }
            }));
        let param_ty     =method.params.iter().map(|param| &param.ty   );
        let param_names_c=param_names_a.clone();
        let return_ty=&method.output;
        
        let self_is_sized_bound=Some(&ctokens.self_sized)
            .filter(|_| is_trait_method&&method.self_param==SelfParam::ByVal );

        let abi=if which_item==WhichItem::VtableImpl {
            Some(&ctokens.extern_c)
        }else{
            method.abi
        };

        let user_where_clause=method.where_clause.get_tokenizer(ctokens);

        let other_attrs=if which_item==WhichItem::Trait { 
            method.other_attrs
        }else{ 
            &[] 
        };

        if WhichItem::VtableDecl==which_item {
            let derive_attrs=method.derive_attrs;
            quote!( 
                #(#[#derive_attrs])*
                #vis #used_name:
                    #(for< #(#lifetimes,)* >)*
                    extern "C" fn(
                        #self_param,
                        #( #param_names_a:#param_ty ,)* 
                    ) #(-> #return_ty )*
            )
        }else{
            quote!(
                #(#[#other_attrs])*
                #vis #abi fn #used_name #(< #(#lifetimes,)* >)* (
                    #self_param, 
                    #( #param_names_a:#param_ty ,)* 
                ) #(-> #return_ty )*
                where
                    #self_is_sized_bound
                    #user_where_clause
            )
        }.to_tokens(ts);


        match (which_item,&method.self_param) {
            (WhichItem::Trait,_)=>{
                method.default.to_tokens(ts);
                method.semicolon.to_tokens(ts);
            }
            (WhichItem::TraitImpl,_)=>{
                quote!({
                    self.#name_method(#(#param_names_c,)*)
                }).to_tokens(ts);
            }
            (WhichItem::TraitMethodsDecl,SelfParam::ByRef{is_mutable:false,..})=>{
                quote!(_ErasedPtr: __DerefTrait<Target=()>;).to_tokens(ts);
            }
            (WhichItem::TraitMethodsDecl,SelfParam::ByRef{is_mutable:true,..})=>{
                quote!(_ErasedPtr: __DerefMutTrait<Target=()>;).to_tokens(ts);
            }
            (WhichItem::TraitMethodsDecl,SelfParam::ByVal)=>{
                quote!(_ErasedPtr: __sabi_re::OwnedPointer<Target=()>;).to_tokens(ts);
            }
            (WhichItem::TraitMethodsImpl,SelfParam::ByRef{is_mutable:false,..})=>{
                quote!(
                        _ErasedPtr: __DerefTrait<Target=()>
                    {
                        let __method=self.sabi_et_vtable().#method_name();
                        __method(self.sabi_erased_ref(),#(#param_names_c,)*)
                    }
                ).to_tokens(ts);
            }
            (WhichItem::TraitMethodsImpl,SelfParam::ByRef{is_mutable:true,..})=>{
                quote!(
                        _ErasedPtr: __DerefMutTrait<Target=()>
                    {
                        let __method=self.sabi_et_vtable().#method_name();
                        __method(self.sabi_erased_mut(),#(#param_names_c,)*)
                    }
                ).to_tokens(ts);
            }
            (WhichItem::TraitMethodsImpl,SelfParam::ByVal)=>{
                quote!(
                        _ErasedPtr: __sabi_re::OwnedPointer<Target=()>
                    {
                        let __method=self.sabi_et_vtable().#method_name();
                        self.sabi_with_value(move|_self|__method(_self,#(#param_names_c,)*))
                    }
                ).to_tokens(ts);
            }
            (WhichItem::VtableDecl,_)=>{
                quote!(,).to_tokens(ts);
            }
            (WhichItem::VtableImpl,SelfParam::ByRef{is_mutable:false,..})=>{
                quote!({unsafe{
                    __sabi_re::sabi_from_ref(
                        _self,
                        move|_self| __Trait::#method_name(_self,#(#param_names_c,)*)
                    )
                }}).to_tokens(ts);
            }
            (WhichItem::VtableImpl,SelfParam::ByRef{is_mutable:true,..})=>{
                quote!({unsafe{
                    __sabi_re::sabi_from_mut(
                        _self,
                        move|_self| __Trait::#method_name(_self,#(#param_names_c,)*)
                    )
                }}).to_tokens(ts);
            }
            (WhichItem::VtableImpl,SelfParam::ByVal)=>{
                // This unsafe block is only necessary for `unsafe` methods.
                quote!({unsafe{
                    ::abi_stable::extern_fn_panic_handling!{no_early_return;
                        __Trait::#method_name(_self.into_inner(),#(#param_names_c,)*)
                    }
                }}).to_tokens(ts);
            }
        }
    }
}