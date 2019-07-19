use crate::{
    *,
    impl_interfacetype::private_associated_type,
    parse_utils::{parse_str_as_ident},
    my_visibility::{MyVisibility,RelativeVis},
    gen_params_in::{GenParamsIn,InWhat},
    workaround::token_stream_to_string,
};

use std::{
    marker::PhantomData,
};

use proc_macro2::TokenStream as TokenStream2;

use syn::{
    ItemTrait,
};

mod attribute_parsing;
mod common_tokens;
mod impl_delegations;
mod method_where_clause;
mod lifetime_unelider;
mod replace_self_path;
mod trait_definition;
mod methods_tokenizer;

#[cfg(test)]
mod tests;

use self::{
    attribute_parsing::SabiTraitOptions,
    common_tokens::CommonTokens,
    lifetime_unelider::LifetimeUnelider,
    trait_definition::{TraitDefinition,TraitMethod},
    method_where_clause::MethodWhereClause,
    methods_tokenizer::MethodsTokenizer,
};


#[allow(dead_code)]
#[derive(Copy,Clone)]
struct TokenizerParams<'a>{
    arenas:&'a Arenas,
    ctokens:&'a CommonTokens,
    config:&'a SabiTraitOptions<'a>,
    trait_def:&'a TraitDefinition<'a>,
    vis:MyVisibility<'a>,
    submod_vis:RelativeVis<'a>,
    totrait_def:&'a TraitDefinition<'a>,
    vtable_trait_decl:&'a TraitDefinition<'a>,
    vtable_trait_impl:&'a TraitDefinition<'a>,
    generated_mod:&'a syn::Ident,
    trait_to:&'a syn::Ident,
    trait_method:&'a syn::Ident,
    trait_backend:&'a syn::Ident,
    trait_interface:&'a syn::Ident,
}



pub fn derive_sabi_trait(item: ItemTrait) -> TokenStream2 {
    let arenas = Arenas::default();
    let arenas = &arenas;
    let ctokens = CommonTokens::new();
    let ctokens = &ctokens;
    
    let trait_ident=&item.ident;

    let config=&self::attribute_parsing::parse_attrs_for_sabi_trait(&item,arenas,ctokens);

    let trait_def=&config.trait_definition;
    let vis=trait_def.vis;
    let submod_vis=trait_def.submod_vis;
    
    let totrait_def=&trait_def.replace_self(WhichItem::TraitObjectImpl);
    let vtable_trait_decl=&trait_def.replace_self(WhichItem::VtableDecl);
    let vtable_trait_impl=&trait_def.replace_self(WhichItem::VtableImpl);


    let generated_mod=&parse_str_as_ident(&format!("{}_module",trait_ident));
    let trait_to    =&parse_str_as_ident(&format!("{}_TO",trait_ident));
    let trait_method=&parse_str_as_ident(&format!("{}_Methods",trait_ident));
    let trait_backend=&parse_str_as_ident(&format!("{}_Backend",trait_ident));
    let trait_interface=&parse_str_as_ident(&format!("{}_Interface",trait_ident));
    
    let mut mod_contents=TokenStream2::default();

    let tokenizer_params=TokenizerParams{
        arenas,
        ctokens,
        config,
        trait_def,
        vis,
        submod_vis,
        totrait_def,
        vtable_trait_decl,
        vtable_trait_impl,
        generated_mod,
        trait_to    ,
        trait_method,
        trait_backend,
        trait_interface,
    };

    first_items(tokenizer_params,&mut mod_contents);

    constructor_items(tokenizer_params,&mut mod_contents);
    
    trait_and_impl(tokenizer_params,&mut mod_contents);

    methods_impls(tokenizer_params,&mut mod_contents);

    declare_vtable(tokenizer_params,&mut mod_contents);
    
    vtable_impl(tokenizer_params,&mut mod_contents);

    impl_delegations::delegated_impls(tokenizer_params,&mut mod_contents);

    quote!(
        #[doc(inline)]
        #vis use self::#generated_mod::{
            #trait_to,
            #trait_interface,
            #trait_ident,
            #trait_backend,
        };

        #[allow(explicit_outlives_requirements)]
        mod #generated_mod{
            #mod_contents
        }
    ).observe(|tokens|{
        // drop(_measure_time1);
        if config.debug_print_trait {
            panic!("\n\n\n{}\n\n\n",token_stream_to_string(tokens.clone()));
        }
    })
}

fn first_items<'a>(
    TokenizerParams{
        ctokens,
        trait_def,
        submod_vis,
        generated_mod,
        trait_to,
        trait_method,
        trait_backend,
        trait_interface,
        ..
    }:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let trait_ident=trait_def.name;

    let mut uto_params=trait_def.generics_tokenizer(
        InWhat::ItemDecl,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_lt_erasedptr,
    );
    uto_params.set_no_bounds();

    let uto_params_use=trait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_lt_erasedptr,
    );

    let mut trait_interface_header=trait_def.generics_tokenizer(
        InWhat::ImplHeader,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_empty,
    );
    trait_interface_header.set_no_bounds();

    let mut trait_interface_decl=trait_def.generics_tokenizer(
        InWhat::ItemDecl,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_empty,
    );
    trait_interface_decl.set_no_bounds();
    // trait_interface_decl.set_unsized_types();

    let trait_interface_use=trait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_empty,
    );

    let to_params=trait_def.generics_tokenizer(
        InWhat::ItemDecl,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_lt_erasedptr,
    );

    let where_preds=&trait_def.where_preds;

    let vtable_args=trait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_unit_erasedptr,
    );

    let impld_traits=trait_def.impld_traits.iter().map(|x|&x.ident);
    let impld_traits_a=impld_traits.clone();
    let impld_traits_b=impld_traits.clone();

    let unimpld_traits_a=trait_def.unimpld_traits.iter().cloned();
    let unimpld_traits_b=trait_def.unimpld_traits.iter().cloned();

    let priv_assocty=private_associated_type();

    let object=match trait_def.which_object {
        WhichObject::DynTrait=>quote!(DynTrait),
        WhichObject::RObject=>quote!(RObject),
    };
    let vtable_argument=match trait_def.which_object {
        WhichObject::DynTrait=>quote!(__sabi_re::StaticRef<VTable<#vtable_args>>),
        WhichObject::RObject=>quote!(VTable<#vtable_args>),
    };

    let dummy_struct_generics=
        trait_def.generics_tokenizer(
            InWhat::DummyStruct,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_empty,
        );
    // dummy_struct_generics.set_unsized_types();

    let gen_params_header=
        trait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_empty,
        );
    let gen_params_use=
        trait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_empty,
        );

    let used_trait_object=quote!(__UsedTraitObject<#uto_params_use>);

    let used_to_bound=format!(
        "{}: ::abi_stable::StableAbi",
        (&used_trait_object).into_token_stream()
    );
    
    let trait_flags=&trait_def.trait_flags;
    let send_syncness=match (trait_flags.sync,trait_flags.send) {
        (false,false)=>"UnsyncUnsend",
        (false,true )=>"UnsyncSend",
        (true ,false)=>"SyncUnsend",
        (true ,true )=>"SyncSend",
    }.piped(parse_str_as_ident);

    quote!(
        use super::*;

        use abi_stable::sabi_trait::reexports::{*,__sabi_re};

        use self::{
            #trait_ident as __Trait,
            #trait_to     as __TraitObject,
            #trait_backend as __UsedTraitObject,
            #trait_interface as __TraitInterface,
        };

        #submod_vis type #trait_backend<#uto_params>=
            __sabi_re::#object<
                'lt,
                _ErasedPtr,
                __TraitInterface<#trait_interface_use>,
                #vtable_argument
            >;


        #[repr(C)]
        #[derive(::abi_stable::StableAbi)]
        #submod_vis struct #trait_interface<#trait_interface_decl>(
            ::std::marker::PhantomData<extern fn(#dummy_struct_generics)>
        );

        impl<#trait_interface_header> #trait_interface<#trait_interface_use> {
            #submod_vis const NEW:Self=#trait_interface(::std::marker::PhantomData);
        }


        #[repr(transparent)]
        #[derive(::abi_stable::StableAbi)]
        #[sabi(bound=#used_to_bound)]
        #submod_vis struct #trait_to<#to_params>
        where
            #(#where_preds)*
        {
            #submod_vis obj:#used_trait_object,
            _marker:__sabi_re::UnsafeIgnoredType< __sabi_re::#send_syncness >,
        }

        const __inside_generated_mod:()={
            use abi_stable::{
                InterfaceType,
                type_level::{
                    impl_enum::{Implemented,Unimplemented},
                    trait_marker,
                },
            };

            impl<#trait_interface_header> 
                abi_stable::InterfaceType 
                for __TraitInterface<#trait_interface_use>
            {
                #( type #impld_traits_a=Implemented<trait_marker::#impld_traits_b>; )*
                #( type #unimpld_traits_a=Unimplemented<trait_marker::#unimpld_traits_b>; )*
                type #priv_assocty=();
            }
        };

    ).to_tokens(mod_);
}



fn constructor_items<'a>(
    TokenizerParams{ctokens,totrait_def,submod_vis,..}:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let from_ptr_params=totrait_def.generics_tokenizer(
        InWhat::ImplHeader,
        WithAssocTys::No,
        &ctokens.ts_lt_origptr_erasability,
    );

    let ret_generics=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::FullyQualified),
        &ctokens.ts_lt_transptr,
    );

    let trait_params=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::No,
        &ctokens.empty_ts,
    );

    let assoc_tys_a=totrait_def.assoc_tys.keys();
    let assoc_tys_b=assoc_tys_a.clone();
    let assoc_tys_c=assoc_tys_a.clone();
    let assoc_tys_d=assoc_tys_a.clone();
    
    let make_vtable_args=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::No,
        &ctokens.ts_make_vtable_args,
    );
    
    let fn_erasability_arg=match totrait_def.which_object {
        WhichObject::DynTrait=>quote!(Erasability),
        WhichObject::RObject=>quote!(),
    };

    let trait_interface_use=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_empty,
    );

    let extra_constraints_ptr=match totrait_def.which_object {
        WhichObject::DynTrait=>quote!(
            __TraitInterface<#trait_interface_use>:
                ::abi_stable::erased_types::InterfaceBound,
            __sabi_re::InterfaceFor<
                _OrigPtr::Target,
                __TraitInterface<#trait_interface_use>,
                Erasability
            >: 
                __sabi_re::GetVtable<
                    'lt,
                    _OrigPtr::Target,
                    _OrigPtr::TransmutedPtr,
                    _OrigPtr,
                    __TraitInterface<#trait_interface_use>,
                >,
        ),
        WhichObject::RObject=>quote!(),
    };

    let extra_constraints_value=match totrait_def.which_object {
        WhichObject::DynTrait=>quote!(
            __TraitInterface<#trait_interface_use>:
                ::abi_stable::erased_types::InterfaceBound,
            __sabi_re::InterfaceFor<_Self,__TraitInterface<#trait_interface_use>,Erasability>: 
                __sabi_re::GetVtable<
                    'lt,
                    _Self,
                    __sabi_re::RBox<()>,
                    __sabi_re::RBox<_Self>,
                    __TraitInterface<#trait_interface_use>,
                >,
        ),
        WhichObject::RObject=>quote!(),
    };

    let gen_params_header=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );
    
    let gen_params_use_to=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );

    let gen_params_header_rbox=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt,
        );
    
    let gen_params_use_to_rbox=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_rbox,
        );

    let uto_params_use=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_lt_erasedptr,
    );

    let trait_interface_use=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_empty,
        );

    quote!(
        impl<#gen_params_header> __TraitObject<#gen_params_use_to> {
            #submod_vis fn from_ptr<_OrigPtr,Erasability>(
                ptr:_OrigPtr,
                _erasability:Erasability,
            )->Self
            where
                _OrigPtr:__sabi_re::TransmuteElement<(),TransmutedPtr=_ErasedPtr>+'lt,
                _OrigPtr::Target:
                    __Trait<#trait_params #( #assoc_tys_a= #assoc_tys_b, )* >+
                    Sized+
                    'lt,
                _ErasedPtr:std::ops::Deref<Target=()>,
                __TraitInterface<#trait_interface_use>:
                    __sabi_re::GetRObjectVTable<
                        Erasability,_OrigPtr::Target,_ErasedPtr,_OrigPtr
                    >,
                #extra_constraints_ptr
            {
                unsafe{
                    Self{
                        obj:__UsedTraitObject::with_vtable::<_,#fn_erasability_arg>(
                            ptr,
                            MakeVTable::<#make_vtable_args>::VTABLE
                        ),
                        _marker:__sabi_re::UnsafeIgnoredType::DEFAULT,
                    }
                }
            }

            /// Constructs this trait object from its underlying implementation.
            #submod_vis fn from_sabi(obj:__UsedTraitObject<#uto_params_use>)->Self{
                Self{
                    obj,
                    _marker:__sabi_re::UnsafeIgnoredType::DEFAULT,
                }
            }
        }
        impl<#gen_params_header_rbox> __TraitObject<#gen_params_use_to_rbox> {
            #submod_vis fn from_value<_Self,Erasability>(
                ptr:_Self,
                erasability:Erasability,
            )->Self
            where
                _Self:__Trait<#trait_params #( #assoc_tys_c= #assoc_tys_d, )* >+'lt,
                __TraitInterface<#trait_interface_use>:
                    __sabi_re::GetRObjectVTable<
                        Erasability,_Self,__sabi_re::RBox<()>,__sabi_re::RBox<_Self>
                    >,
                #extra_constraints_value
            {
                Self::from_ptr::<
                    __sabi_re::RBox<_Self>,
                    Erasability
                >(__sabi_re::RBox::new(ptr),erasability)
            }
        }

    ).to_tokens(mod_);
}


fn trait_and_impl<'a>(
    TokenizerParams{ctokens,submod_vis,trait_def,..}:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let other_attrs=trait_def.other_attrs;
    let gen_params_trait=
        trait_def.generics_tokenizer(InWhat::ItemDecl,WithAssocTys::No,&ctokens.empty_ts);
    let where_preds=&trait_def.where_preds;
    let methods_tokenizer_def=trait_def.methods_tokenizer(WhichItem::Trait);
    let methods_tokenizer_impl=trait_def.methods_tokenizer(WhichItem::TraitImpl);
    let lifetime_bounds=&*trait_def.lifetime_bounds;
    let super_traits_a=trait_def.impld_traits.iter().map(|t| &t.bound );
    let super_traits_b=super_traits_a.clone();

    let assoc_tys_a=trait_def.assoc_tys.values().map(|x| &x.assoc_ty );
    

    let erased_ptr_bounds=trait_def.erased_ptr_preds();

    let trait_ident=&trait_def.name;

    quote!(
        #( #[#other_attrs] )*
        #submod_vis trait #trait_ident<
            #gen_params_trait
        >: #( #super_traits_a + )* #(#lifetime_bounds+)*
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
            Self:#( #super_traits_b + )* #(#lifetime_bounds+)* Sized ,
            #erased_ptr_bounds
            #(#where_preds,)*
        {
            #( type #assoc_ty_named_a=#assoc_ty_named_b; )*

            #methods_tokenizer_impl
        }
    ).to_tokens(mod_);
}


fn methods_impls<'a>(
    param:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let TokenizerParams{ctokens,totrait_def,submod_vis,..}=param;
    
    let where_preds=&totrait_def.where_preds;

    let assoc_ty_defs=totrait_def.assoc_tys.values().map(|x| &x.assoc_ty );

    let methods_tokenizer_decl=
        totrait_def.methods_tokenizer(WhichItem::TraitObjectImpl);
    let impl_where_preds=totrait_def.trait_impl_where_preds();

    let super_traits_a=totrait_def.impld_traits.iter().map(|t| &t.bound );
    let super_traits_b=super_traits_a.clone();

    let lifetime_bounds=&*totrait_def.lifetime_bounds;
    
    let gen_params_header=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );
    let gen_params_use_trait=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::No,
            &ctokens.ts_erasedptr,
        );
    let gen_params_use_to=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );
    
    let methods_tokenizer_def=totrait_def.methods_tokenizer(WhichItem::TraitObjectImpl);

    quote!(
        impl<#gen_params_header> __TraitObject<#gen_params_use_to>
        where 
            Self:#( #super_traits_a + )* #( #lifetime_bounds+ )* Sized ,
            #impl_where_preds
        {
            #methods_tokenizer_def
        }
    ).to_tokens(mod_);
}


fn declare_vtable<'a>(
    TokenizerParams{ctokens,vtable_trait_decl,submod_vis,..}:TokenizerParams,
    mod_:&mut TokenStream2,
){
    

    let generics_decl=
        vtable_trait_decl.generics_tokenizer(
            InWhat::ItemDecl,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_self_erasedptr,
        );

    let mut generics_use0=
        vtable_trait_decl.generics_tokenizer(
            InWhat::DummyStruct,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_self_erasedptr,
        );
    generics_use0.set_no_bounds();




    let derive_attrs=vtable_trait_decl.derive_attrs;

    let methods_tokenizer=vtable_trait_decl.methods_tokenizer(WhichItem::VtableDecl);

    let lifetime_bounds=if vtable_trait_decl.lifetime_bounds.is_empty() {
        None
    }else{
        use std::fmt::Write;
        let mut lifetime_bounds=String::with_capacity(32);
        lifetime_bounds.push_str("_Self:");
        for lt in &*vtable_trait_decl.lifetime_bounds {
            let _=write!(lifetime_bounds,"{}+",lt);
        }
        lifetime_bounds.push_str("Sized");
        Some(lifetime_bounds)
    };

    let trait_interface_use=
        vtable_trait_decl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_empty,
        );


    let robject_vtable=quote!(
        __sabi_re::StaticRef<
            __sabi_re::RObjectVtable<_Self,_ErasedPtr,__TraitInterface<#trait_interface_use>>
        >
    );
    let vtable_bound=format!("{}: ::abi_stable::StableAbi",(&robject_vtable).into_token_stream());

    quote!(
        #[repr(C)]
        #[derive(abi_stable::StableAbi)]
        #[sabi(kind(Prefix(prefix_struct="VTable")))]
        #[sabi(missing_field(panic))]
        #( #[sabi(prefix_bound=#lifetime_bounds)] )*
        #[sabi(bound=#vtable_bound)]
        #(#[#derive_attrs])*
        #submod_vis struct VTableVal<#generics_decl>{
            _sabi_tys: ::std::marker::PhantomData<
                extern "C" fn(#generics_use0)
            >,

            _sabi_vtable:#robject_vtable,

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

    let dummy_struct_tys=
        vtable_trait_impl.generics_tokenizer(
            InWhat::DummyStruct,
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

    let trait_interface_use=
        vtable_trait_impl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::Underscore),
            &ctokens.ts_empty,
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
        struct MakeVTable<#struct_decl_generics>(#dummy_struct_tys);


        impl<#impl_header_generics> MakeVTable<#makevtable_generics>
        where 
            _Self:__Trait<#trait_generics>,
            __TraitInterface<#trait_interface_use>:
                __sabi_re::GetRObjectVTable<IA,_Self,_ErasedPtr,_OrigPtr>,
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
    TraitObjectImpl,
    VtableDecl,
    VtableImpl,
}


/// Which type used to implement the trait object.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum WhichObject{
    DynTrait,
    RObject
}

impl Default for WhichObject{
    fn default()->Self{
        WhichObject::RObject
    }
}


/// Which Self type to get the associated types from.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum WhichSelf{
    /// Self::AssocTy
    #[allow(dead_code)]
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
