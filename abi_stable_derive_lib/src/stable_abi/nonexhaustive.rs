use std::{
    collections::HashMap,
};

use core_extensions::SelfOps;

use syn::{
    Ident,
    visit_mut::VisitMut,
};

use quote::{quote,ToTokens};

use proc_macro2::{Span,TokenStream as TokenStream2};

use super::{
    attribute_parsing::{StabilityKind,StableAbiOptions},
};

use crate::{
    arenas::{Arenas,AllocMethods},
    common_tokens::CommonTokens,
    datastructure::{DataStructure},
    gen_params_in::{GenParamsIn,InWhat},
    impl_interfacetype::{private_associated_type,TRAIT_LIST,UsableTrait},
    parse_utils::{parse_str_as_ident,parse_str_as_path},
    set_span_visitor::SetSpanVisitor,
    to_token_fn::ToTokenFnMut,
};


#[derive(Clone, Default)]
pub(crate) struct UncheckedNonExhaustive<'a>{
    pub(crate) alignment:Option<IntOrType<'a>>,
    pub(crate) size:Option<IntOrType<'a>>,
    pub(crate) enum_interface:Option<EnumInterface<'a>>,
    pub(crate) assert_nonexh:Vec<&'a syn::Type>,
}


#[derive(Clone)]
pub(crate) struct NonExhaustive<'a>{
    pub(crate) nonexhaustive_alias:&'a Ident,
    pub(crate) nonexhaustive_marker:&'a Ident,
    pub(crate) enum_storage:&'a Ident,
    pub(crate) alignment:IntOrType<'a>,
    pub(crate) size:IntOrType<'a>,
    pub(crate) enum_interface:Option<EnumInterface<'a>>,
    pub(crate) new_interface:Option<&'a Ident>,
    pub(crate) default_interface:TokenStream2,
    pub(crate) assert_nonexh:Vec<&'a syn::Type>,
    pub(crate) bounds_trait:Option<BoundsTrait<'a>>,
    pub(crate) variant_constructor:Vec<Option<VariantConstructor<'a>>>,
}


#[derive(Clone)]
pub struct BoundsTrait<'a>{
    ident:&'a Ident,
    bounds:Vec<&'a syn::Path>,
}


#[derive(Clone)]
/// How a NonExhaustive<Enum,...> is constructed
pub enum UncheckedVariantConstructor{
    /// Constructs an enum variant using a function 
    /// with parameters of the same type as the fields.
    Regular,
    /// Constructs an enum variant containing a pointer,
    /// using a function taking the referent of the pointer.
    Boxed,
}

#[derive(Clone)]
/// How a NonExhaustive<Enum,...> is constructed
pub enum VariantConstructor<'a>{
    /// Constructs an enum variant using a function 
    /// with parameters of the same type as the fields.
    Regular,
    /// Constructs an enum variant containing a pointer,
    /// using a function taking the referent of the pointer.
    ///
    /// The type of the referent is extracted from the first type parameter 
    /// in the type of the only field of a variant.
    Boxed{
        referent:Option<&'a syn::Type>,
        pointer:&'a syn::Type,
    },
}


#[derive(Clone)]
pub(crate) enum EnumInterface<'a>{
    New(NewEnumInterface<'a>),
    Old(&'a syn::Type),
}

#[derive(Default,Clone)]
pub(crate) struct NewEnumInterface<'a>{
    pub(crate) impld:Vec<&'a Ident>,
    pub(crate) unimpld:Vec<&'a Ident>,
}


impl<'a> NonExhaustive<'a>{
    pub fn new(
        mut unchecked:UncheckedNonExhaustive<'a>,
        variant_constructor:Vec<Option<UncheckedVariantConstructor>>,
        ds: &'a DataStructure<'a>,
        arenas: &'a Arenas,
    )->Self{
        let name=ds.name;

        let alignment=unchecked.alignment
            .unwrap_or(IntOrType::Int(std::mem::size_of::<usize>()));
        
        let parse_ident=move|s:&str,span:Option<Span>|->&'a Ident{
            let mut ident=parse_str_as_ident(s);
            if let Some(span)=span{
                ident.set_span(span)
            }
            arenas.alloc(ident)
        };

        let size=unchecked.size.unwrap_or_else(||{
            panic!(
                "\n\
                You must specify the size of the enum storage in NonExhaustive<> using \
                the `size=integer literal` or `size=\"type\"` argument inside of \
                the `#[sabi(kind(WithNonExhaustive(...)))]` subattribute.\n\
                "
            );
        });

        let mut bounds_trait=None::<BoundsTrait<'a>>;

        if let Some(EnumInterface::New(enum_interface))=&mut unchecked.enum_interface{
            let mut trait_map=TRAIT_LIST.iter()
                .map(|x| ( parse_ident(x.name,None) , x ) )
                .collect::<HashMap<&'a syn::Ident,&'static UsableTrait>>();
            
            let mut bounds_trait_inner=Vec::<&'a syn::Path>::new();

            for &trait_ in &enum_interface.impld {
                match trait_map.remove(trait_) {
                    Some(ut) => {
                        use crate::impl_interfacetype::WhichTrait as WT;
                        if let WT::Deserialize=ut.which_trait {
                            continue;
                        } 
                        let mut full_path=parse_str_as_path(ut.full_path);

                        SetSpanVisitor::new(trait_.span())
                            .visit_path_mut(&mut full_path);

                        bounds_trait_inner.push(arenas.alloc(full_path));
                    }
                    None => panic!("Trait {} was not in TRAIT_LIST.",trait_),
                }
            }

            bounds_trait=Some(BoundsTrait{
                ident:parse_ident(&format!("{}_Bounds",name),None),
                bounds:bounds_trait_inner,
            });

            for &trait_ in &enum_interface.unimpld{
                if trait_map.remove(trait_).is_none(){
                    panic!("Trait {} was not in TRAIT_LIST.",trait_);
                }
            }

            for (trait_,_) in trait_map {
                enum_interface.unimpld.push(trait_);
            }
        }

        let (default_interface,new_interface)=match unchecked.enum_interface {
            Some(EnumInterface::New{..})=>{
                let name=parse_ident(&format!("{}_Interface",name),None);
                (name.into_token_stream(),Some(name))
            },
            Some(EnumInterface::Old(ty))=>{
                ((&ty).into_token_stream(),None)
            }
            None=>{
                (quote!(()),None)
            }
        };


        let variant_constructor=variant_constructor.into_iter()
            .zip(&ds.variants)
            .map(|(vc,variant)|->Option<VariantConstructor>{
                let vc=vc?;

                match vc {
                    UncheckedVariantConstructor::Regular=>{
                        VariantConstructor::Regular
                    }
                    UncheckedVariantConstructor::Boxed=>{
                        match variant.fields.first() {
                            Some(first_field) => 
                                VariantConstructor::Boxed{
                                    referent:extract_first_type_param(first_field.ty),
                                    pointer:first_field.ty,
                                },
                            None => 
                                VariantConstructor::Regular,
                        }                        
                    }
                }.piped(Some)
            })
            .collect();


        Self{
            nonexhaustive_alias:parse_ident(&format!("{}_NE",name),None),
            nonexhaustive_marker:parse_ident(&format!("{}_NEMarker",name),None),
            enum_storage:parse_ident(&format!("{}_Storage",name),None),
            alignment,
            size,
            enum_interface:unchecked.enum_interface,
            default_interface,
            new_interface,
            assert_nonexh:unchecked.assert_nonexh,
            bounds_trait,
            variant_constructor,
        }
    }
}


#[derive(Copy, Clone)]
pub enum IntOrType<'a>{
    Int(usize),
    Type(&'a syn::Type),
}


fn expr_from_int(int:u64)->syn::Expr{
    let call_site=proc_macro2::Span::call_site();
    let x=syn::LitInt::new(int,syn::IntSuffix::None,call_site);
    let x=syn::Lit::Int(x);
    let x=syn::ExprLit{attrs:Vec::new(),lit:x};
    let x=syn::Expr::Lit(x);
    x
}

fn extract_first_type_param(ty:&syn::Type)->Option<&syn::Type>{
    match ty {
        syn::Type::Path(path)=>{
            if path.qself.is_some() {
                return None;
            }
            let args=&path.path.segments.last()?.into_value().arguments;
            let args=match args {
                syn::PathArguments::AngleBracketed(x)=>x,
                _=>return None,
            };
            args.args
                .iter()
                .find_map(|arg|{
                    match arg {
                        syn::GenericArgument::Type(ty) => Some(ty),
                        _=>None,
                    }
                })
        }
        _=>None,
    }
}


pub(crate) fn tokenize_nonexhaustive_items<'a>(
    module:&'a Ident,
    ds:&'a DataStructure<'a>,
    config:&'a StableAbiOptions<'a>,
    _ct:&'a CommonTokens<'a>
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{

        let this=match &config.kind {
            StabilityKind::NonExhaustive(x)=>x,
            _=>return,
        };
        let vis=ds.vis;
        let nonexhaustive_alias=this.nonexhaustive_alias;
        let nonexhaustive_marker=this.nonexhaustive_marker;
        let enum_storage=this.enum_storage;

        let (aligner_attribute,aligner_field)=match this.alignment {
            IntOrType::Int(bytes)=>{
                let bytes=expr_from_int(bytes as _);
                ( Some(quote!(#[repr(align(#bytes))])) , None )
            }
            IntOrType::Type(ty)=>
                ( None , Some(quote!(__aligner:[#ty;0],)) ),
        };

        let aligner_size=match this.size {
            IntOrType::Int(size)=>quote!( #size ),
            IntOrType::Type(ty)=>quote!( std::mem::size_of::<#ty>() ),
        };

        let name=ds.name;

        let generics_header=
            GenParamsIn::new(&ds.generics,InWhat::ImplHeader);

        let mut type_generics_decl=GenParamsIn::new(ds.generics,InWhat::ImplHeader);
        type_generics_decl.set_no_bounds();

        let type_generics_use=GenParamsIn::new(ds.generics,InWhat::ItemUse);

        let storage_docs=format!(
            "The InlineStorage for the NonExhaustive wrapped version of `{}<_>`.",
            name
        );

        let alias_docs=format!(
            "An alias for the NonExhaustive wrapped version of `{}<_>`.",
            name
        );

        let marker_docs=format!(
            "A marker type which implements StableAbi with the layout of {},\
             used as a phantom field of NonExhaustive.",
            name
        );

        let default_interface=&this.default_interface;

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
                #module::_sabi_reexports::NonExhaustive<
                    #name<#type_generics_use>,
                    #enum_storage,
                    #default_interface,
                >;

            unsafe impl #module::_sabi_reexports::InlineStorage for #enum_storage{}

            #[doc=#marker_docs]
            #vis struct #nonexhaustive_marker<T,S>(
                std::marker::PhantomData<T>,
                std::marker::PhantomData<S>,
            );
        ).to_tokens(ts);


        if let Some(BoundsTrait{ident,bounds})=&this.bounds_trait{
            let trait_docs=format!(
                "Acts as an alias for the traits that \
                 were specified for `{}` in `traits(...)`.",
                name
            );
            quote!( 
                #[doc=#trait_docs]
                #vis trait #ident:#(#bounds+)*{}

                impl<This> #ident for This
                where
                    This:#(#bounds+)*
                {}
            ).to_tokens(ts);
        }

        if let Some(new_interface)=this.new_interface{
            let interface_docs=format!(
                "Describes the traits required when constructing a \
                 `NonExhaustive<>` from `{}`,and are usable with it,\
                 by implementing `InterfaceType`.",
                name
            );

            quote!( 
                #[doc=#interface_docs]
                #[repr(C)]
                #[derive(::abi_stable::StableAbi)]
                #vis struct #new_interface;
            ).to_tokens(ts);
        }

        if this.variant_constructor.iter().any(|x| x.is_some() ) {
            let constructors=this.variant_constructor
                .iter()
                .cloned()
                .zip(&ds.variants)
                .filter_map(|(vc,variant)|{
                    let vc=vc?;
                    let variant_ident=variant.name;
                    let mut method_name=parse_str_as_ident(&format!("{}_NE",variant.name));
                    method_name.set_span(variant.name.span());

                    match vc {
                        VariantConstructor::Regular=>{
                            let field_names_a=variant.fields.iter().map(|x|x.ident());
                            let field_names_b=field_names_a.clone();
                            let field_names_c=variant.fields.iter().map(|x|&x.ident);
                            let field_types=variant.fields.iter().map(|x|x.ty);
                            quote!{
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
                        VariantConstructor::Boxed{referent,pointer}=>{
                            let ptr_field_ident=&variant.fields[0].ident;
                            let type_param=ToTokenFnMut::new(|ts|{
                                match referent {
                                    Some(x) => x.to_tokens(ts),
                                    None => 
                                        quote!( <#pointer as ::std::ops::Deref>::Target )
                                            .to_tokens(ts),
                                }
                            });
                            
                            quote!{
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
                    }.piped(Some)
                });

            let preds=ds.generics.where_clause.as_ref().map(|w| &w.predicates );

            let bound=match &this.bounds_trait {
                Some(BoundsTrait{ident,..}) => 
                    quote!(#ident),
                None => 
                    quote!( 
                        #module::_sabi_reexports::GetNonExhaustiveVTable<
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
            ).to_tokens(ts);
        }

    })
}


pub(crate) fn tokenize_enum_info<'a>(
    ds:&'a DataStructure<'a>,
    config:&'a StableAbiOptions<'a>,
    ct:&'a CommonTokens<'a>
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{
        let this=match &config.kind {
            StabilityKind::NonExhaustive(x)=>x,
            _=>return,
        };

        let name=ds.name;
        let name_str=ds.name.to_string();

        let variant_names=ds.variants.iter().map(|x| x.name.to_string() );

        let discriminants=ds.variants.iter().map(|x|x.discriminant)
            .collect::<Vec<Option<&'a syn::Expr>>>();

        let discriminant_tokens=config.repr
            .tokenize_discriminant_slice(discriminants.iter().cloned(),ct);

        let discriminant_type=match config.repr.type_ident() {
            Some(x)=>x,
            None=>panic!(
                "Attempted to get type of discriminant for this representation:\n\t{:?}",
                config.repr
            )
        };

        let nonexhaustive_marker=this.nonexhaustive_marker;
        let enum_storage=this.enum_storage;

        let mut start_discrs=Vec::new();
        let mut end_discrs=Vec::new();
        if !discriminants.is_empty() {
            let mut first_index=0;
            
            for (mut i,discr) in discriminants[1..].iter().cloned().enumerate() {
                i+=1;
                if discr.is_some() {
                    start_discrs.push(first_index);
                    end_discrs.push(i-1);
                    first_index=i;
                }
            }

            start_discrs.push(first_index);
            end_discrs.push(discriminants.len()-1);
        }


        let generics_header=
            GenParamsIn::with_after_types(&ds.generics,InWhat::ImplHeader,&ct.und_storage);
        
        let generics_use=GenParamsIn::new(&ds.generics,InWhat::ImplHeader);
        
        let default_interface=&this.default_interface;

        let (impl_generics, ty_generics, where_clause) = ds.generics.split_for_impl();

        let preds=where_clause.as_ref().map(|w| &w.predicates );

        quote!(

            unsafe impl #impl_generics _sabi_reexports::GetStaticEquivalent_ for #name #ty_generics 
            where
                #nonexhaustive_marker <Self,#enum_storage> :
                    _sabi_reexports::GetStaticEquivalent_,
                #preds
            {
                type StaticEquivalent=_sabi_reexports::GetStaticEquivalent<
                    #nonexhaustive_marker <Self,#enum_storage>
                >;
            }

            impl #impl_generics #name #ty_generics 
            #where_clause
            {
                const _SABI_NE_DISCR_CNSNT_:&'static [#discriminant_type]=
                    #discriminant_tokens;
            }

            unsafe impl #impl_generics _sabi_reexports::GetEnumInfo for #name #ty_generics 
            #where_clause
            {
                type Discriminant=#discriminant_type;

                type DefaultStorage=#enum_storage;
                
                type DefaultInterface=#default_interface;

                const ENUM_INFO:&'static _sabi_reexports::EnumInfo=
                    &_sabi_reexports::EnumInfo::_for_derive(
                        #name_str,
                        &[ #( __StaticStr::new( #variant_names ), )* ],
                    );

                fn discriminants()->&'static [#discriminant_type]{
                    Self::_SABI_NE_DISCR_CNSNT_
                }

                fn is_valid_discriminant(discriminant:#discriminant_type)->bool{
                    #( 
                        ( 
                            Self::_SABI_NE_DISCR_CNSNT_[#start_discrs] <= discriminant && 
                            discriminant <= Self::_SABI_NE_DISCR_CNSNT_[#end_discrs]
                        )|| 
                    )*
                    false
                }
            }


            unsafe impl<#generics_header> 
                _sabi_reexports::GetNonExhaustive<__Storage> 
            for #name <#generics_use>
            #where_clause
            {
                type NonExhaustive=#nonexhaustive_marker<Self,__Storage>;
            }


        ).to_tokens(ts);

        if !this.assert_nonexh.is_empty() {
            let tests_function=parse_str_as_ident(&format!("{}_storage_assertions",name));
            let assertionsa=this.assert_nonexh.iter().cloned();
            let assertionsb=this.assert_nonexh.iter().cloned();
            quote!(
                #[test]
                fn #tests_function(){
                    use self::_sabi_reexports::assert_nonexhaustive;

                    let storage_str=stringify!(#enum_storage);
                    #(
                        assert_nonexhaustive::<#assertionsa>(
                            stringify!(#assertionsb),
                            storage_str
                        );
                    )*
                }
            ).to_tokens(ts);
        }

        match &this.enum_interface {
            Some(EnumInterface::New(NewEnumInterface{impld,unimpld}))=>{
                let enum_interface=parse_str_as_ident(&format!("{}_Interface",name));
                
                let priv_assocty=private_associated_type();

                let impld_a=impld.iter();
                let impld_b=impld.iter();

                let unimpld_a=unimpld.iter();
                let unimpld_b=unimpld.iter();

                let const_ident=parse_str_as_ident(&format!(
                    "_impl_InterfaceType_constant_{}",
                    name,
                ));

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
                ).to_tokens(ts);
            }
            Some(EnumInterface::Old{..})=>{}
            None=>{}
        }
    })
}