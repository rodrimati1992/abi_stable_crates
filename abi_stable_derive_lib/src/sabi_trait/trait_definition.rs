use super::{
    *,
    attribute_parsing::{MethodWithAttrs,OwnedDeriveAndOtherAttrs},
    impl_interfacetype::{TRAIT_LIST,TraitStruct,WhichTrait},
    replace_self_path::{self,ReplaceWith},
    parse_utils::{parse_str_as_ident,parse_str_as_trait_bound},
};

use crate::{
    set_span_visitor::SetSpanVisitor,
};

use std::{
    collections::{HashMap,HashSet},
    iter,
};

use core_extensions::{matches,IteratorExt};

use syn::{
    token::{Comma,Colon,Semi},
    punctuated::Punctuated,
    Ident,ItemTrait,Visibility,FnArg,Lifetime,LifetimeDef,Meta,
    TypeParamBound,Block,WherePredicate,TraitItem,Abi,
    token::Unsafe,
    spanned::Spanned,
    visit_mut::VisitMut,
};

use proc_macro2::Span;



#[derive(Debug,Clone)]
pub struct AssocTyWithIndex{
    pub index:usize,
    pub assoc_ty:syn::TraitItemType,
}


////////////////////////////////////////////////////////////////////////////////


/// Represents a trait for use in `#[sabi_trait]`.
#[derive(Debug,Clone)]
pub(crate) struct TraitDefinition<'a>{
    pub(crate) item:&'a ItemTrait,
    /// The name of the trait.
    pub(crate) name:&'a Ident,
    /// What type to use the backend for the trait object,DynTrait or RObject.
    pub(crate) which_object:WhichObject,
    /// The where predicates in the where clause of the trait,
    /// if it doesn't have one this is empty.
    pub(crate) where_preds:Punctuated<WherePredicate,Comma>,
    /// Attributes applied to the vtable.
    pub(crate) derive_attrs:&'a [Meta],
    /// Attributes applied to the trait.
    pub(crate) other_attrs:&'a [Meta],
    pub(crate) generics:&'a syn::Generics,
    /// The `Iterator::Item` type for this trait,
    /// None if it doesn't have Iterator as a supertrait.
    pub(crate) iterator_item:Option<&'a syn::Type>,
    #[allow(dead_code)]
    /// The path for the implemented serde::Deserialize trait 
    /// (it may reference some trait lifetime parameter)
    pub(crate) deserialize_bound:Option<DeserializeBound<'a>>,
    /// The traits this has as supertraits.
    pub(crate) impld_traits:Vec<TraitImplness<'a>>,
    /// The traits this doesn't have as supertraits.
    pub(crate) unimpld_traits:Vec<&'a Ident>,
    /// A struct describing the traits this does and doesn't have as supertraits 
    /// (true means implemented,false means unimplemented)
    pub(crate) trait_flags:TraitStruct<bool>,
    /// The region of code of the identifiers for the supertraits,
    /// with `Span::call_site()` for the ones that aren't supertraits.
    pub(crate) trait_spans:TraitStruct<Span>,
    /// The lifetimes declared in the trait generic parameter list that are used in 
    /// `&'lifetime self` `&'lifetime mut self` method receivers,
    /// or used directly as supertraits.
    pub(crate) lifetime_bounds:Vec<&'a Lifetime>,
    /// The visibility of the trait.
    pub(crate) vis:VisibilityKind<'a>,
    /// The visibility of the trait,inside a submodule.
    pub(crate) submod_vis:RelativeVis<'a>,
    // The keys use the proginal identifier for the associated type.
    pub(crate) assoc_tys:HashMap<&'a Ident, AssocTyWithIndex>,
    /// 
    pub(crate) methods:Vec<TraitMethod<'a>>,
    /// Whether this has by mutable reference methods.
    pub(crate) has_mut_methods:bool,
    /// Whether this has by-value methods.
    pub(crate) has_val_methods:bool,
    /// A TokenStream with the equivalent of `<Pointer::Target as Trait>::`
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
        which_object:WhichObject,
        arenas: &'a Arenas,
        ctokens:&'a CommonTokens,
    )->Self {
        let vis=VisibilityKind::new(&trait_.vis);
        let submod_vis=vis.submodule_level(1);
        let mut assoc_tys=HashMap::default();
        let mut methods=Vec::<TraitMethod<'a>>::new();

        methods_with_attrs.into_iter()
            .filter_map(|func| TraitMethod::new(func,&trait_.vis,ctokens,arenas) )
            .extending(&mut methods);

        /////////////////////////////////////////////////////
        ////         Processing the supertrait bounds

        let lifetime_params:HashSet<&'a Lifetime>=
            trait_.generics.lifetimes()
                .map(|l| &l.lifetime )
                .chain(iter::once(&ctokens.static_lifetime))
                .collect();

        let GetSupertraits{
            impld_traits,
            unimpld_traits,
            mut lifetime_bounds,
            iterator_item,
            deserialize_bound,
            trait_flags,
            trait_spans,
        }=get_supertraits(
            &trait_.supertraits,
            &lifetime_params,
            which_object,
            arenas,
            ctokens,
        );

        // Adding the lifetime parameters in `&'a self` and `&'a mut self` 
        // that were declared in the trait generic parameter list.
        // This is done because those lifetime bounds are enforced as soon as 
        // the vtable is created,instead of when the methods are called
        // (it's enforced in method calls in regular trait objects).
        for method in &methods {
            if let SelfParam::ByRef{lifetime:Some(lt),..}=method.self_param {
                if lifetime_params.contains(lt) {
                    lifetime_bounds.push(lt);
                }
            }
        }


        /////////////////////////////////////////////////////

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
            which_object,
            where_preds:trait_.generics.where_clause.as_ref()
                .map(|wc| wc.predicates.clone() )
                .unwrap_or_default(),
            derive_attrs:arenas.alloc(attrs.derive_attrs),
            other_attrs:arenas.alloc(attrs.other_attrs),
            generics:&trait_.generics,
            lifetime_bounds,
            iterator_item,
            deserialize_bound,
            impld_traits,
            unimpld_traits,
            trait_flags,
            trait_spans,
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

    /// Returns a clone of `self`,
    /// where usages of associated types are replaced for use in `which_item`.
    pub fn replace_self(&self,which_item:WhichItem)->Self{
        let mut this=self.clone();

        let ctokens=self.ctokens;

        let replace_with=match which_item {
            WhichItem::Trait|WhichItem::TraitImpl=>{
                return this;
            }
            WhichItem::TraitObjectImpl=>{
                ReplaceWith::Remove
            }
            WhichItem::VtableDecl=>{
                ReplaceWith::Remove
            }
            WhichItem::VtableImpl=>{
                ReplaceWith::Ident(ctokens.u_capself.clone())
            }
        };

        let is_assoc_type=|ident:&Ident|{
            if self.assoc_tys.contains_key(ident) {
                Some(ReplaceWith::Keep)
            }else{
                None
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

    /// Returns a tokenizer for the generic parameters in this trait.
    ///
    /// # Parameters
    /// 
    /// - `in_what`:
    ///     Determines where the generic parameters are printed.
    ///     Eg:impl headers,trait declaration,trait usage.
    ///
    /// - `with_assoc_tys`:
    ///     Whether associated types are printed,and how.
    /// 
    /// - `after_lifetimes`:
    ///     What will be printed after lifetime parameters.
    /// 
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

    /// Returns the where predicates for the erased pointer type of the ffi-safe trait object.
    ///
    /// Example erased pointer types:`RBox<()>`,`RArc<()>`,`&()`,`&mut ()`
    ///
    pub fn erased_ptr_preds(&self)->&'a TokenStream2{
        let ctokens=self.ctokens;
        match (self.has_mut_methods,self.has_val_methods) {
            (false,false)=> &ctokens.ptr_ref_bound,
            (false,true )=> &ctokens.ptr_ref_val_bound,
            (true ,false)=> &ctokens.ptr_mut_bound,
            (true ,true )=> &ctokens.ptr_mut_val_bound,
        }
    }

    /// Returns the where predicates of the inherent implementation of 
    /// the ffi-safe trait object.
    pub fn trait_impl_where_preds(&self)->Punctuated<WherePredicate,Comma>{
        let mut where_preds=self.where_preds.clone();
        for where_pred in &mut where_preds {
            replace_self_path::replace_self_path(
                where_pred,
                ReplaceWith::Remove,
                |ident| self.assoc_tys.get(ident).map(|_| ReplaceWith::Remove )
            );
        }
        where_preds
    }

    /// Returns a tokenizer that outputs the method definitions inside the `which_item` item.
    pub fn methods_tokenizer(&self,which_item:WhichItem)->MethodsTokenizer<'_>{
        MethodsTokenizer{
            trait_def:self,
            which_item,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////


/// Represents a trait method for use in `#[sabi_trait]`.
#[derive(Debug,Clone)]
pub(crate) struct TraitMethod<'a>{
    /// A refernce to the `syn` type this was derived from.
    pub(crate) item:&'a syn::TraitItemMethod,
    pub(crate) unsafety: Option<&'a Unsafe>,
    pub(crate) abi: Option<&'a Abi>,
    /// The visibility of the trait
    pub(crate) vis:&'a Visibility,
    /// Attributes applied to the method in the vtable.
    pub(crate) derive_attrs:&'a [Meta],
    /// Attributes applied to the method in the trait definition.
    pub(crate) other_attrs:&'a [Meta],
    /// The name of the method.
    pub(crate) name:&'a Ident,
    pub(crate) self_param:SelfParam<'a>,
    /// The lifetime parameters of this method.
    pub(crate) lifetimes: Vec<&'a LifetimeDef>,
    pub(crate) params: Vec<MethodParam<'a>>,
    /// The return type of this method,if None this returns `()`.
    pub(crate) output: Option<syn::Type>,
    pub(crate) where_clause:MethodWhereClause<'a>,
    /// The default implementation of the method.
    pub(crate) default:Option<DefaultMethod<'a>>,
    /// The semicolon token for the method
    /// (when the method did not have a default implementation).
    pub(crate) semicolon:Option<&'a Semi>,
    pub(crate) ctokens:&'a CommonTokens,
}


#[derive(Debug,Clone)]
pub(crate) struct DefaultMethod<'a>{
    pub(crate) block:&'a Block,
}


#[derive(Debug,Clone,PartialEq,Eq)]
pub(crate) struct MethodParam<'a>{
    /// The name of the method parameter,
    /// which is `param_<number_of_parameter>` if the parameter is not just an identifier
    /// (ie:`(left,right)`,`Rect3D{x,y,z}`)
    pub(crate) name:&'a Ident,
    /// The parameter type.
    pub(crate) ty:syn::Type,
    /// The pattern for the parameter
    pub(crate) pattern:&'a syn::Pat,
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
        let name=&method_signature.ident;

        let panic_msg=||{
            panic!("\n\n\
                Cannot define #[sabi_trait]traits containing methods \
                without a `self`/`&self`/`&mut self` receiver \
                (static methods).\n\
                Caused by the '{}' method.\n\n\
            ",
                method_signature.ident,
            )
        };
        if decl.inputs.is_empty() {
            panic_msg();
        }

        let mut input_iter=decl.inputs.iter();

        let mut self_param=match input_iter.next()? {
            FnArg::SelfRef(ref_)=>
                SelfParam::ByRef{
                    lifetime:ref_.lifetime.as_ref(),
                    is_mutable:ref_.mutability.is_some(),
                },
            FnArg::SelfValue{..}=>
                SelfParam::ByVal,
            FnArg::Captured{..}|FnArg::Inferred{..}|FnArg::Ignored{..}=>
                panic_msg(),
        };

        let mut lifetimes:Vec<&'a syn::LifetimeDef>=decl.generics.lifetimes().collect();

        let output=match &decl.output {
            syn::ReturnType::Default=>None,
            syn::ReturnType::Type(_,ty)=>{
                let mut ty=(**ty).clone();
                if let SelfParam::ByRef{lifetime,..}=&mut self_param {
                    LifetimeUnelider::new(ctokens,lifetime)
                        .visit_type(&mut ty)
                        .into_iter()
                        .extending(&mut lifetimes);
                }
                Some(ty)
            },
        };

        let default=mwa.item.default.as_ref().map(|block| DefaultMethod{block} );

        Some(Self{
            item:&mwa.item,
            unsafety:method_signature.unsafety.as_ref(),
            abi:method_signature.abi.as_ref(),
            vis,
            derive_attrs:arena.alloc(mwa.attrs.derive_attrs),
            other_attrs:arena.alloc(mwa.attrs.other_attrs),
            name,
            lifetimes,
            self_param,
            params:input_iter
                .enumerate()
                .map(|(param_i,param)|{
                    let (pattern,ty)=match param {
                        FnArg::SelfRef{..}|FnArg::SelfValue{..}|FnArg::Inferred{..}=>
                            unreachable!(),
                        FnArg::Captured(x)=>{
                            (&x.pat,&x.ty)
                        },
                        FnArg::Ignored(ty)=>
                            (&ctokens.ignored_pat,ty),
                    };

                    let name=format!("param_{}",param_i);
                    let mut name=syn::parse_str::<Ident>(&name).unwrap();
                    name.set_span(param.span());
                    MethodParam{
                        name:arena.alloc(name),
                        ty:ty.clone(),
                        pattern,
                    }
                })
                .collect(),
            output,
            where_clause:decl.generics.where_clause.as_ref()
                .map(|wc| MethodWhereClause::new(wc,ctokens) )
                .unwrap_or_default(),
            default,
            semicolon:mwa.item.semi_token.as_ref(),
            ctokens,
        })
    }

    /// Returns a clone of `self`,
    /// where usages of associated types are replaced for use in `which_item`.
    ///
    /// Whether `Self::AssocTy` is an associated type is determined using `is_assoc_type`,
    /// which returns `Some()` with what to do with the associated type.
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
            );
        }
    }
}


////////////////////////////////////////////////////////////////////////////////


/// Used to print the generic parameters of a trait,
/// potentially including its associated types.
#[derive(Debug,Copy,Clone)]
pub struct GenericsTokenizer<'a>{
    gen_params_in:GenParamsIn<'a,&'a TokenStream2>,
    assoc_tys:Option<(&'a HashMap<&'a Ident,AssocTyWithIndex>,&'a TokenStream2)>,
}

impl<'a> GenericsTokenizer<'a>{
    /// Changes type parameters to have a `?Sized` bound.
    #[allow(dead_code)]
    pub fn set_unsized_types(&mut self){
        self.gen_params_in.set_unsized_types();
    }
    /// Removes bounds on type parameters.
    pub fn set_no_bounds(&mut self){
        self.gen_params_in.set_no_bounds();
    }
}


impl<'a> ToTokens for GenericsTokenizer<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let with_bounds = self.gen_params_in.outputs_bounds();
        let with_default = self.gen_params_in.in_what == InWhat::ItemDecl;

        let unsized_types=self.gen_params_in.are_types_unsized();

        let in_dummy_struct= self.gen_params_in.in_what == InWhat::DummyStruct;

        self.gen_params_in.to_tokens(ts);
        if let Some((assoc_tys,self_tokens))=self.assoc_tys {
            for with_index in assoc_tys.values() {
                self_tokens.to_tokens(ts);
                let assoc_ty=&with_index.assoc_ty;

                if in_dummy_struct {
                    use syn::token::{Star,Const};
                    Star::default().to_tokens(ts);
                    Const::default().to_tokens(ts);
                }

                assoc_ty.ident.to_tokens(ts);

                let colon_token=assoc_ty.colon_token.filter(|_| with_bounds );

                if unsized_types {
                    if colon_token.is_none() {
                        Colon::default().to_tokens(ts);
                    }
                    quote!(?Sized+).to_tokens(ts);
                }
                if let Some(colon_token)=colon_token {
                    colon_token.to_tokens(ts);
                    assoc_ty.bounds.to_tokens(ts);
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


/// Represents a `Deserialize<'de>` supertrait bound.
#[derive(Debug,Clone)]
pub(crate) struct DeserializeBound<'a>{
    pub(crate) bound:&'a syn::TraitBound,
    pub(crate) lifetime:&'a syn::Lifetime,
}

/// Used to returns the information about supertraits,to construct TraitDefinition.
struct GetSupertraits<'a>{
    impld_traits:Vec<TraitImplness<'a>>,
    unimpld_traits:Vec<&'a Ident>,
    lifetime_bounds:Vec<&'a Lifetime>,
    iterator_item:Option<&'a syn::Type>,
    deserialize_bound:Option<DeserializeBound<'a>>,
    trait_flags:TraitStruct<bool>,
    trait_spans:TraitStruct<Span>,
}

/// Contains information about a supertrait,including whether it's implemented.
#[derive(Debug,Clone)]
pub(crate) struct TraitImplness<'a>{
    pub(crate) which_trait:WhichTrait,
    pub(crate) name:&'static str,
    pub(crate) ident:Ident,
    pub(crate) bound:syn::TraitBound,
    pub(crate) is_implemented:bool,
    pub(crate) _marker:PhantomData<&'a ()>,
}

/// Processes the supertrait bounds of a trait definition.
fn get_supertraits<'a,I>(
    supertraits: I,
    lifetime_params:&HashSet<&'a Lifetime>,
    which_object:WhichObject,
    arenas: &'a Arenas,
    _ctokens:&'a CommonTokens,
)->GetSupertraits<'a>
where
    I:IntoIterator<Item=&'a TypeParamBound>
{
    let trait_map=TRAIT_LIST.iter()
        .map(|t| (parse_str_as_ident(t.name),t.which_trait) )
        .collect::<HashMap<Ident,WhichTrait>>();

    // A struct indexable by `WhichTrait`,
    // with information about all possible supertraits.
    let mut trait_struct=TraitStruct::TRAITS.map(|_,t|{
        TraitImplness{
            which_trait:t.which_trait,
            name:t.name,
            ident:parse_str_as_ident(t.name),
            bound:parse_str_as_trait_bound(t.full_path),
            is_implemented:false,
            _marker:PhantomData,
        }
    });

    let mut lifetime_bounds=Vec::new();
    let mut iterator_item=None;
    let deserialize_bound=None;

    for supertrait_bound in supertraits{
        match supertrait_bound {
            TypeParamBound::Trait(trait_bound)=>{
                let last_path_component=match trait_bound.path.segments.last() {
                    Some(x)=>x.into_value(),
                    None=>continue,
                };
                let trait_ident=&last_path_component.ident;

                match trait_map.get(&trait_ident) {
                    Some(&which_trait)=>{
                        let usable_by=which_trait.usable_by();
                        match which_object {
                            WhichObject::DynTrait if !usable_by.dyn_trait() => {
                                panic!(
                                    "Cannot use this trait with DynTrait:{}",
                                    (&trait_bound.path).into_token_stream()
                                );
                            },
                            WhichObject::RObject if !usable_by.robject() => {
                                panic!(
                                    "Cannot use this trait with RObject:\n\
                                     \t{}\n\
                                     To make that trait usable you must use the \
                                     #[sabi(use_dyntrait)] attribute,\
                                     which changes the trait object implementation \
                                     from using RObject to using DynTrait.\n\
                                    ",
                                    (&trait_bound.path).into_token_stream()
                                );
                            },
                            WhichObject::DynTrait|WhichObject::RObject => {}
                        }

                        fn set_impld<'a>(
                            wtrait:&mut TraitImplness<'a>,
                            span:Span,
                        ){
                            wtrait.is_implemented=true;
                            wtrait.ident.set_span(span);
                            SetSpanVisitor::new(span)
                                .visit_trait_bound_mut(&mut wtrait.bound);
                        }

                        let span=trait_bound.span();

                        set_impld(&mut trait_struct[which_trait],span);


                        match which_trait {
                            WhichTrait::Iterator|WhichTrait::DoubleEndedIterator=>{
                                set_impld(&mut trait_struct.iterator,span);

                                let iter_item=extract_iterator_item(last_path_component,arenas);
                                iterator_item=iterator_item.or(iter_item);
                            }
                            WhichTrait::Deserialize=>{
                                // deserialize_bound=deserialize_bound.or(Some(
                                //     DeserializeBound{
                                //         bound:trait_bound
                                //             .clone()
                                //             .piped(|x| arenas.alloc(x) ),
                                //         lifetime:
                                //             extract_deserialize_lifetime(
                                //                 last_path_component,
                                //                 arenas
                                //             ),
                                //     }
                                // ));
                                panic!("Deserialize is not currently supported.");
                            }
                            WhichTrait::Serialize=>{
                                panic!("Serialize is not currently supported.");
                            }
                            WhichTrait::Eq|WhichTrait::PartialOrd=>{
                                set_impld(&mut trait_struct.partial_eq,span);
                            }
                            WhichTrait::Ord=>{
                                set_impld(&mut trait_struct.partial_eq,span);
                                set_impld(&mut trait_struct.eq,span);
                                set_impld(&mut trait_struct.partial_ord,span);
                            }
                            WhichTrait::IoBufRead=>{
                                set_impld(&mut trait_struct.io_read,span);
                            }
                            WhichTrait::Error=>{
                                set_impld(&mut trait_struct.display,span);
                                set_impld(&mut trait_struct.debug,span);
                            }
                            _=>{}
                        }
                    },
                    None=>{
                        let list=trait_map.keys()
                            .map(|x| x.to_string() )
                            .collect::<Vec<String>>();

                        panic!(
                            "Unexpected supertrait bound:\n\t{}\nExpected one of:\n{}\n", 
                            supertrait_bound.into_token_stream(),
                            list.join("/"),
                        );
                    },
                }
            }
            TypeParamBound::Lifetime(lt)=>{
                if lifetime_params.contains(lt) {
                    lifetime_bounds.push(lt);
                }else{
                    panic!(
                        "\nLifetimes is not from the trait or `'static`:\n\t{}\n\n",
                        lt.into_token_stream(),
                    );
                }
            }
        };
    }


    let iter_trait=&mut trait_struct.iterator;
    let de_iter_trait=&mut trait_struct.double_ended_iterator;
    if iter_trait.is_implemented||de_iter_trait.is_implemented {
        let iter_item:syn::Type=iterator_item.cloned()
            .unwrap_or_else(||{
                panic!(
                    "You must specify the Iterator item type,with `{}<Item= SomeType >` .",
                    if de_iter_trait.is_implemented { "DoubleEndedÌterator" }else{ "Ìterator" }
                );
            });
        let path_args=type_as_iter_path_arguments(iter_item);

        fn set_last_arguments(bounds:&mut syn::TraitBound,path_args:syn::PathArguments){
            bounds.path.segments.last_mut().unwrap().value_mut().arguments=path_args;
        }

        if de_iter_trait.is_implemented{
            set_last_arguments(&mut de_iter_trait.bound,path_args.clone());
        }
        set_last_arguments(&mut iter_trait.bound,path_args);
    }


    let mut impld_traits=Vec::new();
    let mut unimpld_traits=Vec::new();
    let trait_flags=trait_struct.as_ref().map(|_,x| x.is_implemented );
    let trait_spans=trait_struct.as_ref().map(|_,x| x.ident.span() );

    for trait_ in trait_struct.to_vec() {
        if trait_.is_implemented {
            impld_traits.push(trait_);
        }else{
            unimpld_traits.push(arenas.alloc(trait_.ident.clone()));
        }
    }


    GetSupertraits{
        impld_traits,
        unimpld_traits,
        lifetime_bounds,
        iterator_item,
        deserialize_bound,
        trait_flags,
        trait_spans,
    }
}


////////////////////////////////////////////////////////////////////////////////


/// Extracts the Iterator::Item out of a path component.
fn extract_iterator_item<'a>(
    last_path_component:&syn::PathSegment,
    arenas:&'a Arenas,
)->Option<&'a syn::Type>{
    use syn::{GenericArgument,PathArguments};

    let angle_brackets=match &last_path_component.arguments {
        PathArguments::AngleBracketed(x)=>x,
        _=>return None
    };

    for gen_arg in &angle_brackets.args {
        match gen_arg {
            GenericArgument::Binding(bind) if bind.ident=="Item" =>{
                return Some(arenas.alloc(bind.ty.clone()));
            }
            _=>{}
        }
    }
    None
}


/// Converts a type to `<Item= ty >`.
fn type_as_iter_path_arguments(ty:syn::Type)->syn::PathArguments{
    let x=syn::Binding{
        ident: parse_str_as_ident("Item"),
        eq_token: Default::default(),
        ty,
    };
    
    let x=syn::GenericArgument::Binding(x);

    let x=syn::AngleBracketedGenericArguments{
        colon2_token: None,
        lt_token: Default::default(),
        args: iter::once(x).collect(),
        gt_token: Default::default(),
    };

    syn::PathArguments::AngleBracketed(x)
}


/// Extracts the lifetime in `Deserialize<'lt>` out of a path component.
#[allow(dead_code)]
fn extract_deserialize_lifetime<'a>(
    last_path_component:&syn::PathSegment,
    arenas:&'a Arenas,
)->&'a syn::Lifetime{
    use syn::{GenericArgument,PathArguments};

    let angle_brackets=match &last_path_component.arguments {
        PathArguments::AngleBracketed(x)=>x,
        _=>panic!(
            "Expected a lifetime parameter inside '{}'",
            last_path_component.into_token_stream(),
        )
    };

    for gen_arg in &angle_brackets.args {
        match gen_arg {
            GenericArgument::Lifetime(lt) =>{
                return arenas.alloc(lt.clone());
            }
            _=>{}
        }
    }
    panic!(
        "Expected a lifetime parameter inside '{}'",
        last_path_component.into_token_stream(),
    )
}