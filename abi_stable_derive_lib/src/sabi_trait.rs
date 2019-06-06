use crate::{
    *,
    parse_utils::{parse_str_as_ident},
    my_visibility::{MyVisibility,RelativeVis},
    gen_params_in::{GenParamsIn,InWhat},
    utils::NoTokens,
};

use std::{
    borrow::Cow,
    marker::PhantomData,
};

use proc_macro2::TokenStream as TokenStream2;

use syn::{
    ItemTrait,
};

mod attribute_parsing;
mod common_tokens;
mod method_where_clause;
mod replace_self_path;
mod trait_definition;
mod methods_tokenizer;

use self::{
    attribute_parsing::SabiTraitOptions,
    common_tokens::CommonTokens,
    trait_definition::{TraitDefinition,TraitMethod,GenericsTokenizer},
    method_where_clause::MethodWhereClause,
    methods_tokenizer::MethodsTokenizer,
};

#[derive(Copy,Clone)]
struct TokenizerParams<'a>{
    arenas:&'a Arenas,
    ctokens:&'a CommonTokens,
    config:&'a SabiTraitOptions<'a>,
    trait_def:&'a TraitDefinition<'a>,
    vis:MyVisibility<'a>,
    submod_vis:RelativeVis<'a>,
    alttrait_def:&'a TraitDefinition<'a>,
    vtable_trait_decl:&'a TraitDefinition<'a>,
    vtable_trait_impl:&'a TraitDefinition<'a>,
}



pub fn derive_sabi_trait(mut item: ItemTrait) -> TokenStream2 {
    let arenas = Arenas::default();
    let arenas = &arenas;
    let ctokens = CommonTokens::new();
    let ctokens = &ctokens;
    
    let trait_ident=&item.ident;

    let config=&self::attribute_parsing::parse_attrs_for_sabi_trait(&item,arenas,ctokens);

    let trait_def=&config.trait_definition;
    let vis=trait_def.vis;
    let submod_vis=trait_def.submod_vis;
    
    let alttrait_def=&trait_def.replace_self(WhichItem::TraitMethodsDecl);
    let vtable_trait_decl=&trait_def.replace_self(WhichItem::VtableDecl);
    let vtable_trait_impl=&trait_def.replace_self(WhichItem::VtableImpl);


    let generated_mod=&parse_str_as_ident(&format!("{}_module",trait_ident));

    let trait_to    =&parse_str_as_ident(&format!("{}_TO",trait_ident));
    let trait_method=&parse_str_as_ident(&format!("{}_Methods",trait_ident));
    let trait_marker=&parse_str_as_ident(&format!("{}_Marker",trait_ident));
    let from_value_ctr=&parse_str_as_ident(&format!("{}_from_value",trait_ident));
    let from_ptr_ctr=&parse_str_as_ident(&format!("{}_from_ptr",trait_ident));

    let mut mod_contents=TokenStream2::default();

    let tokenizer_params=TokenizerParams{
        arenas,
        ctokens,
        config,
        trait_def,
        vis,
        submod_vis,
        alttrait_def,
        vtable_trait_decl,
        vtable_trait_impl,
    };

    first_items(tokenizer_params,&mut mod_contents);

    constructor_items(tokenizer_params,&mut mod_contents);
    
    trait_and_impl(tokenizer_params,&mut mod_contents);

    methods_trait_and_impl(tokenizer_params,&mut mod_contents);

    declare_vtable(tokenizer_params,&mut mod_contents);
    
    vtable_impl(tokenizer_params,&mut mod_contents);

    quote!(
        #vis use self::#generated_mod::{
            __TraitObject as #trait_to,
            __Methods as #trait_method,
            __TraitMarker as #trait_marker,
            __Trait as #trait_ident,
            __trait_from_value as #from_value_ctr,
            __trait_from_ptr as #from_ptr_ctr,
        };

        mod #generated_mod{
            #mod_contents
        }
    ).observe(|tokens|{
        // drop(_measure_time1);
        if config.debug_print_trait {
            panic!("\n\n\n{}\n\n\n",tokens );
        }
    })
}



fn first_items<'a>(
    TokenizerParams{
        ctokens,trait_def,
        submod_vis,
        ..
    }:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let mut to_params=trait_def.generics_tokenizer(
        InWhat::ItemDecl,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_lt_erasedptr,
    );
    to_params.set_no_bounds();

    let vtable_args=trait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_unit_erasedptr,
    );

    let impld_traits=trait_def.impld_traits.iter().map(|x|parse_str_as_ident(x.name));
    let unimpld_traits=trait_def.unimpld_traits.keys();

    quote!(
        use super::*;

        use abi_stable::sabi_trait::reexports::{*,__sabi_re};

        #[repr(C)]
        #[derive(StableAbi)]
        #submod_vis struct __TraitMarker;

        pub type __TraitObject<#to_params>=
            __sabi_re::RObject<'lt,_ErasedPtr,__TraitMarker,VTable<#vtable_args>>;

        mod __inside_generated_mod{
            use super::__TraitMarker;
            use abi_stable::{InterfaceType,type_level::bools::*};

            abi_stable::impl_InterfaceType!{
                impl InterfaceType for __TraitMarker{
                    #( type #impld_traits=True; )*
                    #( type #unimpld_traits=False; )*
                }
            }
        }

    ).to_tokens(mod_);
}



fn constructor_items<'a>(
    TokenizerParams{ctokens,trait_def,..}:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let from_ptr_params=trait_def.generics_tokenizer(
        InWhat::ImplHeader,
        WithAssocTys::No,
        &ctokens.ts_lt_origptr_erasability,
    );

    let ret_generics=trait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::FullyQualified),
        &ctokens.ts_lt_transptr,
    );

    let trait_params=trait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::No,
        &ctokens.empty_ts,
    );
    
    let make_vtable_args=trait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::No,
        &ctokens.ts_make_vtable_args,
    );
    
    

    quote!(
        pub fn __trait_from_ptr<#from_ptr_params>(
            ptr:_OrigPtr,
        )->__TraitObject<#ret_generics>
        where
            _OrigPtr:__sabi_re::TransmuteElement<()>+'lt,
            _OrigPtr::Target:__Trait<#trait_params>+Sized+'lt,
            __TraitMarker:__sabi_re::GetRObjectVTable<
                Erasability,_OrigPtr::Target,_OrigPtr::TransmutedPtr,_OrigPtr
            >
        {
            unsafe{
                __TraitObject::with_vtable(ptr,MakeVTable::<#make_vtable_args>::VTABLE)
            }
        }
    ).to_tokens(mod_);

    let from_value_params=trait_def.generics_tokenizer(
        InWhat::ImplHeader,
        WithAssocTys::No,
        &ctokens.ts_lt_self_erasability,
    );

    let ret_generics=trait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::Underscore),
        &ctokens.ts_lt_rbox,
    );

    quote!(
        pub fn __trait_from_value<#from_value_params>(
            ptr:_Self,
        )->__TraitObject<#ret_generics>
        where
            _Self:__Trait<#trait_params>+'lt,
            __TraitMarker:__sabi_re::GetRObjectVTable<
                Erasability,_Self,__sabi_re::RBox<()>,__sabi_re::RBox<_Self>
            >
        {
            __trait_from_ptr(__sabi_re::RBox::new(ptr))
        }

    ).to_tokens(mod_);
}


fn trait_and_impl<'a>(
    TokenizerParams{ctokens,trait_def,submod_vis,alttrait_def,..}:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let other_attrs=trait_def.other_attrs;
    let gen_params_trait=
        trait_def.generics_tokenizer(InWhat::ItemDecl,WithAssocTys::No,&ctokens.empty_ts);
    let where_preds=&trait_def.where_preds;
    let methods_tokenizer_def=trait_def.methods_tokenizer(WhichItem::Trait);
    let methods_tokenizer_impl=trait_def.methods_tokenizer(WhichItem::TraitImpl);
    let super_traits=&trait_def.impld_traits.iter()
        .filter(|t| t.is_object_safe() )
        .map(|t| &t.full_path )
        .collect::<Vec<_>>();

    let assoc_tys_a=trait_def.assoc_tys.values().map(|x| &x.assoc_ty );
    

    let erased_ptr_bounds=alttrait_def.erased_ptr_preds();

    quote!(
        #( #[#other_attrs] )*
        #submod_vis trait __Trait<#gen_params_trait>: #( #super_traits + )*
        where 
            #(#where_preds,)*
        {
            #( #assoc_tys_a )*

            #methods_tokenizer_def
        }
    ).to_tokens(mod_);

    let gen_params_header=
        trait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );
    let gen_params_use_trait=
        trait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::No,
            &ctokens.empty_ts,
        );
    let gen_params_use_to=
        trait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );

    let assoc_ty_named_a=trait_def.assoc_tys.values().map(|x| &x.assoc_ty.ident );
    let assoc_ty_named_b=assoc_ty_named_a.clone();

    quote!(
        impl<#gen_params_header> __Trait<#gen_params_use_trait> 
        for __TraitObject<#gen_params_use_to>
        where
            Self:#( #super_traits + )* Sized ,
            #erased_ptr_bounds
            #(#where_preds,)*
        {
            #( type #assoc_ty_named_a=#assoc_ty_named_b; )*

            #methods_tokenizer_impl
        }
    ).to_tokens(mod_);
}


fn methods_trait_and_impl<'a>(
    TokenizerParams{ctokens,trait_def,submod_vis,alttrait_def,..}:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let other_attrs=trait_def.other_attrs;

    let where_preds=&alttrait_def.where_preds;

    let assoc_ty_defs=alttrait_def.assoc_tys.values().map(|x| &x.assoc_ty );

    let gen_params_traitmethod=
        alttrait_def.generics_tokenizer(
            InWhat::ItemDecl,
            WithAssocTys::No,
            &ctokens.ts_erasedptr
        );


    let methods_tokenizer_decl=
        alttrait_def.methods_tokenizer(WhichItem::TraitMethodsDecl);
    let impl_where_preds=alttrait_def.trait_impl_where_preds();
    let super_traits_a=alttrait_def.impld_traits.iter().map(|t| &t.full_path );
    let super_traits_b=super_traits_a.clone();
    
    let trait_docs=get_methods_trait_docs(alttrait_def);

    quote!(
        #[doc=#trait_docs]
        #( #[#other_attrs] )*
        #submod_vis trait __Methods<#gen_params_traitmethod>: #( #super_traits_a + )*
        where 
            #impl_where_preds
            #(#where_preds,)*
        {
            #( #assoc_ty_defs )*

            #methods_tokenizer_decl
        }
    ).to_tokens(mod_);

    let gen_params_header=
        alttrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );
    let gen_params_use_trait=
        alttrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::No,
            &ctokens.ts_erasedptr,
        );
    let gen_params_use_to=
        alttrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );
    
    let assoc_ty_named_a=alttrait_def.assoc_tys.values().map(|x| &x.assoc_ty.ident );
    let assoc_ty_named_b=assoc_ty_named_a.clone();

    let methods_tokenizer_def=alttrait_def.methods_tokenizer(WhichItem::TraitMethodsImpl);

    quote!(
        impl<#gen_params_header> __Methods<#gen_params_use_trait>
        for __TraitObject<#gen_params_use_to>
        where 
            Self:#( #super_traits_b + )* Sized ,
            #(#where_preds,)*
        {
            #( type #assoc_ty_named_a=#assoc_ty_named_b; )*

            #methods_tokenizer_def
        }
    ).to_tokens(mod_);
}


fn declare_vtable<'a>(
    TokenizerParams{ctokens,vtable_trait_decl,..}:TokenizerParams,
    mod_:&mut TokenStream2,
){
    

    let generics_decl=
        vtable_trait_decl.generics_tokenizer(
            InWhat::ItemDecl,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_self_erasedptr,
        );

    let generics_use0=
        vtable_trait_decl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_self_erasedptr,
        );

    let derive_attrs=vtable_trait_decl.derive_attrs;

    let methods_tokenizer=vtable_trait_decl.methods_tokenizer(WhichItem::VtableDecl);

    quote!(
        #[repr(C)]
        #[derive(StableAbi)]
        #[sabi(kind(Prefix(prefix_struct="VTable")))]
        #[sabi(missing_field(panic))]
        #(#[sabi(#derive_attrs)])*
        pub struct VTableVal<#generics_decl>{
            _sabi_tys: ::std::marker::PhantomData<
                extern "C" fn(#generics_use0)
            >,

            _sabi_vtable:__sabi_re::StaticRef<
                __sabi_re::RObjectVtable<_Self,_ErasedPtr,__TraitMarker>
            >,

            #methods_tokenizer
        }
    ).to_tokens(mod_);

}


fn vtable_impl<'a>(
    TokenizerParams{ctokens,vtable_trait_impl,..}:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let struct_decl_generics=
        vtable_trait_impl.generics_tokenizer(
            InWhat::ItemDecl,
            WithAssocTys::No,
            &ctokens.ts_getvtable_params,
        );

    let struct_use_generics=
        vtable_trait_impl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::No,
            &ctokens.ts_getvtable_params,
        );

    let impl_header_generics=
        vtable_trait_impl.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::No,
            &ctokens.ts_getvtable_params,
        );

    let makevtable_generics=
        vtable_trait_impl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::No,
            &ctokens.ts_getvtable_params,
        );

    let trait_generics=
        vtable_trait_impl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::No,
            &ctokens.empty_ts
        );

    let withmetadata_generics=
        vtable_trait_impl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::Underscore),
            &ctokens.ts_self_erasedptr,
        );

    let method_names_a=vtable_trait_impl.methods.iter().map(|m|m.name);
    let method_names_b=method_names_a.clone();

    let vtable_generics=
        vtable_trait_impl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::Underscore),
            &ctokens.ts_unit_erasedptr,
        );

    let methods_tokenizer=vtable_trait_impl.methods_tokenizer(WhichItem::VtableImpl);

    quote!(
        struct MakeVTable<#struct_decl_generics>(#struct_use_generics);


        impl<#impl_header_generics> MakeVTable<#makevtable_generics>
        where 
            _Self:__Trait<#trait_generics>,
            __TraitMarker:__sabi_re::GetRObjectVTable<IA,_Self,_ErasedPtr,_OrigPtr>,
        {
            const TMP0: *const __sabi_re::WithMetadata<
                VTableVal<#withmetadata_generics>
            >={
                let __vtable=VTableVal{
                    _sabi_tys: ::std::marker::PhantomData,
                    _sabi_vtable:__sabi_re::GetRObjectVTable::ROBJECT_VTABLE,
                    #(
                        #method_names_a:Self::#method_names_b,
                    )*
                };
                &__sabi_re::WithMetadata::new(
                    __sabi_re::PrefixTypeTrait::METADATA,
                    __vtable
                )
            };

            const VTABLE:__sabi_re::StaticRef<VTable<#vtable_generics>>=unsafe{
                let __vtable=__sabi_re::StaticRef::from_raw(Self::TMP0);
                __sabi_re::WithMetadata::staticref_as_prefix(__vtable)
                    .transmute_ref()
            };

            #methods_tokenizer
        }                    
    ).to_tokens(mod_);
}




fn get_methods_trait_docs<'a>(trait_def:&TraitDefinition<'a>)->String{

    format!("\
This is equivalent to {name}.

",
    name=trait_def.name,
)

}



#[derive(Debug,Clone,PartialEq,Eq)]
pub(crate) enum SelfParam<'a>{
    ByRef{
        lifetime:Option<&'a syn::Lifetime>,
        is_mutable:bool,
    },
    ByVal,
}


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum WhichItem{
    Trait,
    TraitImpl,
    TraitMethodsDecl,
    TraitMethodsImpl,
    VtableDecl,
    VtableImpl,
}


/// Which Self type to get the associated types from.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum WhichSelf{
    /// Self::AssocTy
    Regular,
    /// _Self::AssocTy
    Underscore,
    /// <_OrigPtr as __Trait< <generic_params> >>::AssocTy
    FullyQualified,
    /// AssocTy
    NoSelf,
}


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum WithAssocTys{
    No,
    Yes(WhichSelf),
}
