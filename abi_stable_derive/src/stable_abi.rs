

use crate::*;

use crate::{
    composite_collections::{
        SmallStartLen as StartLen,
        SmallCompositeString as CompositeString,
        SmallCompositeVec as CompositeVec,
    },
    datastructure::{DataStructure,DataVariant,Field,FieldIndex},
    gen_params_in::{GenParamsIn,InWhat},
    impl_interfacetype::impl_interfacetype_tokenizer,
    lifetimes::LifetimeIndex,
    to_token_fn::ToTokenFnMut,
    //utils::SynResultExt,
};


use syn::Ident;

use proc_macro2::{TokenStream as TokenStream2,Span};

use core_extensions::{
    prelude::*,

};


#[doc(hidden)]
pub mod reflection;

mod attribute_parsing;

mod common_tokens;

mod nonexhaustive;

mod prefix_types;

mod repr_attrs;

mod tl_function;

#[cfg(test)]
mod tests;

use self::{
    attribute_parsing::{
        parse_attrs_for_stable_abi, StabilityKind, StableAbiOptions, NotStableAbiBound,
        FieldTransparency,
    },
    common_tokens::CommonTokens,
    nonexhaustive::{tokenize_enum_info,tokenize_nonexhaustive_items},
    prefix_types::prefix_type_tokenizer,
    reflection::ModReflMode,
    tl_function::{VisitedFieldMap,CompTLFunction},
};


pub(crate) fn derive(mut data: DeriveInput) -> Result<TokenStream2,syn::Error> {
    data.generics.make_where_clause();

    // println!("\nderiving for {}",data.ident);
    // let _measure_time0=PrintDurationOnDrop::new(file_span!());

    let arenas = Arenas::default();
    let arenas = &arenas;
    let ctokens = CommonTokens::new(arenas);
    let ctokens = &ctokens;
    let ds = &DataStructure::new(&mut data, arenas);
    let config = &parse_attrs_for_stable_abi(ds.attrs, ds, arenas)?;
    let generics=ds.generics;
    let name=ds.name;

    let visited_fields=&VisitedFieldMap::new(ds,config,arenas,ctokens)?;

    let module=Ident::new(&format!("_sabi_{}",name),Span::call_site());


    // drop(_measure_time0);

    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause=(&where_clause.expect("BUG").predicates).into_iter();
    let where_clause_b=where_clause.clone();

    // The value of the `SharedStableAbi::Kind` associated type.
    let associated_kind = match config.kind {
        StabilityKind::Value|StabilityKind::NonExhaustive{..} => 
            &ctokens.value_kind,
        StabilityKind::Prefix{..}=>
            &ctokens.prefix_kind,
    };

    // The type that implements SharedStableAbi
    let impl_ty= match &config.kind {
        StabilityKind::Value => 
            quote!(#name #ty_generics ),
        StabilityKind::Prefix(prefix)=>{
            let n=&prefix.prefix_struct;
            quote!(#n #ty_generics )
        },
        StabilityKind::NonExhaustive(nonexhaustive)=>{
            let marker=nonexhaustive.nonexhaustive_marker;
            quote!(#marker < #name  #ty_generics , __Storage > )
        }
    };

    let mut prefix_type_trait_bound=None;
    let mut prefix_bounds:&[_]=&[];

    // The type whose size and alignment that is stored in the type layout.
    let size_align_for=match &config.kind {
        StabilityKind::NonExhaustive(_)=>{
            quote!(__Storage)
        },
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

    let is_transparent=config.repr.is_repr_transparent();
    let is_enum=ds.data_variant==DataVariant::Enum;
    let prefix=match &config.kind {
        StabilityKind::Prefix(prefix)=>Some(prefix),
        _=>None,
    };
    let nonexhaustive_opt=match &config.kind {
        StabilityKind::NonExhaustive(nonexhaustive)=>Some(nonexhaustive),
        _=>None,
    };


    let tags_opt=&config.tags;
    // tokenizes the `Tag` data structure associated with this type.
    let tags=ToTokenFnMut::new(move|ts|{
        ctokens.and_.to_tokens(ts);
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

    
    let extra_checks=
        match &config.extra_checks {
            Some(extra_checks)=>quote!( Some(_sabi_reexports::Constructor(#extra_checks)) ),
            None=>quote!( None ),
        };

    
    // tokenizes the items for nonexhaustive enums outside of the module this generates.
    let nonexhaustive_items=tokenize_nonexhaustive_items(&module,ds,config,ctokens);

    // tokenizes the items for nonexhaustive enums inside of the module this generates.
    let nonexhaustive_tokens=tokenize_enum_info(ds,config,ctokens)?;



    let first_field=FieldIndex{variant:0,pos:0};
    let is_nonzero=if is_transparent && visited_fields.map.contains_index(first_field) {
        let visited_field=&visited_fields.map[first_field];

        let is_opaque_field=config.opaque_fields[first_field].is_opaque();
        if visited_field.is_function {
            quote!( _sabi_reexports::True )
        }else if is_opaque_field {
            quote!( _sabi_reexports::False )
        }else{
            let ty=&visited_field.mutated_ty;
            quote!( <#ty as __SharedStableAbi>::IsNonZeroType )
        }
    }else{
        quote!( _sabi_reexports::False )
    };


    // The tokenizer for the TLData stored in the TypeLayout
    let data_variant=ToTokenFnMut::new(|ts|{
        let ct=ctokens;

        match ( is_enum, prefix ) {
            (false,None)=>{
                let struct_=&ds.variants[0];

                let variant_lengths=&[ struct_.fields.len() as u8 ];

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
                        visited_fields,
                        ct
                    ).to_tokens(ts);
                })
            }
            (true,None)=>{
                quote!(__TLData::Enum).to_tokens(ts);
                ct.paren.surround(ts,|ts|{
                    tokenize_enum(ds,nonexhaustive_opt,config,visited_fields,ctokens).to_tokens(ts);
                });
            }
            (false,Some(prefix))=>{
                if is_transparent{
                    spanned_err!(name,"repr(transparent) prefix types not supported")
                        .to_compile_error()
                        .to_tokens(ts);
                    return;
                }

                let struct_=&ds.variants[0];
                let variant_lengths=&[ struct_.fields.len() as u8 ];
                let first_suffix_field=prefix.first_suffix_field.field_pos;
                let fields=struct_.fields.iter();
                let fields=fields_tokenizer(ds,fields,variant_lengths,config,visited_fields,ct);
                
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
                spanned_err!(name,"enum prefix types not supported")
                    .to_compile_error()
                    .to_tokens(ts);
                return;
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
            match config.unconstrained_type_params.get(ty) {
                Some(NotStableAbiBound::NoBound)=>{
                    ct.empty_tuple.to_tokens(ts);
                }
                Some(NotStableAbiBound::GetStaticEquivalent)|None=>{
                    to_stream!(ts; ct.static_equivalent, ct.lt, ty, ct.gt);
                }
            }
            ct.comma.to_tokens(ts);
        }
    });
    let const_params_s=&const_params;

    // The name of the struct this generates,
    // to use as the `GetStaticEquivalent_::StaticEquivalent` associated type.
    let static_struct_name=Ident::new(&format!("_static_{}",name),Span::call_site());
    
    let item_info_const=Ident::new(&format!("_item_info_const_{}",name),Span::call_site());

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

    // if the `#[sabi(impl_InterfaceType())]` attribute was used:
    // tokenizes the implementation of `InterfaceType` for `#name #ty_params`
    let interfacetype_tokenizer=
        impl_interfacetype_tokenizer(
            ds.name,
            ds.generics,
            config.impl_interfacetype.as_ref(),
        );


    let stringified_name=name.to_string();

    let stable_abi_bounded =(&config.stable_abi_bounded).into_iter();
    let stable_abi_bounded_b=stable_abi_bounded.clone();
    let static_equiv_bounded =&config.static_equiv_bounded;
    let extra_bounds       =&config.extra_bounds;
    
    let prefix_type_tokenizer_=prefix_type_tokenizer(&module,&ds,config,ctokens)?;

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
    let phantom_field_tys_b=phantom_field_tys.clone();

    let phantom_type_params=&config.phantom_type_params;

    // let _measure_time1=PrintDurationOnDrop::new(file_span!());

    // The storage type parameter that is added if this is a nonexhaustive enum.
    let storage_opt=nonexhaustive_opt.map(|_| &ctokens.und_storage );
    let generics_header=
        GenParamsIn::with_after_types(&ds.generics,InWhat::ImplHeader,storage_opt);


    quote!(
        #prefix_type_tokenizer_

        #nonexhaustive_items

        const #item_info_const:abi_stable::type_layout::ItemInfo=
            abi_stable::make_item_info!();

        mod #module {
            use super::*;

            pub(super) use ::abi_stable;

            #[allow(unused_imports)]
            pub(super) use ::abi_stable::derive_macro_reexports::{
                self as _sabi_reexports,
                renamed::*,
            };

            #static_struct_decl

            #nonexhaustive_tokens

            #interfacetype_tokenizer

            unsafe impl <#generics_header> __GetStaticEquivalent_ for #impl_ty 
            where 
                #(#where_clause,)*
                #(#stable_abi_bounded:__StableAbi,)*
                #(#static_equiv_bounded:__GetStaticEquivalent_,)*
                #(#extra_bounds,)*
                #(#prefix_bounds,)*
                #prefix_type_trait_bound
            {
                type StaticEquivalent=#static_struct_name < 
                    #(#lifetimes_s,)*
                    #type_params_s
                    #(#const_params_s),* 
                >;
            }

            unsafe impl <#generics_header> __SharedStableAbi for #impl_ty 
            where 
                #(#where_clause_b,)*
                #(#stable_abi_bounded_b:__StableAbi,)*
                #(#phantom_field_tys_b:__SharedStableAbi,)*
                #(#static_equiv_bounded:__GetStaticEquivalent_,)*
                #(#extra_bounds,)*
                #(#prefix_bounds,)*
                #prefix_type_trait_bound
            {
                type IsNonZeroType=#is_nonzero;
                type Kind=#associated_kind;

                const S_LAYOUT: &'static _sabi_reexports::TypeLayout = {
                    &_sabi_reexports::TypeLayout::from_derive::<#size_align_for>(
                        __private_TypeLayoutDerive {
                            abi_consts: Self::S_ABI_CONSTS,
                            name: #stringified_name,
                            item_info:#item_info_const,
                            data: #data_variant,
                            generics: abi_stable::tl_genparams!(
                                #(#lifetimes),*;
                                #(#type_params_for_generics,)*
                                #(#phantom_type_params,)*;
                                #(#const_params),*
                            ),
                            phantom_fields:&[
                                #(
                                    __TLField::new(
                                        #phantom_field_names,
                                        &[],
                                        __GetTypeLayoutCtor::<
                                            #phantom_field_tys
                                        >::SHARED_STABLE_ABI,
                                    ),
                                )*
                            ],
                            tag:#tags,
                            extra_checks:#extra_checks,
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
    .piped(Ok)
}

// Tokenizes a `TLEnum{ .. }`
fn tokenize_enum<'a>(
    ds:&'a DataStructure<'a>,
    nonexhaustive_opt:Option<&'a nonexhaustive::NonExhaustive<'a>>,
    config:&'a StableAbiOptions<'a>,
    visited_fields:&'a VisitedFieldMap<'a>,
    ct:&'a CommonTokens<'a>
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{
        let mut variant_names=String::new();
        for variant in &ds.variants {
            use std::fmt::Write;
            let _=write!(variant_names,"{};",variant.name);
        }

        let is_exhaustive=match nonexhaustive_opt {
            Some(_)=>{
                let name=ds.name;
                let (_, ty_generics,_) = ds.generics.split_for_impl();
                quote!(nonexhaustive(
                    &_sabi_reexports::TLNonExhaustive::new::< #name #ty_generics >()
                ))
            },
            None=>quote!(exhaustive()),
        };

        let variant_lengths=&ds.variants.iter()
            .map(|x|{
                assert!(x.fields.len() < 256,"variant '{}' has more than 255 fields.",x.name);
                x.fields.len() as u8
            })
            .collect::<Vec<u8>>();

        let fields=ds.variants.iter().flat_map(|v| v.fields.iter() );
        let fields=fields_tokenizer(ds,fields,variant_lengths,config,visited_fields,ct);

        let discriminants=ds.variants.iter().map(|x|x.discriminant);
        let discriminants=config.repr.tokenize_discriminant_exprs(discriminants,ct);


        quote!(
            &__TLEnum::for_derive(
                #variant_names,
                __IsExhaustive::#is_exhaustive,
                #fields,
                #discriminants,
                &[#( #variant_lengths ),*],
            )
        ).to_tokens(ts);
    })
}


#[must_use]
fn make_get_type_layout_tokenizer<'a,T:'a>(
    ty:T,
    field_transparency:FieldTransparency,
    ct:&'a CommonTokens<'a>,
)->impl ToTokens+'a
where T:ToTokens
{
    ToTokenFnMut::new(move|ts|{
        to_stream!{ts; 
            ct.get_type_layout_ctor,
            ct.colon2,
            ct.lt,ty,ct.gt,
            ct.colon2,
        };
        match field_transparency {
            FieldTransparency::Regular=> &ct.cap_stable_abi,
            FieldTransparency::Opaque=> &ct.cap_opaque_field,
            FieldTransparency::SabiOpaque=> &ct.cap_sabi_opaque_field,
        }.to_tokens(ts);
    })
}


/// Tokenizes a TLFields,
fn fields_tokenizer<'a>(
    ds:&'a DataStructure<'a>,
    mut fields:impl Iterator<Item=&'a Field<'a>>+'a,
    variant_length:&'a [u8],
    config:&'a StableAbiOptions<'a>,
    visited_fields:&'a VisitedFieldMap<'a>,
    ctokens:&'a CommonTokens<'a>,
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{
        to_stream!(ts;ctokens.tl_fields,ctokens.colon2,ctokens.new);
        ctokens.paren.surround(ts,|ts|{
            let fields=fields.by_ref().collect::<Vec<_>>();
            fields_tokenizer_inner(ds,fields,variant_length,config,visited_fields,ctokens,ts);
        });
    })
}

fn fields_tokenizer_inner<'a>(
    ds:&'a DataStructure<'a>,
    fields:Vec<&'a Field<'a>>,
    variant_length:&'a [u8],
    config:&'a StableAbiOptions<'a>,
    visited_fields:&'a VisitedFieldMap<'a>,
    ct:&'a CommonTokens<'a>,
    ts:&mut TokenStream2,
){

    let mut names=String::new();

    let mut lifetime_ind_pos:Vec<(FieldIndex,u16)>=Vec::new();

    let mut current_lt_index=0_u16;
    for &field in &fields {
        use std::fmt::Write;
        let visited_field=&visited_fields.map[field];
        let name=config.renamed_fields[field].unwrap_or(field.ident());
        let _=write!(names,"{};",name);

        lifetime_ind_pos.push((
            field.index,
            current_lt_index,
        ));
        current_lt_index+=visited_field.referenced_lifetimes.len() as u16;
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
            for li in fields.iter().cloned()
                .flat_map(|f| &visited_fields.map[f].referenced_lifetimes ) 
            {
                to_stream!(ts;li.tokenizer(ct.as_ref()),ct.comma);
            }
        });
        ct.comma.to_tokens(ts);
        ct.and_.to_tokens(ts);
        ct.bracket.surround(ts,|ts|{
            for (fi,index) in lifetime_ind_pos {
                to_stream!(ts;ct.with_field_index,ct.colon2,ct.from_vari_field_val);
                ct.paren.surround(ts,|ts|{
                    to_stream!(ts;fi.variant as u16,ct.comma,fi.pos as u8,ct.comma,index)
                });
                to_stream!(ts;ct.comma);
            }
        });
    });
    ct.comma.to_tokens(ts);


    if visited_fields.fn_ptr_count==0 {
        ct.none.to_tokens(ts);
    }else{
        to_stream!(ts;ct.some);
        ct.paren.surround(ts,|ts|{
            ct.and_.to_tokens(ts);
            tokenize_tl_functions(ds,&fields,&visited_fields,variant_length,config,ct,ts);
        });
    }
    to_stream!{ts; ct.comma };


    to_stream!{ts; ct.and_ };
    ct.bracket.surround(ts,|ts|{
        for &field in &fields {
            let visited_field=&visited_fields.map[field];

            let field_accessor=config.override_field_accessor[field]
                .unwrap_or_else(|| config.kind.field_accessor(config.mod_refl_mode,field) );

            to_stream!(ts;ct.field_1to1,ct.colon2,ct.new);
            ct.paren.surround(ts,|ts|{
                {//layout:

                    if visited_field.is_function {
                        if visited_field.functions[0].is_unsafe {
                            &ct.unsafe_extern_fn_type_layout
                        }else{
                            &ct.extern_fn_type_layout
                        }.to_tokens(ts);
                    }else{
                        make_get_type_layout_tokenizer(
                            &visited_field.mutated_ty,
                            config.opaque_fields[field],
                            ct
                        ).to_tokens(ts);
                    }

                    to_stream!(ts;ct.comma);
                }

                to_stream!(ts;visited_field.is_function,ct.comma);
                to_stream!(ts;field_accessor,ct.comma);
            });
            to_stream!(ts;ct.comma);
        }
    });
}

/// Tokenizes a TLFunctions
fn tokenize_tl_functions<'a>(
    ds:&'a DataStructure<'a>,
    fields:&[&'a Field<'a>],
    visited_fields:&'a VisitedFieldMap<'a>,
    _variant_length:&[u8],
    config:&'a StableAbiOptions<'a>,
    ct:&'a CommonTokens<'a>,
    ts:&mut TokenStream2,
){
    let mut strings=CompositeString::new();
    let mut functions=CompositeVec::<CompTLFunction>::with_capacity(visited_fields.fn_ptr_count);
    let mut field_fn_ranges=Vec::<StartLen>::with_capacity(ds.field_count);
    let mut type_layouts=CompositeVec::<( FieldTransparency, &'a syn::Type )>::new();
    let mut paramret_lifetime_indices=CompositeVec::<LifetimeIndex>::new();

    for &field in fields {
        let field_transparency=config.opaque_fields[field];
        let visited_field=&visited_fields.map[field];

        let field_fns=visited_field.functions.iter().enumerate()
            .map(|(fn_i,func)|{
                let mut current_func=CompTLFunction::new(ct);
                
                current_func.name=if visited_field.is_function {
                    strings.push_display(field.ident())
                }else{
                    strings.push_str(&format!("fn_{}",fn_i))
                };

                current_func.bound_lifetimes=strings
                    .extend_with_display(";",func.named_bound_lts.iter());

                current_func.param_names=strings
                    .extend_with_display(";",func.params.iter().map(|p| p.name.unwrap_or("") ));

                current_func.param_type_layouts=type_layouts
                    .extend( func.params.iter().map(|p| ( field_transparency, p.ty ) ) );

                current_func.paramret_lifetime_indices=paramret_lifetime_indices
                    .extend( 
                        func.params.iter()
                            .chain(&func.returns)
                            .flat_map(|p| p.lifetime_refs.iter().cloned() ) 
                    );

                if let Some(returns)=&func.returns {
                    current_func.return_type_layout=
                        Some( type_layouts.push(( field_transparency, returns.ty )) );
                }
                current_func
            });

        field_fn_ranges.push( functions.extend(field_fns) )
    }

    let strings=strings.into_inner();

    let functions=functions.into_inner();

    let field_fn_ranges=field_fn_ranges.into_iter().map(|sl| sl.tokenizer(ct.as_ref()) );

    let type_layouts=type_layouts.into_inner();
    let type_layouts=ToTokenFnMut::new(|ts|{
        for ( field_transparency, ty ) in &type_layouts {
            make_get_type_layout_tokenizer(ty,*field_transparency,ct).to_tokens(ts);
            ct.comma.to_tokens(ts);
        }
    });

    let paramret_lifetime_indices=paramret_lifetime_indices.into_inner().into_iter()
        .map(|sl| sl.tokenizer(ct.as_ref()) );


    quote!(
        __TLFunctions::new(
            #strings,
            &[#(#functions),*],
            &[#(#field_fn_ranges),*],
            &[#type_layouts],
            &[#(#paramret_lifetime_indices),*],
        )
    ).to_tokens(ts);

}



