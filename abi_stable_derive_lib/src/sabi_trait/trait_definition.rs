use super::{
    *,
    attribute_parsing::{MethodWithAttrs,OwnedDeriveAndOtherAttrs},
    impl_interfacetype::{TRAIT_LIST,UsableTrait},
    replace_self_path::{self,ReplaceWith},
    parse_utils::{parse_str_as_ident,parse_str_as_path},
};

use std::{
    collections::HashMap,
    rc::Rc,
};

use core_extensions::{matches,IteratorExt};

use syn::{
    token::{Semi,Comma},
    punctuated::Punctuated,
    Ident,ItemTrait,Visibility,FnArg,LifetimeDef,MethodSig,Meta,
    TypeParamBound,Block,WherePredicate,TraitItem,Abi,
    token::Unsafe,
};




#[derive(Debug,Clone)]
pub struct AssocTyWithIndex{
    pub index:usize,
    pub assoc_ty:syn::TraitItemType,
}


pub type RcUsableTrait=Rc<UsableTrait<&'static str,syn::Path>>;


////////////////////////////////////////////////////////////////////////////////


#[derive(Debug,Clone)]
pub(crate) struct TraitDefinition<'a>{
    pub(crate) item:&'a ItemTrait,
    pub(crate) name:&'a Ident,
    pub(crate) where_preds:Punctuated<WherePredicate,Comma>,
    pub(crate) derive_attrs:&'a [Meta],
    pub(crate) other_attrs:&'a [Meta],
    pub(crate) generics:&'a syn::Generics,
    pub(crate) impld_traits:Vec<RcUsableTrait>,
    pub(crate) unimpld_traits:Rc<HashMap<Ident,RcUsableTrait>>,
    pub(crate) vis:MyVisibility<'a>,
    pub(crate) submod_vis:RelativeVis<'a>,
    // The keys use the proginal identifier for the associated type.
    pub(crate) assoc_tys:HashMap<&'a Ident, AssocTyWithIndex>,
    pub(crate) methods:Vec<TraitMethod<'a>>,
    pub(crate) has_mut_methods:bool,
    pub(crate) has_val_methods:bool,
    pub(crate) ts_fq_self:&'a TokenStream2,
    pub(crate) ctokens:&'a CommonTokens,
    pub(crate) arenas:&'a Arenas,
}

////////////////////////////////////////////////////////////////////////////////

impl<'a> TraitDefinition<'a>{
    pub fn new(
        trait_:&'a ItemTrait,
        attrs:OwnedDeriveAndOtherAttrs,
        methods_with_attrs:Vec<MethodWithAttrs<'a>>,
        arenas: &'a Arenas,
        ctokens:&'a CommonTokens,
    )->Self {
        let trait_map=TRAIT_LIST.iter()
            .map(|t|{
                let val=UsableTrait{
                    name:t.name,
                    full_path:parse_str_as_path(t.full_path),
                    default_value:t.default_value,
                    object_safe:t.object_safe,
                    usable_by:t.usable_by,
                };
                (parse_str_as_ident(t.name),Rc::new(val)) 
            })
            .collect::<HashMap<Ident,_>>();

        let mut impld_traits=Vec::new();
        let mut unimpld_traits=trait_map.clone();
        let vis=MyVisibility::new(&trait_.vis);
        let submod_vis=vis.submodule_level(1);
        let mut assoc_tys=HashMap::default();
        let mut methods=Vec::<TraitMethod<'a>>::new();


        for supertrait_bound in &trait_.supertraits{
            let trait_bound=match supertrait_bound {
                TypeParamBound::Trait(x)=>x,
                _=>continue,
            };
            let last_path_component=match trait_bound.path.segments.last() {
                Some(x)=>&x.value().ident,
                None=>continue,
            };

            match trait_map.get(&last_path_component) {
                Some(supertrait)=>{
                    unimpld_traits.remove(&last_path_component);
                    impld_traits.push(supertrait.clone());
                },
                None=>{
                    let list=trait_map.keys().map(|x| x.to_string() ).collect::<Vec<String>>();

                    panic!(
                        "Unexpected supertrait bound:\n\t{}\nExpected one of:\n{}\n", 
                        supertrait_bound.into_token_stream(),
                        list.join("/"),
                    );
                },
            }
        }

        methods_with_attrs.into_iter()
            .filter_map(|func| TraitMethod::new(func,&trait_.vis,ctokens,arenas) )
            .extending(&mut methods);


        let mut assoc_ty_index=0;
        for item in &trait_.items {
            match item {
                TraitItem::Method{..}=>{},
                TraitItem::Type(assoc_ty)=>{
                    let with_index=AssocTyWithIndex{
                        index:assoc_ty_index,
                        assoc_ty:assoc_ty.clone()
                    };
                    assoc_tys.insert(&assoc_ty.ident,with_index);

                    assoc_ty_index+=1;
                },
                item=>panic!(
                    "\nAssociated item not compatible with #[sabi_trait]:\n\t{}\n\n",
                    item.into_token_stream(),
                ),
            }
        }

        let has_mut_methods=methods.iter()
            .any(|m| matches!(SelfParam::ByRef{is_mutable:true,..}= &m.self_param) );

        let has_val_methods=methods.iter()
            .any(|m| matches!(SelfParam::ByVal= &m.self_param) );
        

        let ts_fq_self={
            let (_,generics_params,_)=trait_.generics.split_for_impl();
            quote!( <_OrigPtr::Target as __Trait #generics_params >:: )
        };

        TraitDefinition{
            item:trait_,
            name:&trait_.ident,
            where_preds:trait_.generics.where_clause.as_ref()
                .map(|wc| wc.predicates.clone() )
                .unwrap_or_default(),
            derive_attrs:arenas.alloc(attrs.derive_attrs),
            other_attrs:arenas.alloc(attrs.other_attrs),
            generics:&trait_.generics,
            impld_traits,
            unimpld_traits:Rc::new(unimpld_traits),
            vis,
            submod_vis,
            assoc_tys,
            methods,
            has_mut_methods,
            has_val_methods,
            ts_fq_self:arenas.alloc(ts_fq_self),
            ctokens,
            arenas,
        }
    }

    pub fn replace_self(&self,which_item:WhichItem)->Self{
        let mut this=self.clone();

        let ctokens=self.ctokens;
        let arenas=self.arenas;

        // `is_alt_trait==true` means that the returned trait is `trait __Methods`,
        // which changes the names of associated types.
        let (is_alt_trait,replace_with)=match which_item {
            WhichItem::Trait|WhichItem::TraitImpl=>return this,
            WhichItem::TraitMethodsDecl|WhichItem::TraitMethodsImpl=>{
                (true ,ReplaceWith::Ident(ctokens.capself.clone()))
            },
            WhichItem::VtableDecl=>{
                (false,ReplaceWith::Remove)
            }
            WhichItem::VtableImpl=>{
                (false,ReplaceWith::Ident(ctokens.u_capself.clone()))
            },
        };

        let mut renamed_assoc_tys:Vec<&'a Ident>=Vec::new();

        if is_alt_trait {
            // pre-initializing the vec with dummy identifiers because 
            // it will be initialized in random order.
            renamed_assoc_tys=vec![&ctokens.nope_ident;this.assoc_tys.len()];

            for AssocTyWithIndex{assoc_ty,index} in this.assoc_tys.values_mut() {
                let ident=parse_str_as_ident(&format!("{}_",assoc_ty.ident));
                assoc_ty.ident=ident.clone();
                renamed_assoc_tys[*index]=arenas.alloc(ident);
            }
        }
        
        let is_assoc_type=|ident:&Ident|{
            let index=self.assoc_tys.get(ident)?.index;
            if is_alt_trait {
                let new_ident=renamed_assoc_tys[index];
                Some(ReplaceWith::Ident(new_ident.clone()))
            }else{
                Some(ReplaceWith::Keep)
            }
        };

        for where_pred in &mut this.where_preds {
            replace_self_path::replace_self_path(where_pred,replace_with.clone(),is_assoc_type);
        }

        for assoc_ty in this.assoc_tys.values_mut() {
            replace_self_path::replace_self_path(
                &mut assoc_ty.assoc_ty,
                replace_with.clone(),
                is_assoc_type,
            );
        }

        for method in &mut this.methods {
            method.replace_self(replace_with.clone(),is_assoc_type);
        }
        this
    }


    pub fn generics_tokenizer(
        &self,
        in_what:InWhat,
        with_assoc_tys:WithAssocTys,
        after_lifetimes:&'a TokenStream2,
    )->GenericsTokenizer<'_> {
        let ctokens=self.ctokens;
        GenericsTokenizer{
            gen_params_in:
                GenParamsIn::with_after_lifetimes(self.generics,in_what,after_lifetimes),
            assoc_tys:match with_assoc_tys {
                WithAssocTys::Yes(WhichSelf::Regular)=>
                    Some((&self.assoc_tys,&ctokens.ts_self_colon2)),
                WithAssocTys::Yes(WhichSelf::Underscore)=>
                    Some((&self.assoc_tys,&ctokens.ts_uself_colon2)),
                WithAssocTys::Yes(WhichSelf::FullyQualified)=>
                    Some((&self.assoc_tys,self.ts_fq_self)),
                WithAssocTys::Yes(WhichSelf::NoSelf)=>
                    Some((&self.assoc_tys,&ctokens.empty_ts)),
                WithAssocTys::No =>None,
            },
        }
    }

    pub fn erased_ptr_preds(&self)->&'a TokenStream2{
        let ctokens=self.ctokens;
        match (self.has_mut_methods,self.has_val_methods) {
            (false,false)=> &ctokens.ptr_ref_bound,
            (false,true )=> &ctokens.ptr_ref_val_bound,
            (true ,false)=> &ctokens.ptr_mut_bound,
            (true ,true )=> &ctokens.ptr_mut_val_bound,
        }
    }

    pub fn trait_impl_where_preds(&self)->Punctuated<WherePredicate,Comma>{
        let mut where_preds=self.where_preds.clone();
        for where_pred in &mut where_preds {
            replace_self_path::replace_self_path(
                where_pred,
                ReplaceWith::Remove,
                |ident| self.assoc_tys.get(ident).map(|_| ReplaceWith::Keep )
            );
        }
        where_preds
    }

    pub fn methods_tokenizer(&self,which_item:WhichItem)->MethodsTokenizer<'_>{
        MethodsTokenizer{
            trait_def:self,
            which_item,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug,Clone)]
pub(crate) struct TraitMethod<'a>{
    /// The visibility of the trait
    pub(crate) unsafety: Option<&'a Unsafe>,
    pub(crate) abi: Option<&'a Abi>,
    pub(crate) vis:&'a Visibility,
    pub(crate) derive_attrs:&'a [Meta],
    pub(crate) other_attrs:&'a [Meta],
    pub(crate) name:&'a Ident,
    /// The name of this method in the __Method trait.
    pub(crate) name_method:&'a Ident,
    pub(crate) self_param:SelfParam<'a>,
    pub(crate) lifetimes: Vec<&'a LifetimeDef>,
    pub(crate) params: Vec<MethodParam<'a>>,
    pub(crate) output: Option<syn::Type>,
    pub(crate) where_clause:MethodWhereClause<'a>,
    pub(crate) default:Option<&'a Block>,
    pub(crate) semicolon:Option<&'a Semi>,
    pub(crate) ctokens:&'a CommonTokens,
}


#[derive(Debug,Clone,PartialEq,Eq)]
pub(crate) struct MethodParam<'a>{
    pub(crate) name:&'a Ident,
    pub(crate) ty:syn::Type,
}


impl<'a> TraitMethod<'a>{
    pub fn new(
        mwa:MethodWithAttrs<'a>,
        vis:&'a Visibility,
        ctokens:&'a CommonTokens,
        arena:&'a Arenas,
    )->Option<Self> {
        let method_signature=&mwa.item.sig;
        let decl=&method_signature.decl;

        let mut input_iter=decl.inputs.iter();

        let self_param=match input_iter.next()? {
            FnArg::SelfRef(ref_)=>
                SelfParam::ByRef{
                    lifetime:ref_.lifetime.as_ref(),
                    is_mutable:ref_.mutability.is_some(),
                },
            FnArg::SelfValue{..}=>
                SelfParam::ByVal,
            FnArg::Captured{..}|FnArg::Inferred{..}|FnArg::Ignored{..}=>
                return None,
        };

        let name_method=format!("{}_",method_signature.ident).as_str()
            .piped(parse_str_as_ident)
            .piped(|x| arena.alloc(x) );

        Some(Self{
            unsafety:method_signature.unsafety.as_ref(),
            abi:method_signature.abi.as_ref(),
            vis,
            derive_attrs:arena.alloc(mwa.attrs.derive_attrs),
            other_attrs:arena.alloc(mwa.attrs.other_attrs),
            name:&method_signature.ident,
            name_method,
            lifetimes:decl.generics.lifetimes().collect(),
            self_param,
            params:input_iter
                .enumerate()
                .map(|(param_i,param)|{
                    let ty=match param {
                        FnArg::SelfRef{..}|FnArg::SelfValue{..}|FnArg::Inferred{..}=>
                            unreachable!(),
                        FnArg::Captured(x)=>&x.ty,
                        FnArg::Ignored(ty)=>ty,
                    }.clone();

                    let name=format!("param_{}",param_i);
                    let name=syn::parse_str::<Ident>(&name).unwrap();
                    MethodParam{
                        name:arena.alloc(name),
                        ty,
                    }
                })
                .collect(),
            output:match &decl.output {
                syn::ReturnType::Default=>None,
                syn::ReturnType::Type(_,ty)=>Some((**ty).clone()),
            },
            where_clause:decl.generics.where_clause.as_ref()
                .map(|wc| MethodWhereClause::new(wc,ctokens) )
                .unwrap_or_default(),
            default:mwa.item.default.as_ref(),
            semicolon:mwa.item.semi_token.as_ref(),
            ctokens,
        })
    }


    pub fn replace_self<F>(&mut self,replace_with:ReplaceWith,mut is_assoc_type: F)
    where
        F: FnMut(&Ident) -> Option<ReplaceWith>,
    {
        for param in self.params.iter_mut()
            .map(|x| &mut x.ty )
            .chain(self.output.as_mut())
        {
            replace_self_path::replace_self_path(
                param,
                replace_with.clone(),
                &mut is_assoc_type
            )
        }
    }
}


////////////////////////////////////////////////////////////////////////////////


#[derive(Debug,Copy,Clone)]
pub struct GenericsTokenizer<'a>{
    gen_params_in:GenParamsIn<'a,&'a TokenStream2>,
    assoc_tys:Option<(&'a HashMap<&'a Ident,AssocTyWithIndex>,&'a TokenStream2)>,
}

impl<'a> GenericsTokenizer<'a>{
    pub fn set_no_bounds(&mut self){
        self.gen_params_in.set_no_bounds();
    }
}


impl<'a> ToTokens for GenericsTokenizer<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let with_bounds = 
            self.gen_params_in.with_bounds&&
            self.gen_params_in.in_what != InWhat::ItemUse;
        let with_default = self.gen_params_in.in_what == InWhat::ItemDecl;

        self.gen_params_in.to_tokens(ts);
        if let Some((assoc_tys,self_tokens))=self.assoc_tys {
            for with_index in assoc_tys.values() {
                self_tokens.to_tokens(ts);
                let assoc_ty=&with_index.assoc_ty;
                assoc_ty.ident.to_tokens(ts);

                match &assoc_ty.colon_token {
                    Some(colon_token)if with_bounds=>{
                        colon_token.to_tokens(ts);
                        assoc_ty.bounds.to_tokens(ts);
                    }
                    _=>{}
                }

                match &assoc_ty.default {
                    Some((eq_token,default_ty))if with_default=>{
                        eq_token.to_tokens(ts);
                        default_ty.to_tokens(ts);
                    }
                    _=>{}
                }

                Comma::default().to_tokens(ts);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

