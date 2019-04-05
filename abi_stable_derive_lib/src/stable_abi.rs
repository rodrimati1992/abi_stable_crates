use std::ptr;

use crate::*;

use crate::{
    attribute_parsing::{parse_attrs_for_stable_abi, StabilityKind,StableAbiOptions,Repr},
    datastructure::{DataStructure,DataVariant,Struct,Field},
    to_token_fn::ToTokenFnMut,
    lifetimes::LifetimeTokenizer,
    fn_pointer_extractor::ParamOrReturn,
};

use hashbrown::HashSet;

use syn::Ident;

use proc_macro2::{TokenStream as TokenStream2,Span};

use core_extensions::{
    prelude::*,
    iter_cloner,
};

use arrayvec::ArrayString;




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

    let impld_trait = match config.kind {
        StabilityKind::Value => &ctokens.stable_abi,
        StabilityKind::Prefix => &ctokens.shared_stable_abi,
    };


    let function_fields=ToTokenFnMut::new(|ts|{
        use std::fmt::Write;
        let ct=ctokens;

        let mut buffer=ArrayString::<[u8;64]>::new();

        for (fn_i,func) in ds.fn_info.functions.iter().enumerate() {
            for (i,pr) in func.params.iter().chain(&func.returns).enumerate() {
                buffer.clear();
                match pr.param_or_ret {
                    ParamOrReturn::Param=>write!(buffer,"fn_{}_p_{}",fn_i,i).drop_(),
                    ParamOrReturn::Return=>write!(buffer,"fn_{}_returns",fn_i).drop_(),
                };
                let field_name=&*buffer;
                let lifetime_refs=pr.lifetime_refs_tokenizer(ctokens);
                // println!("{}={:?}",field_name,pr.ty);
                
                to_stream!{ts; ct.tl_field,ct.colon2,ct.new };
                ct.paren.surround(ts,|ts|{
                    to_stream!(ts; field_name,ct.comma );

                    to_stream!(ts; ct.and_ );
                    ct.bracket.surround(ts,|ts| lifetime_refs.to_tokens(ts) );
                    
                    to_stream!(ts; ct.comma );

                    make_get_abi_info_tokenizer(pr.ty,&ct.stable_abi_bound,ct).to_tokens(ts);
                });

                ct.comma.to_tokens(ts);
            }
        }
    });

    let is_transparent=config.repr==Repr::Transparent ;
    let is_enum=ds.data_variant==DataVariant::Enum;

    let data_variant=ToTokenFnMut::new(|ts|{
        let ct=ctokens;

        match ( is_transparent, is_enum ) {
            (false,false)=>{
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
            (false,true)=>{
                let variants=variants_tokenizer(&ds,config,ct);

                to_stream!(ts;
                    ct.tl_data,ct.colon2,ct.enum_under
                );
                ct.paren.surround(ts,|ts|{
                    ct.and_.to_tokens(ts);
                    ct.bracket.surround(ts,|ts| variants.to_tokens(ts) );
                });
            }
            (true,false)=>{
                let repr_field_ty=get_abi_info_tokenizer(ds.variants[0].fields[0].ty,ctokens);
                    
                to_stream!(ts;
                    ct.tl_data,ct.colon2,ct.cap_repr_transparent
                );
                ct.paren.surround(ts,|ts| repr_field_ty.to_tokens(ts) );
            }
            (true,true)=>{
                panic!("repr(transparent) enums are not yet supported");
            }
        };
    });

    let repr_transparent_assertions=ToTokenFnMut::new(|ts|{
        let ct=ctokens;
        if !is_transparent { return }

        let struct_=&ds.variants[0];
        let repr_field=struct_.fields[0].ty;

        for field in &struct_.fields[1..] {
            to_stream!(ts;
                ct.assert_zero_sized,
                ct.lt,field.ty,ct.gt
            );
            ct.paren.surround(ts,|_|());
            ct.semicolon.to_tokens(ts);
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause=&where_clause.unwrap().predicates;

    let lifetimes=generics.lifetimes().map(|x|&x.lifetime);
    let type_params=generics.type_params()
        .map(|x|&x.ident)
        .filter(|x| !config.unconstrained_type_params.contains(x) );
    let const_params=generics.const_params().map(|x|&x.ident);


    let stringified_name=name.to_string();

    let module=Ident::new(&format!("_stable_abi_impls_for_{}",name),Span::call_site());

    let stable_abi_bounded =&config.stable_abi_bounded;
    let extra_bounds       =&config.extra_bounds;
    let shared_sabi_bounded=&config.shared_sabi_bounded;
    
    let inside_abi_stable_crate=if config.inside_abi_stable_crate { 
        quote!(use crate as abi_stable;)
    }else{ 
        quote!(use abi_stable;)
    };

    let import_group=if config.inside_abi_stable_crate { 
        quote!(crate)
    }else{ 
        quote!(abi_stable)
    };

    quote!(mod #module {
        use super::*;

        #inside_abi_stable_crate

        #[allow(unused_imports)]
        use #import_group::reexports::{
            self as _sabi_reexports,
            renamed::*,
        };

        unsafe impl #impl_generics #impld_trait for #name #ty_generics 
        where 
            #(#where_clause,)*
            #(#stable_abi_bounded:__StableAbi,)*
            #(#shared_sabi_bounded:__SharedStableAbi,)*
            #(#extra_bounds,)*
        {
            type IsNonZeroType=_sabi_reexports::False;

            const LAYOUT: &'static _sabi_reexports::TypeLayout = {
                &_sabi_reexports::TypeLayout::from_params::<Self>(
                    {
                        #repr_transparent_assertions;
                        __TypeLayoutParams {
                            name: #stringified_name,
                            package: env!("CARGO_PKG_NAME"),
                            package_version: abi_stable::package_version_string!(),
                            data: #data_variant,
                            generics: abi_stable::tl_genparams!(
                                #(#lifetimes),*;#(#type_params),*;#(#const_params),*
                            ),
                            phantom_fields: &[
                                #function_fields
                            ],
                        }
                    }
                )
            };
        }

    })
    .observe(|tokens|{
        if config.debug_print {
            panic!("\n\n\n{}\n\n\n",tokens );
        }
    })
}

/// Creates a value that outputs 
/// `<#ty as __SharedStableAbi>::ABI_INFO.get()`
/// to a token stream
fn get_abi_info_tokenizer<'a>(
    ty:&'a ::syn::Type,
    ct:&'a CommonTokens<'a>
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{
        to_stream!{ts; 
            ct.lt,ty,ct.as_,ct.shared_stable_abi,ct.gt,
            ct.colon2,ct.abi_info,ct.dot,ct.get
        }
        ct.paren.surround(ts,|_|());
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
    ctokens:&'a CommonTokens<'a>
)->impl ToTokens+'a{
    ToTokenFnMut::new(move|ts|{
        for field in &struct_.fields {
            let ct=ctokens;

            to_stream!{ts; ct.tl_field,ct.colon2,ct.new };
            ct.paren.surround(ts,|ts|{
                to_stream!(ts; field.ident().to_string() ,ct.comma );
                to_stream!(ts; ct.and_ );
                ct.bracket.surround(ts,|ts|{
                    for li in &field.referenced_lifetimes {
                        to_stream!{ts; li.tokenizer(ct),ct.comma }
                    }
                });
                
                to_stream!{ts; ct.comma };

                let impls_sabi=config.shared_sabi_field.map_or(false,|f|ptr::eq(f,field));
                let field_ptr:*const Field<'_>=field;
                let is_opaque_field=config.opaque_fields.contains(&field_ptr);

                let flavor=match (is_opaque_field,impls_sabi) {
                    (false,false)=>&ct.shared_stable_abi_bound,
                    (false,true )=>&ct.stable_abi_bound,
                    (true ,_    )=>&ct.unsafe_opaque_field_bound,
                };

                // println!(
                //     "field:`{}:{}` impls_sabi={} is_opaque_field={} ptr={:?}\n\
                //      opaque_fields:{:?}\n\
                //     ",
                //     field.ident(),
                //     (&field.ty).into_token_stream(),
                //     impls_sabi,
                //     is_opaque_field,
                //     field_ptr,
                //     config.opaque_fields,
                // );

                make_get_abi_info_tokenizer(field.ty,flavor,ct).to_tokens(ts);
            });
            to_stream!{ts; ct.comma }
        }
    })
}