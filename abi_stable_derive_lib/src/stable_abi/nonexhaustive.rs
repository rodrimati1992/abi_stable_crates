use std::{
    collections::HashMap,
};

use syn::Ident;

use quote::{quote,ToTokens};

use proc_macro2::TokenStream as TokenStream2;

use super::{
    attribute_parsing::{StabilityKind,StableAbiOptions},
};

use crate::{
    arenas::{Arenas,AllocMethods},
    common_tokens::CommonTokens,
    datastructure::{DataStructure},
    gen_params_in::{GenParamsIn,InWhat},
    impl_interfacetype::{TRAIT_LIST,private_associated_type},
    parse_utils::parse_str_as_ident,
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
        ds: &'a DataStructure<'a>,
        arenas: &'a Arenas,
    )->Self{
        let name=ds.name;

        let alignment=unchecked.alignment
            .unwrap_or(IntOrType::Int(std::mem::size_of::<usize>()));
        
        let parse_ident=move|s:&str|->&'a Ident{
            arenas.alloc(parse_str_as_ident(s))
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

        if let Some(EnumInterface::New(enum_interface))=&mut unchecked.enum_interface{
            let mut trait_map=TRAIT_LIST.iter()
                .map(|x| ( parse_ident(x.name) , x.which_trait.default_value() ) )
                .collect::<HashMap<&'a syn::Ident,bool>>();

            enum_interface.impld.iter()
                .chain(enum_interface.unimpld.iter())
                .for_each(|&trait_|{
                    if trait_map.remove(trait_).is_none() {
                        panic!("Trait {} was not in TRAIT_LIST.",trait_);
                    }
                });

            for (trait_,is_impld) in trait_map {
                if is_impld {
                    &mut enum_interface.impld
                }else{
                    &mut enum_interface.unimpld
                }.push(trait_);
            }
        }

        let (default_interface,new_interface)=match unchecked.enum_interface {
            Some(EnumInterface::New{..})=>{
                let name=arenas.alloc(parse_str_as_ident(&format!("{}_Interface",name)));
                (name.into_token_stream(),Some(name))
            },
            Some(EnumInterface::Old(ty))=>{
                ((&ty).into_token_stream(),None)
            }
            None=>{
                (quote!(()),None)
            }
        };


        Self{
            nonexhaustive_alias:parse_ident(&format!("{}_NE",name)),
            nonexhaustive_marker:parse_ident(&format!("{}_NEMarker",name)),
            enum_storage:parse_ident(&format!("{}_Storage",name)),
            alignment,
            size,
            enum_interface:unchecked.enum_interface,
            default_interface,
            new_interface,
            assert_nonexh:unchecked.assert_nonexh,
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

        let mut type_generics_decl=GenParamsIn::new(ds.generics,InWhat::ImplHeader);
        type_generics_decl.set_no_bounds();

        let type_generics_use=GenParamsIn::new(ds.generics,InWhat::ItemUse);

        let alias_docs=format!(
            "An alias for the NonExhaustive wrapped version of `{}<_>`.",
            name
        );

        let default_interface=&this.default_interface;

        quote!(
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

            #vis struct #nonexhaustive_marker<T,S>(
                std::marker::PhantomData<T>,
                std::marker::PhantomData<S>,
            );
        ).to_tokens(ts);


        if let Some(new_interface)=this.new_interface{
            quote!( 
                #[repr(C)]
                #[derive(::abi_stable::StableAbi)]
                #vis struct #new_interface;
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