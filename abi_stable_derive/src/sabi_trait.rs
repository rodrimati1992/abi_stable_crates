use crate::{
    *,
    impl_interfacetype::private_associated_type,
    parse_utils::{parse_str_as_ident},
    my_visibility::{VisibilityKind,RelativeVis},
    gen_params_in::{GenParamsIn,InWhat},
    to_token_fn::ToTokenFnMut,
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
    common_tokens::{CommonTokens,IsStaticTrait,LifetimeTokens},
    lifetime_unelider::LifetimeUnelider,
    trait_definition::{TraitDefinition,TraitMethod},
    method_where_clause::MethodWhereClause,
    methods_tokenizer::MethodsTokenizer,
};

/// Variables passed to all the `*_items` functions here.
#[allow(dead_code)]
#[derive(Copy,Clone)]
struct TokenizerParams<'a>{
    arenas:&'a Arenas,
    ctokens:&'a CommonTokens,
    config:&'a SabiTraitOptions<'a>,
    trait_def:&'a TraitDefinition<'a>,
    vis:VisibilityKind<'a>,
    submod_vis:RelativeVis<'a>,
    totrait_def:&'a TraitDefinition<'a>,
    vtable_trait_decl:&'a TraitDefinition<'a>,
    vtable_trait_impl:&'a TraitDefinition<'a>,
    trait_ident:&'a syn::Ident,
    /// The name of `Trait_Bounds`.
    trait_bounds:&'a syn::Ident,
    trait_to:&'a syn::Ident,
    trait_backend:&'a syn::Ident,
    trait_interface:&'a syn::Ident,
    make_vtable_ident:&'a syn::Ident,
    trait_cto_ident:&'a syn::Ident,
    /// TokenStreams that don't have a `'lt,` if the trait object requires 
    /// `'static` to be constructed.
    lt_tokens:&'a LifetimeTokens,
}


/// The implementation of the `#[sabi_trait]` proc-macro attribute.
pub fn derive_sabi_trait(item: ItemTrait) -> Result<TokenStream2,syn::Error> {
    let arenas = Arenas::default();
    let arenas = &arenas;
    let ctokens = CommonTokens::new();
    let ctokens = &ctokens;
    
    let trait_ident=&item.ident;

    let config=&self::attribute_parsing::parse_attrs_for_sabi_trait(&item,arenas,ctokens)?;

    let trait_def=&config.trait_definition;
    let lt_tokens=&LifetimeTokens::new(trait_def.is_static);
    let vis=trait_def.vis;
    let submod_vis=trait_def.submod_vis;
    
    let totrait_def=&trait_def.replace_self(WhichItem::TraitObjectImpl)?;
    let vtable_trait_decl=&trait_def.replace_self(WhichItem::VtableDecl)?;
    let vtable_trait_impl=&trait_def.replace_self(WhichItem::VtableImpl)?;


    let generated_mod=&parse_str_as_ident(&format!("{}_module",trait_ident));
    let trait_bounds=&parse_str_as_ident(&format!("{}_Bounds",trait_ident));
    let trait_to    =&parse_str_as_ident(&format!("{}_TO",trait_ident));
    let trait_backend=&parse_str_as_ident(&format!("{}_Backend",trait_ident));
    let trait_interface=&parse_str_as_ident(&format!("{}_Interface",trait_ident));
    let make_vtable_ident=&parse_str_as_ident(&format!("{}_MV",trait_ident));
    let trait_cto_ident=&parse_str_as_ident(&format!("{}_CTO",trait_ident));
    
    let mut mod_contents=TokenStream2::default();

    let tokenizer_params=TokenizerParams{
        arenas,
        ctokens,
        config,
        lt_tokens,
        trait_def,
        vis,
        submod_vis,
        totrait_def,
        vtable_trait_decl,
        vtable_trait_impl,
        trait_ident,
        trait_bounds,
        trait_to    ,
        trait_backend,
        trait_interface,
        make_vtable_ident,
        trait_cto_ident,
    };

    first_items(tokenizer_params,&mut mod_contents);

    constructor_items(tokenizer_params,&mut mod_contents);
    
    trait_and_impl(tokenizer_params,&mut mod_contents);

    methods_impls(tokenizer_params,&mut mod_contents)?;

    declare_vtable(tokenizer_params,&mut mod_contents);
    
    vtable_impl(tokenizer_params,&mut mod_contents);

    impl_delegations::delegated_impls(tokenizer_params,&mut mod_contents);

    quote!(
        #[doc(inline)]
        #vis use self::#generated_mod::{
            #trait_to,
            #trait_bounds,
            #trait_interface,
            #trait_ident,
            #trait_backend,
            #make_vtable_ident,
            #trait_cto_ident,
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
    .piped(Ok)
}


/**
Outputs these items:

- `Trait_Backend`: 
    A type alias to the underlying implementation of the trait object,
    which is either RObject

- `Trait_Interface`
    A marker type describing the traits that are required when constructing 
    the underlying implementation of the trait object,
    and are then implemented by it,by implementing InterfaceType.

- `Trait_TO`:
    The ffi-safe trait object for the trait.

*/
fn first_items<'a>(
    TokenizerParams{
        ctokens,
        lt_tokens,
        trait_def,
        submod_vis,
        trait_to,
        trait_backend,
        trait_interface,
        trait_cto_ident,
        ..
    }:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let trait_ident=trait_def.name;

    let mut uto_params=trait_def.generics_tokenizer(
        InWhat::ItemDecl,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &lt_tokens.lt_erasedptr,
    );
    uto_params.set_no_bounds();
    
    let mut gen_params_header_rref=
        trait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt_sub_lt,
        );
    gen_params_header_rref.set_no_bounds();

    let gen_params_use_to_rref=
        trait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt_rref,
        );

    let uto_params_use=trait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &lt_tokens.lt_erasedptr,
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

    let mut to_params=trait_def.generics_tokenizer(
        InWhat::ItemDecl,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &lt_tokens.lt_erasedptr,
    );
    to_params.set_no_bounds();

    let where_preds=(&trait_def.where_preds).into_iter();

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

    let used_trait_object=quote!(#trait_backend<#uto_params_use>);

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

    let trait_backend_docs=format!(
        "An alias for the underlying implementation of `{}`.",
        trait_to,
    );

    let trait_interface_docs=format!(
        "A marker type describing the traits that are required when constructing `{}`,\
         and are then implemented by it,
         by implementing InterfaceType.",
        trait_to,
    );
    
    let trait_to_docs=format!(
        "\
The trait object for [{Trait}](trait.{Trait}.html).

There are extra methods on the `obj` field.
",
        Trait=trait_ident
    );    
    
    let trait_cto_docs=format!(
        "A type alias for the const-constructible `{trait_to}`.",
        trait_to=trait_to
    );

    let one_lt=&lt_tokens.one_lt;

    quote!(
        use super::*;

        use abi_stable::sabi_trait::reexports::{*,__sabi_re};

        use self::#trait_ident as __Trait;


        #[doc=#trait_cto_docs]
        #submod_vis type #trait_backend<#uto_params>=
            __sabi_re::#object<
                #one_lt
                _ErasedPtr,
                #trait_interface<#trait_interface_use>,
                #vtable_argument
            >;

        
        #[doc=#trait_cto_docs]
        #submod_vis type #trait_cto_ident<#gen_params_header_rref>=
            #trait_to<#gen_params_use_to_rref>;

        

        #[doc=#trait_interface_docs]
        #[repr(C)]
        #[derive(::abi_stable::StableAbi)]
        #submod_vis struct #trait_interface<#trait_interface_decl>(
            ::std::marker::PhantomData<extern "C" fn(#dummy_struct_generics)>
        );

        impl<#trait_interface_header> #trait_interface<#trait_interface_use> {
            #submod_vis const NEW:Self=#trait_interface(::std::marker::PhantomData);
        }


        #[doc=#trait_to_docs]
        #[repr(transparent)]
        #[derive(::abi_stable::StableAbi)]
        #[sabi(bound=#used_to_bound)]
        #submod_vis struct #trait_to<#to_params>
        where
            _ErasedPtr:__GetPointerKind,
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
                for #trait_interface<#trait_interface_use>
            {
                #( type #impld_traits_a=Implemented<trait_marker::#impld_traits_b>; )*
                #( type #unimpld_traits_a=Unimplemented<trait_marker::#unimpld_traits_b>; )*
                type #priv_assocty=();
            }
        };

    ).to_tokens(mod_);
}


/// Outputs the trait object constructors.
fn constructor_items<'a>(
    params:TokenizerParams<'a>,
    mod_:&mut TokenStream2,
){
    let TokenizerParams{
        ctokens,totrait_def,submod_vis,trait_ident,trait_to,trait_backend,trait_interface,
        lt_tokens,trait_bounds,make_vtable_ident,
        ..
    }=params;

    let trait_params=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::No,
        &ctokens.empty_ts,
    );

    let assoc_tys_a=totrait_def.assoc_tys.keys();
    let assoc_tys_b=assoc_tys_a.clone();
    let assoc_tys_c=assoc_tys_a.clone();
    let assoc_tys_d=assoc_tys_a.clone();
    
    let mut make_vtable_args=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::No,
        &ctokens.ts_make_vtable_args,
    );
    make_vtable_args.skip_lifetimes();
    
    let fn_unerasability_arg=match totrait_def.which_object {
        WhichObject::DynTrait=>quote!(Unerasability),
        WhichObject::RObject=>quote!(),
    };

    let trait_interface_use=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_empty,
    );

    let one_lt=&lt_tokens.one_lt;

    let extra_constraints_ptr=match totrait_def.which_object {
        WhichObject::DynTrait=>quote!(
            #trait_interface<#trait_interface_use>:
                ::abi_stable::erased_types::InterfaceBound,
            __sabi_re::InterfaceFor<
                _OrigPtr::Target,
                #trait_interface<#trait_interface_use>,
                Unerasability
            >: 
                __sabi_re::GetVtable<
                    #one_lt
                    _OrigPtr::Target,
                    _OrigPtr::TransmutedPtr,
                    _OrigPtr,
                    #trait_interface<#trait_interface_use>,
                >,
        ),
        WhichObject::RObject=>quote!(),
    };


    let extra_constraints_value=match totrait_def.which_object {
        WhichObject::DynTrait=>quote!(
            #trait_interface<#trait_interface_use>:
                ::abi_stable::erased_types::InterfaceBound,
            __sabi_re::InterfaceFor<_Self,#trait_interface<#trait_interface_use>,Unerasability>: 
                __sabi_re::GetVtable<
                    #one_lt
                    _Self,
                    __sabi_re::RBox<()>,
                    __sabi_re::RBox<_Self>,
                    #trait_interface<#trait_interface_use>,
                >,
        ),
        WhichObject::RObject=>quote!(),
    };

    let gen_params_header=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt_erasedptr,
        );
    
    let gen_params_use_to=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt_erasedptr,
        );

    let gen_params_header_rbox=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt,
        );
    
    let gen_params_use_to_rbox=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt_rbox,
        );

    let uto_params_use=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &lt_tokens.lt_erasedptr,
    );

    let trait_interface_use=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_empty,
        );

    let mut gen_params_header_rref=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt_sub_lt,
        );
    gen_params_header_rref.set_no_bounds();

    let gen_params_use_to_rref=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt_rref,
        );

    let shared_docs=format!(
        "
`unerasability` describes whether the trait object can be \
converted back into the original type or not.
Its possible values are `TU_Unerasable` and `TU_Opaque`.
"
    );

    let from_ptr_docs=format!(
        "Constructs this trait object from a pointer to a type that implements `{trait_}`.",
        trait_=trait_ident
    );

    let from_value_docs=format!(
        "Constructs this trait a type that implements `{trait_}`.",
        trait_=trait_ident
    );

    let from_const_docs=format!(
        "Constructs this trait from a constant of a type that implements `{trait_}`.\n\
         \n\
         You can construct the `vtable_for` parameter with `{make_vtable_ident}::VTABLE`.
        ",
        trait_=trait_ident,
        make_vtable_ident=make_vtable_ident,
    );
    
    let vtable_generics=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::Underscore),
            &ctokens.ts_unit_erasedptr,
        );
    
    let vtable_generics_rref=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_unit_rref_unit,
        );

    let reborrow_methods=reborrow_methods_tokenizer(params);

    let plus_lt=&lt_tokens.plus_lt;
    
    
    let vtable_type=match totrait_def.which_object {
        WhichObject::DynTrait=>quote!(
            __sabi_re::VTableTO_DT<
                #one_lt
                _Self,
                __sabi_re::RRef<'_sub,()>,
                __sabi_re::RRef<'_sub,_Self>,
                #trait_interface<#trait_interface_use>,
                Unerasability,
                VTable<#vtable_generics_rref>,
            >
        ),
        WhichObject::RObject=>quote!(
            __sabi_re::VTableTO_RO<
                _Self,
                __sabi_re::RRef<'_sub,_Self>,
                Unerasability,
                VTable<#vtable_generics_rref>,
            >
        ),
    };
    
    let constructing_backend=match totrait_def.which_object {
        WhichObject::DynTrait=>quote!(
            #trait_backend::from_const(
                ptr,
                unerasability,
                vtable_for.dyntrait_vtable(),
                vtable_for.robject_vtable(),
            )
        ),
        WhichObject::RObject=>quote!({
            __sabi_re::ManuallyDrop::new(unerasability);
            #trait_backend::with_vtable_const(ptr,vtable_for)
        }),
    };

    quote!(
        impl<#gen_params_header> #trait_to<#gen_params_use_to> 
        where
            _ErasedPtr:__GetPointerKind,
        {
            #[doc=#from_ptr_docs]
            #[doc=#shared_docs]
            #submod_vis fn from_ptr<_OrigPtr,Unerasability>(
                ptr:_OrigPtr,
                unerasability:Unerasability,
            )->Self
            where
                _OrigPtr:__sabi_re::CanTransmuteElement<(),TransmutedPtr=_ErasedPtr> #plus_lt,
                _OrigPtr::Target:
                    #trait_bounds<#trait_params #( #assoc_tys_a= #assoc_tys_b, )* >+
                    Sized
                    #plus_lt,
                _ErasedPtr:std::ops::Deref<Target=()>,
                #trait_interface<#trait_interface_use>:
                    __sabi_re::GetRObjectVTable<
                        Unerasability,_OrigPtr::Target,_ErasedPtr,_OrigPtr
                    >,
                #extra_constraints_ptr
            {
                let _unerasability=unerasability;
                unsafe{
                    Self{
                        obj:#trait_backend::with_vtable::<_,#fn_unerasability_arg>(
                            ptr,
                            #make_vtable_ident::<#make_vtable_args>::VTABLE_INNER
                        ),
                        _marker:__sabi_re::UnsafeIgnoredType::DEFAULT,
                    }
                }
            }

            /// Constructs this trait object from its underlying implementation.
            #submod_vis fn from_sabi(obj:#trait_backend<#uto_params_use>)->Self{
                Self{
                    obj,
                    _marker:__sabi_re::UnsafeIgnoredType::DEFAULT,
                }
            }

            #reborrow_methods
        }
        
        impl<#gen_params_header_rbox> #trait_to<#gen_params_use_to_rbox> {
            #[doc=#from_value_docs]
            #[doc=#shared_docs]
            #submod_vis fn from_value<_Self,Unerasability>(
                ptr:_Self,
                unerasability:Unerasability,
            )->Self
            where
                _Self:
                    #trait_bounds<#trait_params #( #assoc_tys_c= #assoc_tys_d, )* >
                    #plus_lt,
                #trait_interface<#trait_interface_use>:
                    __sabi_re::GetRObjectVTable<
                        Unerasability,_Self,__sabi_re::RBox<()>,__sabi_re::RBox<_Self>
                    >,
                #extra_constraints_value
            {
                Self::from_ptr::<
                    __sabi_re::RBox<_Self>,
                    Unerasability
                >(__sabi_re::RBox::new(ptr),unerasability)
            }
        }

        impl<#gen_params_header_rref> #trait_to<#gen_params_use_to_rref>{
            #[doc=#from_const_docs]
            #[doc=#shared_docs]
            #submod_vis const fn from_const<_Self,Unerasability>(
                ptr:&'_sub _Self,
                unerasability:Unerasability,
                vtable_for:#vtable_type,
            )->Self
            where
                _Self:#one_lt
            {
                unsafe{
                    Self{
                        obj:#constructing_backend,
                        _marker:__sabi_re::UnsafeIgnoredType::DEFAULT,
                    }
                }
            }
        }

    ).to_tokens(mod_);
}


/// Returns a tokenizer for the reborrowing methods
fn reborrow_methods_tokenizer<'a>(
    TokenizerParams{totrait_def,submod_vis,trait_to,lt_tokens,..}:TokenizerParams<'a>,
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{
        let traits=totrait_def.trait_flags;
        // If the trait object doesn't have both Sync+Send as supertraits or neither,
        // it can't be reborrowed.
        if traits.sync!=traits.send {
            return;
        }

        let gen_params_use_ref=
            totrait_def.generics_tokenizer(
                InWhat::ItemUse,
                WithAssocTys::Yes(WhichSelf::NoSelf),
                &lt_tokens.lt_ref,
            );

        let gen_params_use_mut=
            totrait_def.generics_tokenizer(
                InWhat::ItemUse,
                WithAssocTys::Yes(WhichSelf::NoSelf),
                &lt_tokens.lt_mut,
            );


        quote!(
            #submod_vis fn sabi_reborrow<'_sub>(&'_sub self)->#trait_to<#gen_params_use_ref>
            where
                _ErasedPtr:std::ops::Deref<Target=()>
            {
                let x=self.obj.reborrow();
                // This is transmuting the pointer type parameter of the vtable.
                let x=unsafe{ std::mem::transmute(x) };
                #trait_to::from_sabi(x)
            }

            #submod_vis fn sabi_reborrow_mut<'_sub>(&'_sub mut self)->#trait_to<#gen_params_use_mut>
            where
                _ErasedPtr:std::ops::DerefMut<Target=()>
            {
                let x=self.obj.reborrow_mut();
                // This is transmuting the pointer type parameter of the vtable.
                let x=unsafe{ std::mem::transmute(x) };
                #trait_to::from_sabi(x)
            }
        ).to_tokens(ts);
    })
}


/// Outputs the annotated trait (as modified by the proc-macro)
/// and an implementation of the trait for the generated trait object.
fn trait_and_impl<'a>(
    TokenizerParams{
        ctokens,submod_vis,trait_def,trait_to,lt_tokens,
        trait_ident,trait_bounds,..
    }:TokenizerParams,
    mod_:&mut TokenStream2,
){
    let other_attrs=trait_def.other_attrs;
    let gen_params_trait=
        trait_def.generics_tokenizer(InWhat::ItemDecl,WithAssocTys::No,&ctokens.empty_ts);
    let where_preds=(&trait_def.where_preds).into_iter();
    let where_preds_b=where_preds.clone();
    let methods_tokenizer_def=trait_def.methods_tokenizer(WhichItem::Trait);
    let methods_tokenizer_impl=trait_def.methods_tokenizer(WhichItem::TraitImpl);
    let lifetime_bounds_a=trait_def.lifetime_bounds.iter();
    let lifetime_bounds_b=trait_def.lifetime_bounds.iter();
    let lifetime_bounds_c=trait_def.lifetime_bounds.iter();
    let super_traits_a=trait_def.impld_traits.iter().map(|t| &t.bound );
    let super_traits_b=super_traits_a.clone();

    let assoc_tys_a=trait_def.assoc_tys.values().map(|x| &x.assoc_ty );
    

    let erased_ptr_bounds=trait_def.erased_ptr_preds();

    quote!(
        #( #[#other_attrs] )*
        #submod_vis trait #trait_ident<
            #gen_params_trait
        >: #( #super_traits_a + )*
        where 
            #(#where_preds,)*
        {
            #( #assoc_tys_a )*

            #methods_tokenizer_def
        }
    ).to_tokens(mod_);

    let gen_params_use_trait=
        trait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::No,
            &ctokens.empty_ts,
        );

    {
        let gen_params_header=
            trait_def.generics_tokenizer(
                InWhat::ImplHeader,
                WithAssocTys::No,
                &ctokens.ts_uself,
            );

        let docs=format!(
            "A trait alias for [{Trait}](trait.{Trait}.html) + the lifetime bounds that \
             it had before being stripped by the `#[sabi_trait]` attribute.
            ",
            Trait=trait_ident
        );
        quote!(
            #[doc= #docs ]
            #submod_vis trait #trait_bounds<#gen_params_trait>: 
                #trait_ident<#gen_params_use_trait>
                #( + #lifetime_bounds_a )*
            {}

            impl<#gen_params_header> #trait_bounds<#gen_params_use_trait> for _Self
            where
                _Self: #trait_ident<#gen_params_use_trait> #( + #lifetime_bounds_b )*
            {}
        ).to_tokens(mod_);

    }


    if ! trait_def.disable_trait_impl {

        let gen_params_header=
            trait_def.generics_tokenizer(
                InWhat::ImplHeader,
                WithAssocTys::Yes(WhichSelf::NoSelf),
                &lt_tokens.lt_erasedptr,
            );
        let gen_params_use_to=
            trait_def.generics_tokenizer(
                InWhat::ItemUse,
                WithAssocTys::Yes(WhichSelf::NoSelf),
                &lt_tokens.lt_erasedptr,
            );

        let assoc_ty_named_a=trait_def.assoc_tys.values().map(|x| &x.assoc_ty.ident );
        let assoc_ty_named_b=assoc_ty_named_a.clone();

        quote!(
            impl<#gen_params_header> #trait_ident<#gen_params_use_trait> 
            for #trait_to<#gen_params_use_to>
            where
                Self:#( #super_traits_b + )* #(#lifetime_bounds_c+)* Sized ,
                #erased_ptr_bounds
                #(#where_preds_b,)*
            {
                #( type #assoc_ty_named_a=#assoc_ty_named_b; )*

                #methods_tokenizer_impl
            }
        ).to_tokens(mod_);
    }

}

/// An inherent implementation of the generated trait object,
/// which mirrors the trait definition.
fn methods_impls<'a>(
    param:TokenizerParams,
    mod_:&mut TokenStream2,
)-> Result<(),syn::Error> {
    let TokenizerParams{totrait_def,trait_to,ctokens,lt_tokens,..}=param;
    
    let impl_where_preds=totrait_def.trait_impl_where_preds()?;

    let super_traits_a=totrait_def.impld_traits.iter().map(|t| &t.bound );
    
    let gen_params_header=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt_erasedptr,
        );
    let gen_params_use_to=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &lt_tokens.lt_erasedptr,
        );

    let generics_use1=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_unit_erasedptr,
        );
    
    let methods_tokenizer_def=totrait_def.methods_tokenizer(WhichItem::TraitObjectImpl);

    quote!(
        impl<#gen_params_header> #trait_to<#gen_params_use_to>
        where 
            _ErasedPtr:__GetPointerKind,
            Self:#( #super_traits_a + )* Sized ,
            #impl_where_preds
        {
            #[inline]
            fn sabi_vtable<'_vtable>(&self)->&'_vtable VTableInner<#generics_use1>{
                unsafe{
                    &*(self.obj.sabi_et_vtable().get_raw() as *const _ as *const _)
                }
            }

            #methods_tokenizer_def
        }
    ).to_tokens(mod_);

    Ok(())
}

/// Outputs the vtable struct.
fn declare_vtable<'a>(
    TokenizerParams{ctokens,vtable_trait_decl,submod_vis,trait_interface,..}:TokenizerParams,
    mod_:&mut TokenStream2,
){
    

    let generics_decl=
        vtable_trait_decl.generics_tokenizer(
            InWhat::ItemDecl,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_self_erasedptr,
        );    

    let mut generics_decl_unbounded=generics_decl.clone();
    generics_decl_unbounded.set_no_bounds();

    let mut generics_use0=
        vtable_trait_decl.generics_tokenizer(
            InWhat::DummyStruct,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_self_erasedptr,
        );
    generics_use0.set_no_bounds();

    let generics_use1=
        vtable_trait_decl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_self_erasedptr,
        );

    let impl_header_generics=
        vtable_trait_decl.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_self_erasedptr,
        );




    let derive_attrs=vtable_trait_decl.derive_attrs;

    let methods_tokenizer=vtable_trait_decl.methods_tokenizer(WhichItem::VtableDecl);

    let lifetime_bounds=if vtable_trait_decl.lifetime_bounds.is_empty() {
        None
    }else{
        use std::fmt::Write;
        let mut lifetime_bounds=String::with_capacity(32);
        lifetime_bounds.push_str("_Self:");
        for lt in &vtable_trait_decl.lifetime_bounds {
            let _=write!(lifetime_bounds,"{}+",lt);
        }
        lifetime_bounds.push_str("Sized");
        Some(lifetime_bounds)
    }.into_iter();

    let trait_interface_use=
        vtable_trait_decl.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_empty,
        );

    let robject_vtable=quote!(
        __sabi_re::StaticRef<
            __sabi_re::RObjectVtable<
                _Self,
                _ErasedPtr,
                #trait_interface<#trait_interface_use>
            >
        >
    );

    let real_vtable_ty=format!(
        "__sabi_re::StaticRef<VTableInner<{}>>",
        generics_use1.into_token_stream(),
    );
    let inner_vtable_bound=format!("{}: ::abi_stable::StableAbi",real_vtable_ty);

    let vtable_bound=format!("{}: ::abi_stable::StableAbi",(&robject_vtable).into_token_stream());

    let bounds={
        let mut gen_toks=vtable_trait_decl.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_empty,
        );
        gen_toks.skip_unbounded();
        format!(
            "{}_Self:{},_ErasedPtr: __GetPointerKind",
            gen_toks.into_token_stream(),
            vtable_trait_decl.lifetime_bounds.to_token_stream(),
        )
    };

    quote!(
        #[repr(transparent)]
        #[derive(abi_stable::StableAbi)]
        #[sabi(
            bounds=#bounds,
            bound=#inner_vtable_bound,
        )]
        #submod_vis struct VTable<#generics_decl_unbounded>{
            #[sabi(unsafe_change_type=#real_vtable_ty)]
            inner:u64,

            _sabi_tys: ::std::marker::PhantomData<
                extern "C" fn(#generics_use0)
            >,
        }
        
        #[repr(C)]
        #[derive(abi_stable::StableAbi)]
        #[sabi(kind(Prefix(prefix_struct="VTableInner")))]
        #[sabi(missing_field(panic))]
        #( #[sabi(prefix_bound=#lifetime_bounds)] )*
        #[sabi(bound=#vtable_bound)]
        #(#[#derive_attrs])*
        #submod_vis struct VTableInnerVal<#generics_decl>
        where
            _ErasedPtr:__GetPointerKind,
        {
            _sabi_tys: ::std::marker::PhantomData<
                extern "C" fn(#generics_use0)
            >,

            _sabi_vtable:#robject_vtable,

            #methods_tokenizer
        }
    ).to_tokens(mod_);

}

/**
Outputs the vtable impl block with both:

- A constant where the vtable is constructed.

- The methods that the vtable is constructed with.

*/
fn vtable_impl<'a>(
    TokenizerParams{
        ctokens,vtable_trait_impl,trait_interface,
        trait_bounds,make_vtable_ident,submod_vis,lt_tokens,..
    }:TokenizerParams,
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
            &ctokens.ts_getvtable_dummy_struct_fields,
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

    let const_vtable_item=match vtable_trait_impl.which_object {
        WhichObject::DynTrait=>quote!(
            pub const VTABLE:__sabi_re::VTableTO_DT<
                'lt,
                _Self,
                _ErasedPtr,
                _OrigPtr,
                #trait_interface<#trait_interface_use>,
                IA,
                VTable<#vtable_generics>
            >=unsafe{
                __sabi_re::VTableTO_DT::for_dyntrait(
                    Self::VTABLE_INNER,
                    __sabi_re::VTableDT::GET,
                )
            };
        ),
        WhichObject::RObject=>quote!(
            pub const VTABLE:__sabi_re::VTableTO_RO<
                _Self,
                _OrigPtr,
                IA,
                VTable<#vtable_generics>
            >=unsafe{
                __sabi_re::VTableTO_RO::for_robject(Self::VTABLE_INNER)
            };
        ),
    };

    let one_lt=&lt_tokens.one_lt;
    
    let extra_constraints=match vtable_trait_impl.which_object {
        WhichObject::DynTrait=>quote!(
            #trait_interface<#trait_interface_use>:
                ::abi_stable::erased_types::InterfaceBound,
            __sabi_re::InterfaceFor<
                _Self,
                #trait_interface<#trait_interface_use>,
                IA
            >: 
                __sabi_re::GetVtable<
                    #one_lt
                    _Self,
                    _ErasedPtr,
                    _OrigPtr,
                    #trait_interface<#trait_interface_use>,
                >,
        ),
        WhichObject::RObject=>quote!(),
    };


    quote!(
        #submod_vis struct #make_vtable_ident<#struct_decl_generics>(#dummy_struct_tys);


        impl<#impl_header_generics> #make_vtable_ident<#makevtable_generics>
        where 
            _Self:#trait_bounds<#trait_generics>,
            _OrigPtr:__sabi_re::CanTransmuteElement<(),Target=_Self,TransmutedPtr=_ErasedPtr>,
            _ErasedPtr:__GetPointerKind<Target=()>,
            #trait_interface<#trait_interface_use>:
                __sabi_re::GetRObjectVTable<IA,_Self,_ErasedPtr,_OrigPtr>,
            #extra_constraints
        {
            const TMP0: *const __sabi_re::WithMetadata<
                VTableInnerVal<#withmetadata_generics>
            >={
                let __vtable=VTableInnerVal{
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

            const VTABLE_INNER:__sabi_re::StaticRef<VTable<#vtable_generics>>=unsafe{
                let __vtable=__sabi_re::StaticRef::from_raw(Self::TMP0);
                __sabi_re::WithMetadata::staticref_as_prefix(__vtable)
                    .transmute_ref()
            };

            #const_vtable_item

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

/// Which item this is refering to.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum WhichItem{
    /// the method in the trait definition
    Trait,
    /// the method in the trait implemetation for the generated trait object.
    TraitImpl,
    /// the methods in the inherent implemetation of the generated trait object.
    TraitObjectImpl,
    /// the fields of the trait object vtable.
    VtableDecl,
    /// the methods used to construct the vtable.
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
    #[allow(dead_code)]
    FullyQualified,
    /// AssocTy
    NoSelf,
}


/// Whether to include associated types when printing generic parameters. 
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum WithAssocTys{
    No,
    Yes(WhichSelf),
}
