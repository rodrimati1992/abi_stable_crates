

use crate::*;

use crate::{
    datastructure::{DataStructure,DataVariant,Struct,Field,FieldIndex},
    lifetimes::LifetimeIndex,
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

mod tl_functions;

#[cfg(test)]
mod tests;

use self::{
    attribute_parsing::{parse_attrs_for_stable_abi, StabilityKind,StableAbiOptions},
    prefix_types::prefix_type_tokenizer,
    repr_attrs::ReprAttr,
    reflection::ModReflMode,
    tl_functions::{StartLen,CompTLFunction,TLFunctionsString,TLFunctionsVec},
};


pub(crate) fn derive(mut data: DeriveInput) -> TokenStream2 {
    data.generics.make_where_clause();

    // println!("\nderiving for {}",data.ident);
    // let _measure_time0=PrintDurationOnDrop::new(file_span!());

    let arenas = Arenas::default();
    let arenas = &arenas;
    let ctokens = CommonTokens::new(arenas);
    let ctokens = &ctokens;
    let ds = &DataStructure::new(&mut data, arenas, ctokens);
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
    let mut prefix_bounds:&[_]=&[];

    let size_align_for=match &config.kind {
        StabilityKind::Prefix(prefix)=>{
            let prefix_struct=prefix.prefix_struct;

            prefix_type_trait_bound=Some(quote!(
                #name #ty_generics:_sabi_reexports::PrefixTypeTrait,
            ));
            prefix_bounds=&prefix.prefix_bounds;

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

                let variant_lengths=vec![ struct_.fields.len() as u16 ];

                to_stream!(ts;ct.tl_data,ct.colon2);
                match ds.data_variant {
                    DataVariant::Struct=>&ct.struct_under,
                    DataVariant::Union=>&ct.union_under,
                    DataVariant::Enum=>unreachable!(),
                }.to_tokens(ts);
                ct.paren.surround(ts,|ts|{
                    fields_tokenizer(
                        ds,
                        struct_.fields.iter(),
                        variant_lengths,
                        config,
                        ct
                    ).to_tokens(ts);
                })
            }
            (true,None)=>{
                let variants=variants_tokenizer(&ds,config,ct);

                let variant_lengths=ds.variants.iter()
                    .map(|x|x.fields.len() as u16)
                    .collect::<Vec<u16>>();

                to_stream!(ts;
                    ct.tl_data,ct.colon2,ct.enum_under
                );
                ct.paren.surround(ts,|ts|{
                    let fields=ds.variants.iter().flat_map(|v| v.fields.iter() );
                    fields_tokenizer(ds,fields,variant_lengths,config,ct).to_tokens(ts);
                    ct.comma.to_tokens(ts);
                    ct.and_.to_tokens(ts);
                    ct.bracket.surround(ts,|ts| variants.to_tokens(ts) );
                });
            }
            (false,Some(prefix))=>{
                if is_transparent{
                    panic!("repr(transparent) prefix types not supported");
                }

                let struct_=&ds.variants[0];
                let variant_lengths=vec![ struct_.fields.len() as u16 ];
                let first_suffix_field=prefix.first_suffix_field.field_pos;
                let fields=fields_tokenizer(ds,struct_.fields.iter(),variant_lengths,config,ct);
                
                quote!(
                    __TLData::prefix_type_derive(
                        #first_suffix_field,
                        <#name #ty_generics as 
                            _sabi_reexports::PrefixTypeTrait
                        >::PT_FIELD_ACCESSIBILITY,
                        <#name #ty_generics as 
                            _sabi_reexports::PrefixTypeTrait
                        >::PT_COND_PREFIX_FIELDS,
                        #fields
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
                #(#type_params_a:?Sized,)*
                #(const #const_param_name:#const_param_type,)*
            >(
                #(& #lifetimes_a (),)*
                extern fn(#(&#type_params_a,)*)
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
                #(#prefix_bounds,)*
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
                ct.comma.to_tokens(ts);
                variant.fields.len().to_tokens(ts);
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
    ds:&'a DataStructure<'a>,
    mut fields:impl Iterator<Item=&'a Field<'a>>+'a,
    variant_length:Vec<u16>,
    config:&'a StableAbiOptions<'a>,
    ctokens:&'a CommonTokens<'a>,
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{
        to_stream!(ts;ctokens.tl_fields,ctokens.colon2,ctokens.new);
        ctokens.paren.surround(ts,|ts|{
            let fields=fields.by_ref().collect::<Vec<_>>();
            fields_tokenizer_inner(ds,fields,&variant_length,config,ctokens,ts);
        });
    })
}


fn fields_tokenizer_inner<'a>(
    ds:&'a DataStructure<'a>,
    fields:Vec<&'a Field<'a>>,
    variant_length:&[u16],
    config:&'a StableAbiOptions<'a>,
    ct:&'a CommonTokens<'a>,
    ts:&mut TokenStream2,
){

    let mut names=String::new();

    let mut lifetime_ind_pos:Vec<(FieldIndex,usize)>=Vec::new();

    let mut current_lt_index=0_usize;
    for field in &fields {
        use std::fmt::Write;
        let name=config.renamed_fields[field].unwrap_or(field.ident());
        let _=write!(names,"{};",name);

        lifetime_ind_pos.push((
            field.index,
            current_lt_index,
        ));
        current_lt_index+=field.referenced_lifetimes.len();
    }

    names.to_tokens(ts);
    ct.comma.to_tokens(ts);

    ct.and_.to_tokens(ts);
    ct.bracket.surround(ts,|ts|{
        for len in variant_length.iter().cloned() {
            to_stream!(ts;len,ct.comma);
        }
    });
    ct.comma.to_tokens(ts);

    to_stream!(ts;ct.slice_and_field_indices,ct.colon2,ct.new);
    ct.paren.surround(ts,|ts|{
        ct.and_.to_tokens(ts);
        ct.bracket.surround(ts,|ts|{
            for li in fields.iter().flat_map(|f| &f.referenced_lifetimes ) {
                to_stream!(ts;li.tokenizer(ct),ct.comma);
            }
        });
        ct.comma.to_tokens(ts);
        ct.and_.to_tokens(ts);
        ct.bracket.surround(ts,|ts|{
            for (fi,index) in lifetime_ind_pos {
                to_stream!(ts;ct.with_field_index,ct.colon2,ct.from_vari_field_val);
                ct.paren.surround(ts,|ts|{
                    to_stream!(ts;fi.variant as u16,ct.comma,fi.pos as u16,ct.comma,index)
                });
                to_stream!(ts;ct.comma);
            }
        });
    });
    ct.comma.to_tokens(ts);


    if ds.fn_ptr_count==0 {
        ct.none.to_tokens(ts);
    }else{
        to_stream!(ts;ct.some);
        ct.paren.surround(ts,|ts|{
            ct.and_.to_tokens(ts);
            tokenize_tl_functions(ds,&fields,variant_length,config,ct,ts);
        });
    }
    to_stream!{ts; ct.comma };


    to_stream!{ts; ct.and_ };
    ct.bracket.surround(ts,|ts|{
        for field in &fields {

            let field_accessor=config.override_field_accessor[field]
                .unwrap_or_else(|| config.kind.field_accessor(config.mod_refl_mode,field) );

            to_stream!(ts;ct.field_1to1,ct.colon2,ct.new);
            ct.paren.surround(ts,|ts|{
                {//abi_info:
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
                }

                to_stream!(ts;field.is_function,ct.comma);
                to_stream!(ts;field_accessor,ct.comma);
            });
            to_stream!(ts;ct.comma);
        }
    });
}


fn tokenize_tl_functions<'a>(
    ds:&'a DataStructure<'a>,
    fields:&[&'a Field<'a>],
    _variant_length:&[u16],
    _config:&'a StableAbiOptions<'a>,
    ct:&'a CommonTokens<'a>,
    ts:&mut TokenStream2,
){
    let mut strings=TLFunctionsString::new();
    let mut functions=TLFunctionsVec::<CompTLFunction>::with_capacity(ds.fn_ptr_count);
    let mut field_fn_ranges=Vec::<StartLen>::with_capacity(ds.field_count);
    let mut abi_infos=TLFunctionsVec::<&'a syn::Type>::new();
    let mut paramret_lifetime_indices=TLFunctionsVec::<LifetimeIndex>::new();

    for field in fields {
        let field_fns=field.functions.iter().enumerate()
            .map(|(fn_i,func)|{
                let mut current_func=CompTLFunction::new(ct);
                
                current_func.name=if field.is_function {
                    strings.push_display(field.ident())
                }else{
                    strings.push_str(&format!("fn_{}",fn_i))
                };

                current_func.bound_lifetimes=strings
                    .extend_with_display(";",func.named_bound_lts.iter());

                current_func.param_names=strings
                    .extend_with_display(";",func.params.iter().map(|p| p.name.unwrap_or("") ));

                current_func.param_abi_infos=abi_infos
                    .extend( func.params.iter().map(|p| p.ty ) );

                current_func.paramret_lifetime_indices=paramret_lifetime_indices
                    .extend( 
                        func.params.iter()
                            .chain(&func.returns)
                            .flat_map(|p| p.lifetime_refs.iter().cloned() ) 
                    );

                if let Some(returns)=&func.returns {
                    current_func.return_abi_info=Some( abi_infos.push(returns.ty) );
                }
                current_func
            });

        field_fn_ranges.push( functions.extend(field_fns) )
    }

    let strings=strings.into_inner();

    let functions=functions.into_inner();

    let field_fn_ranges=field_fn_ranges.into_iter().map(|sl| sl.tokenizer(ct) );

    let abi_infos=abi_infos.into_inner().into_iter()
        .map(|ty| make_get_abi_info_tokenizer(ty,ct) );

    let paramret_lifetime_indices=paramret_lifetime_indices.into_inner().into_iter()
        .map(|sl| sl.tokenizer(ct) );


    quote!(
        __TLFunctions::new(
            #strings,
            &[#(#functions),*],
            &[#(#field_fn_ranges),*],
            &[#(#abi_infos),*],
            &[#(#paramret_lifetime_indices),*],
        )
    ).to_tokens(ts);

}



/*
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
                __WithFieldIndex::from_vari_field_val(
                    #variant,
                    #field_index,
                    __TLFunction::new(
                        #fn_name,

                        &[ #( __StaticStr::new(#bound_lifetimes) ),* ],

                        #param_names,

                        &[ #( #param_abi_infos ),* ],

                        &[ #(#paramret_lifetime_indices)* ],

                        #returns,
                    )
                ),
            ).to_tokens(ts);

*/