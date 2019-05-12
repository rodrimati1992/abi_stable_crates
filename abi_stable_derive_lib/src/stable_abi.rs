

use crate::*;

use crate::{
    attribute_parsing::{parse_attrs_for_stable_abi, StabilityKind,StableAbiOptions,Repr},
    datastructure::{DataStructure,DataVariant,Struct,Field},
    to_token_fn::ToTokenFnMut,
    fn_pointer_extractor::{FnParamRet},
    prefix_types::prefix_type_tokenizer,
};


use syn::Ident;

use proc_macro2::{TokenStream as TokenStream2,Span};

use core_extensions::{
    prelude::*,

};






pub(crate) fn derive(mut data: DeriveInput) -> TokenStream2 {
    data.generics.make_where_clause();

    let arenas = Arenas::default();
    let arenas = &arenas;
    let ctokens = CommonTokens::new(arenas);
    let ctokens = &ctokens;
    let ds = DataStructure::new(&mut data, arenas, ctokens);
    let config = &parse_attrs_for_stable_abi(&ds.attrs, &ds, arenas);
    let generics=ds.generics;
    let name=ds.name;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause=&where_clause.unwrap().predicates;

    let associated_kind = match config.kind {
        StabilityKind::Value => &ctokens.value_kind,
        StabilityKind::Prefix{..}=>&ctokens.prefix_kind,
    };

    let impl_name= match &config.kind {
        StabilityKind::Value => name,
        StabilityKind::Prefix(prefix)=>&prefix.prefix_struct,
    };

    let mut prefix_type_trait_bound=None;

    let size_align_for=match &config.kind {
        StabilityKind::Prefix(prefix)=>{
            let prefix_struct=prefix.prefix_struct;

            prefix_type_trait_bound=Some(quote!(
                #name #ty_generics:_sabi_reexports::PrefixTypeTrait,
            ));

            quote!( __WithMetadata_<#name #ty_generics,#prefix_struct #ty_generics> )
        }
        StabilityKind::Value=>quote!(Self),
    };
    
    let is_transparent=config.repr==Repr::Transparent ;
    let is_enum=ds.data_variant==DataVariant::Enum;
    let prefix=match &config.kind {
        StabilityKind::Prefix(prefix)=>Some(prefix),
        _=>None,
    };

    let tags_opt=&config.tags;
    let tags=ToTokenFnMut::new(move|ts|{
        match &tags_opt {
            Some(tag)=>{
                tag.to_tokens(ts);
            }
            None=>{
                quote!( _sabi_reexports::Tag::null() )
                    .to_tokens(ts);
            }
        }
    });


    let data_variant=ToTokenFnMut::new(|ts|{
        let ct=ctokens;

        match ( is_enum, prefix ) {
            (false,None)=>{
                let struct_=&ds.variants[0];

                to_stream!(ts;
                    ct.tl_data,ct.colon2,ct.struct_under
                );
                ct.paren.surround(ts,|ts|{
                    ct.and_.to_tokens(ts);
                    ct.bracket.surround(ts,|ts|{
                        fields_tokenizer(struct_,config,ct).to_tokens(ts);
                    })
                })
            }
            (true,None)=>{
                let variants=variants_tokenizer(&ds,config,ct);

                to_stream!(ts;
                    ct.tl_data,ct.colon2,ct.enum_under
                );
                ct.paren.surround(ts,|ts|{
                    ct.and_.to_tokens(ts);
                    ct.bracket.surround(ts,|ts| variants.to_tokens(ts) );
                });
            }
            (false,Some(prefix))=>{
                if is_transparent{
                    panic!("repr(transparent) prefix types not supported");
                }

                let struct_=&ds.variants[0];
                let first_suffix_field=prefix.first_suffix_field;
                let fields=fields_tokenizer(struct_,config,ct);
                
                quote!(
                    __TLData::prefix_type(
                        #first_suffix_field,
                        <#name #ty_generics as 
                            _sabi_reexports::PrefixTypeTrait
                        >::PT_FIELD_ACCESSIBILITY,
                        <#name #ty_generics as 
                            _sabi_reexports::PrefixTypeTrait
                        >::PT_COND_PREFIX_FIELDS,
                        &[#fields]
                    )
                ).to_tokens(ts);
            }
            (true,Some(_))=>{
                panic!("enum prefix types not supported");
            }
        };
    });

    
    let lifetimes=&generics.lifetimes().map(|x|&x.lifetime).collect::<Vec<_>>();
    let type_params=&generics.type_params().map(|x|&x.ident).collect::<Vec<_>>();
    let const_params=&generics.const_params().map(|x|&x.ident).collect::<Vec<_>>();
    
    let type_params_for_generics=
        type_params.iter().filter(|&x| !config.unconstrained_type_params.contains_key(x) );
    

    // For `type StaticEquivalent= ... ;`
    let lifetimes_s=lifetimes.iter().map(|_| &ctokens.static_lt );
    let type_params_s=ToTokenFnMut::new(|ts|{
        let ct=ctokens;

        for ty in type_params {
            if let Some(unconstrained)=config.unconstrained_type_params.get(ty) {
                unconstrained.static_equivalent
                    .unwrap_or(&ct.empty_tuple)
                    .to_tokens(ts);
            }else{
                to_stream!(ts; ct.static_equivalent, ct.lt, ty, ct.gt);
            }
            ct.comma.to_tokens(ts);
        }
    });
    let const_params_s=&const_params;


    let stringified_name=name.to_string();

    let module=Ident::new(&format!("_stable_abi_impls_for_{}",name),Span::call_site());

    let stable_abi_bounded =&config.stable_abi_bounded;
    let extra_bounds       =&config.extra_bounds;
    
    let inside_abi_stable_crate=if config.inside_abi_stable_crate { 
        quote!(pub(super) use crate as abi_stable;)
    }else{ 
        quote!(pub(super) use ::abi_stable;)
    };

    let import_group=if config.inside_abi_stable_crate { 
        quote!(crate)
    }else{ 
        quote!(abi_stable)
    };

    let prefix_type_tokenizer_=prefix_type_tokenizer(&module,&ds,config,ctokens);

    let mod_refl_mode=config.mod_refl_mode;

    quote!(
        #prefix_type_tokenizer_

        mod #module {
            use super::*;

            #inside_abi_stable_crate

            #[allow(unused_imports)]
            pub(super) use #import_group::derive_macro_reexports::{
                self as _sabi_reexports,
                renamed::*,
            };

            unsafe impl #impl_generics __SharedStableAbi for #impl_name #ty_generics 
            where 
                #(#where_clause,)*
                #(#stable_abi_bounded:__StableAbi,)*
                #(#extra_bounds,)*
                #prefix_type_trait_bound
            {
                type IsNonZeroType=_sabi_reexports::False;
                type Kind=#associated_kind;
                type StaticEquivalent=#impl_name < 
                    #(#lifetimes_s,)* 
                    #type_params_s
                    #(#const_params_s),* 
                >;

                const S_LAYOUT: &'static _sabi_reexports::TypeLayout = {
                    &_sabi_reexports::TypeLayout::from_params::<#size_align_for>(
                        {
                            __TypeLayoutParams {
                                name: #stringified_name,
                                package: env!("CARGO_PKG_NAME"),
                                package_version: abi_stable::package_version_strings!(),
                                file:file!(),
                                line:line!(),
                                data: #data_variant,
                                generics: abi_stable::tl_genparams!(
                                    #(#lifetimes),*;
                                    #(#type_params_for_generics),*;
                                    #(#const_params),*
                                ),
                            }
                        }
                    ).set_phantom_fields(&[])
                     .set_tag(#tags)
                     .set_mod_refl_mode(#mod_refl_mode)
                };
            }

        }
    ).observe(|tokens|{
        if config.debug_print {
            panic!("\n\n\n{}\n\n\n",tokens );
        }
    })
}

/// Outputs `value` wrapped in `stringify!( ... )` to a TokenStream .
fn stringified_token<'a,T>(ctokens:&'a CommonTokens<'a>,value:&'a T)->impl ToTokens+'a
where T:ToTokens
{
    ToTokenFnMut::new(move|ts|{
        to_stream!(ts; ctokens.stringify_,ctokens.bang );
        ctokens.paren.surround(ts,|ts| value.to_tokens(ts) );
    })
}

fn variants_tokenizer<'a>(
    ds:&'a DataStructure<'a>,
    config:&'a StableAbiOptions<'a>,
    ct:&'a CommonTokens<'a>
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{
        for variant in &ds.variants {
            to_stream!{ts;
                ct.tl_enum_variant,ct.colon2,ct.new
            }
            ct.paren.surround(ts,|ts|{
                stringified_token(ct,variant.name).to_tokens(ts);
                to_stream!(ts; ct.comma,ct.and_ );
                ct.bracket.surround(ts,|ts|{
                    fields_tokenizer(variant,config,ct).to_tokens(ts);
                })
            });
            to_stream!(ts; ct.comma );
        }
    })
}


/// Outputs:
///
/// `< #ty as MakeGetAbiInfo<#flavor> >::CONST`
fn make_get_abi_info_tokenizer<'a,T:'a>(
    ty:T,
    flavor:&'a Ident,
    ct:&'a CommonTokens<'a>,
)->impl ToTokens+'a
where T:ToTokens
{
    ToTokenFnMut::new(move|ts|{
        to_stream!{ts; 
            ct.lt,
                ty,ct.as_,
                ct.make_get_abi_info,ct.lt,flavor,ct.gt,
            ct.gt,
            ct.colon2,
            ct.cap_const
        };
    })
}


fn fields_tokenizer<'a>(
    struct_:&'a Struct<'a>,
    config:&'a StableAbiOptions<'a>,
    ctokens:&'a CommonTokens<'a>,
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{
        for field in &struct_.fields {
            field_tokenizer(struct_,field,config,ctokens)
                .to_tokens(ts);
        } 
    })
}

fn field_tokenizer<'a>(
    _struct_:&'a Struct<'a>,
    field:&'a Field<'a>,
    config:&'a StableAbiOptions<'a>,
    ctokens:&'a CommonTokens<'a>,
)->impl ToTokens+'a
{
    ToTokenFnMut::new(move|ts|{
        let ct=ctokens;

        to_stream!{ts; ct.tl_field,ct.colon2,ct.with_functions };
        ct.paren.surround(ts,|ts|{
            let name=config.renamed_fields
                .get(&(field as *const _))
                .cloned()
                .unwrap_or(field.ident());

            to_stream!(ts; name.to_string() ,ct.comma );
            to_stream!(ts; ct.and_ );
            ct.bracket.surround(ts,|ts|{
                for li in &field.referenced_lifetimes {
                    to_stream!{ts; li.tokenizer(ct),ct.comma }
                }
            });
            
            to_stream!{ts; ct.comma };

            let impls_sabi=true;
            let field_ptr:*const Field<'_>=field;
            let is_opaque_field=config.opaque_fields.contains(&field_ptr);

            let flavor=match (is_opaque_field,impls_sabi) {
                (false,false)=>&ct.stable_abi_bound,
                (false,true )=>&ct.stable_abi_bound,
                (true ,_    )=>&ct.unsafe_opaque_field_bound,
            };

            // println!(
            //     "field:`{}:{}` impls_sabi={} is_opaque_field={} ptr={:?}\n\
            //      opaque_fields:{:?}\n\
            //     ",
            //     field.ident(),
            //     (&field.mutated_ty).into_token_stream(),
            //     impls_sabi,
            //     is_opaque_field,
            //     field_ptr,
            //     config.opaque_fields,
            // );

            make_get_abi_info_tokenizer(&field.mutated_ty,flavor,ct).to_tokens(ts);

            to_stream!(ts;ct.comma);
            
            to_stream!(ts;ct.and_);
            ct.bracket.surround(ts,|ts|{
                for (fn_i,func) in field.functions.iter().enumerate() {

                    let pr_tokenizer=move|pr:&'a FnParamRet<'a>|{
                        ToTokenFnMut::new(move|ts|{
                            let field_name=pr.name;
                            let lifetime_refs=pr.lifetime_refs_tokenizer(ctokens);
                            
                            to_stream!{ts; ct.tl_field,ct.colon2,ct.new };
                            ct.paren.surround(ts,|ts|{
                                to_stream!(ts; field_name,ct.comma );

                                to_stream!(ts; ct.and_ );
                                ct.bracket.surround(ts,|ts| lifetime_refs.to_tokens(ts) );
                                
                                to_stream!(ts; ct.comma );

                                make_get_abi_info_tokenizer(
                                    pr.ty,
                                    &ct.stable_abi_bound,
                                    ct,
                                ).to_tokens(ts);
                            });
                        })
                    };

                    let fn_name=if field.is_function {
                        format!("fn_{}",fn_i)
                    }else{
                        field.ident().to_string()
                    };

                    let bound_lifetimes=func.named_bound_lts.iter().map(|x| x.to_string() );
                    
                    let params=func.params.iter().map(pr_tokenizer);

                    let returns=match func.returns.as_ref() {
                        Some(returns)=>{
                            let returns=pr_tokenizer(returns);
                            quote!( _sabi_reexports::RSome(#returns) )
                        },
                        None=>
                            quote!( _sabi_reexports::RNone ),
                    };

                    quote!(
                        __TLFunction::new(
                            #fn_name,

                            &[ #( __StaticStr::new(#bound_lifetimes) ),* ],

                            &[ #( #params ),* ],

                            #returns,
                        ),
                    ).to_tokens(ts);

                }

            });
            to_stream!{ts; ct.comma };

            field.is_function.to_tokens(ts);
        });
        to_stream!{ts; ct.comma }
    })
}