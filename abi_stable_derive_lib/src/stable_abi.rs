

use crate::*;

use crate::{
    datastructure::{DataStructure,DataVariant,Struct,Field},
    to_token_fn::ToTokenFnMut,
    fn_pointer_extractor::{FnParamRet},
};


use syn::Ident;

use proc_macro2::{TokenStream as TokenStream2,Span};

use core_extensions::{
    prelude::*,

};


#[doc(hidden)]
pub mod reflection;

mod attribute_parsing;

mod prefix_types;

mod repr_attrs;

#[cfg(test)]
mod tests;

use self::{
    attribute_parsing::{parse_attrs_for_stable_abi, StabilityKind,StableAbiOptions},
    prefix_types::prefix_type_tokenizer,
    repr_attrs::ReprAttr,
    reflection::ModReflMode,
};


pub(crate) fn derive(mut data: DeriveInput) -> TokenStream2 {
    data.generics.make_where_clause();

    // println!("\nderiving for {}",data.ident);
    // let _measure_time0=PrintDurationOnDrop::new(file_span!());

    let arenas = Arenas::default();
    let arenas = &arenas;
    let ctokens = CommonTokens::new(arenas);
    let ctokens = &ctokens;
    let ds = DataStructure::new(&mut data, arenas, ctokens);
    let config = &parse_attrs_for_stable_abi(ds.attrs, &ds, arenas);
    let generics=ds.generics;
    let name=ds.name;

    // drop(_measure_time0);

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
    
    let repr=config.repr;

    let is_transparent=config.repr==ReprAttr::Transparent ;
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

                to_stream!(ts;ct.tl_data,ct.colon2);
                match ds.data_variant {
                    DataVariant::Struct=>&ct.struct_under,
                    DataVariant::Union=>&ct.union_under,
                    DataVariant::Enum=>unreachable!(),
                }.to_tokens(ts);
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
                let first_suffix_field=prefix.first_suffix_field.field_pos;
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


    let static_struct_name=Ident::new(&format!("_static_{}",name),Span::call_site());

    let static_struct_decl={
        let const_param_name=generics.const_params().map(|c| &c.ident );
        let const_param_type=generics.const_params().map(|c| &c.ty );

        let lifetimes_a  =lifetimes  ;
        
        let type_params_a=type_params;
        

        quote!{
            pub struct #static_struct_name<
                #(#lifetimes_a,)*
                #(#type_params_a,)*
                #(const #const_param_name:#const_param_type,)*
            >(
                #(& #lifetimes_a (),)*
                #(#type_params_a,)*
            );
        }
    };


    let stringified_name=name.to_string();

    let module=Ident::new(&format!("_sabi_{}",name),Span::call_site());

    let stable_abi_bounded =&config.stable_abi_bounded;
    let extra_bounds       =&config.extra_bounds;
    
    let prefix_type_tokenizer_=prefix_type_tokenizer(&module,&ds,config,ctokens);

    let mod_refl_mode=match config.mod_refl_mode {
        ModReflMode::Module=>quote!( __ModReflMode::Module ),
        ModReflMode::Opaque=>quote!( __ModReflMode::Opaque ),
        ModReflMode::DelegateDeref(field_index)=>{
            quote!(
                __ModReflMode::DelegateDeref{
                    phantom_field_index:#field_index
                }
            )
        }
    };

    let phantom_field_names=config.phantom_fields.iter().map(|x| x.0 );
    let phantom_field_tys  =config.phantom_fields.iter().map(|x| x.1 );

    // let _measure_time1=PrintDurationOnDrop::new(file_span!());

    quote!(
        #prefix_type_tokenizer_

        mod #module {
            use super::*;

            pub(super) use ::abi_stable;

            #[allow(unused_imports)]
            pub(super) use ::abi_stable::derive_macro_reexports::{
                self as _sabi_reexports,
                renamed::*,
            };

            #static_struct_decl

            unsafe impl #impl_generics __SharedStableAbi for #impl_name #ty_generics 
            where 
                #(#where_clause,)*
                #(#stable_abi_bounded:__StableAbi,)*
                #(#extra_bounds,)*
                #prefix_type_trait_bound
            {
                type IsNonZeroType=_sabi_reexports::False;
                type Kind=#associated_kind;
                type StaticEquivalent=#static_struct_name < 
                    #(#lifetimes_s,)*
                    #type_params_s
                    #(#const_params_s),* 
                >;

                const S_LAYOUT: &'static _sabi_reexports::TypeLayout = {
                    &_sabi_reexports::TypeLayout::from_derive::<#size_align_for>(
                        __private_TypeLayoutDerive {
                            name: #stringified_name,
                            item_info:abi_stable::make_item_info!(),
                            data: #data_variant,
                            generics: abi_stable::tl_genparams!(
                                #(#lifetimes),*;
                                #(#type_params_for_generics),*;
                                #(#const_params),*
                            ),
                            phantom_fields:&[
                                #(
                                    __TLField::new(
                                        #phantom_field_names,
                                        &[],
                                        <#phantom_field_tys as 
                                            __MakeGetAbiInfo<__StableAbi_Bound>
                                        >::CONST,
                                    ),
                                )*
                            ],
                            tag:#tags,
                            mod_refl_mode:#mod_refl_mode,
                            repr_attr:#repr,
                        }
                    )
                };
            }

        }
    ).observe(|tokens|{
        // drop(_measure_time1);
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

            to_stream!(ts; config.repr.tokenize_discriminant_expr(variant.discriminant) );

            to_stream!(ts; ct.comma );
        }
    })
}


/// Outputs the StableAbi constant.
fn make_get_abi_info_tokenizer<'a,T:'a>(
    ty:T,
    ct:&'a CommonTokens<'a>,
)->impl ToTokens+'a
where T:ToTokens
{
    ToTokenFnMut::new(move|ts|{
        to_stream!{ts; 
            ct.make_get_abi_info_sa,
            ct.colon2,
            ct.lt,ty,ct.gt,
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
            let name=config.renamed_fields[field].unwrap_or(field.ident());

            to_stream!(ts; name.to_string() ,ct.comma );
            to_stream!(ts; ct.and_ );
            ct.bracket.surround(ts,|ts|{
                for li in &field.referenced_lifetimes {
                    to_stream!{ts; li.tokenizer(ct),ct.comma }
                }
            });
            
            to_stream!{ts; ct.comma };

            


            if field.is_function {
                if field.functions[0].is_unsafe {
                    &ct.unsafe_extern_fn_abi_info
                }else{
                    &ct.extern_fn_abi_info
                }.to_tokens(ts);
            }else{
                make_get_abi_info_tokenizer(&field.mutated_ty,ct).to_tokens(ts);
            }

            to_stream!(ts;ct.comma);
            
            to_stream!(ts;ct.and_);
            ct.bracket.surround(ts,|ts|{
                for (fn_i,func) in field.functions.iter().enumerate() {


                    let fn_name=if field.is_function {
                        field.ident().to_string()
                    }else{
                        format!("fn_{}",fn_i)
                    };

                    let bound_lifetimes=func.named_bound_lts.iter().map(|x| x.to_string() );
                    
                    let param_names:String=func.params.iter()
                        .map(|p| p.name.unwrap_or("") )
                        .collect::<Vec<&str>>()
                        .join(";");

                    let param_abi_infos=func.params.iter()
                        .map(|p|{
                            make_get_abi_info_tokenizer(p.ty,ct)
                        });

                    let paramret_lifetime_indices=func.params.iter()
                        .map(|p| p.lifetime_refs_tokenizer(ct) );

                    let returns=match func.returns.as_ref() {
                        Some(returns)=>{
                            let returns=make_get_abi_info_tokenizer(returns.ty,ct);
                            quote!( _sabi_reexports::RSome(#returns) )
                        },
                        None=>
                            quote!( _sabi_reexports::RNone ),
                    };

                    quote!(
                        __TLFunction::new(
                            #fn_name,

                            &[ #( __StaticStr::new(#bound_lifetimes) ),* ],

                            #param_names,

                            &[ #( #param_abi_infos ),* ],

                            &[ #(#paramret_lifetime_indices)* ],

                            #returns,
                        ),
                    ).to_tokens(ts);

                }

            });
            to_stream!{ts; ct.comma };

            field.is_function.to_tokens(ts);
        });

        let field_acc=config.override_field_accessor[field]
            .unwrap_or_else(|| config.kind.field_accessor(config.mod_refl_mode,field) );
            
        quote!( .set_field_accessor(#field_acc) ).to_tokens(ts);

        to_stream!{ts; ct.comma }
    })
}