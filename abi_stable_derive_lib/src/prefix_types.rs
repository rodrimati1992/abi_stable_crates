use std::collections::HashMap;


use syn::Ident;

use quote::ToTokens;

use proc_macro2::{TokenStream as TokenStream2,Span};

use core_extensions::{
    prelude::*,
    matches,
};

use arrayvec::ArrayString;


use crate::*;

use crate::{
    attribute_parsing::{parse_attrs_for_stable_abi, StabilityKind,StableAbiOptions,Repr},
    datastructure::{DataStructure,Struct,Field},
    to_token_fn::ToTokenFnMut,
};

pub(crate) struct PrefixKind<'a>{
    /// Which is the last field of the prefix,if None there is no prefix.
    pub(crate) last_prefix_field:Option<LastPrefixField<'a>>,
    pub(crate) first_suffix_field:usize,
    pub(crate) prefix_struct:&'a Ident,
    pub(crate) default_on_missing_fields:OnMissingField<'a>,
    pub(crate) on_missing_fields:HashMap<*const Field<'a>,OnMissingField<'a>>,
    
}



#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) struct LastPrefixField<'a>{
    pub(crate) field_index:usize,
    pub(crate) field:*const Field<'a>,
}


/// What happens in a Prefix-type field getter if the field does not exist.
#[derive(Copy,Clone, PartialEq, Eq, Hash)]
pub enum OnMissingField<'a>{
    /// Returns an `Option<FieldType>`,where it returns None if the field is absent.
    ReturnOption,
    /// Panics with a default message.
    Panic,
    /// Calls `function` with context information for panicking.
    PanicWith{
        function:&'a syn::Path,
    },
    /// Evaluates `function()`,and returns the return value of the function.
    With{
        function:&'a syn::Path,
    },
    /// Returns Default::default
    Default_,
}

impl<'a> Default for OnMissingField<'a>{
    fn default()->Self{
        OnMissingField::ReturnOption
    }
}


////////////////////////////////////////////////////////////////////////////////
/////                 Code generation
////////////////////////////////////////////////////////////////////////////////


/**
Returns a value which for a prefix-type .
*/
pub(crate) fn prefix_type_tokenizer<'a>(
    module:&'a Ident,
    ds:&'a DataStructure<'a>,
    config:&'a StableAbiOptions<'a>,
    ctokens:&'a CommonTokens<'a>,
)->impl ToTokens+'a {
    let ct=ctokens;
    ToTokenFnMut::new(move|ts|{
        let struct_=match ds.variants.get(0) {
            Some(x)=>x,
            None=>return,
        };

        let prefix=match &config.kind {
            StabilityKind::Prefix(prefix)=>prefix,
            _=>return,
        };

        // let repr_attrs=ToTokenFnMut::new(move|ts|{
        //     for list in &config.repr_attrs {
        //         ct.pound.to_tokens(ts);
        //         ct.bracket.surround(ts,|ts|{
        //             list.to_tokens(ts);
        //         });
        //     }
        // });

        let deriving_name=ds.name;
        let (impl_generics, ty_generics, where_clause) = ds.generics.split_for_impl();
        let where_clause=where_clause.unwrap();
        let where_clause_preds=&where_clause.predicates;

        // Generating the `*_Prefix` struct
        {
            let vis=ds.vis;
            let prefix_struct=prefix.prefix_struct;
            let generics=ds.generics;
            quote!(
                #[repr(transparent)]
                #vis struct #prefix_struct #generics #where_clause {
                    inner:#module::__WithMetadata_<(),Self>,
                    _priv: ::std::marker::PhantomData<fn()-> #deriving_name #ty_generics >,
                }
            ).to_tokens(ts);
        }
        
        let mut buffer=String::new();
        let prefix_struct=prefix.prefix_struct;

        let accessors=struct_.fields.iter().enumerate()
            .map(move|(field_i,field)|{
                use std::fmt::Write;
                buffer.clear();
                write!(buffer,"{}",field.ident()).drop_();
                let vis=field.vis;
                let getter_name=syn::parse_str::<Ident>(&*buffer).unwrap();
                let field_name=field.ident();
                let ty=field.ty;

                let on_missing_field=prefix.on_missing_fields
                    .get(&(field as *const _))
                    .cloned()
                    .unwrap_or(prefix.default_on_missing_fields);

                let is_optional=on_missing_field==OnMissingField::ReturnOption;

                if field_i < prefix.first_suffix_field {
                    quote!{
                        #vis fn #getter_name(&self)->#ty{
                            unsafe{ 
                                let ref_=&(*self.as_full_unchecked()).original.#field_name;
                                *ref_ 
                            }
                        }
                    }
                }else if is_optional{
                    quote!{
                        #vis fn #getter_name(&self)->Option< #ty >{
                            if #field_i < self.inner._prefix_type_field_count {
                                unsafe{ 
                                    let ref_=&(*self.as_full_unchecked()).original.#field_name;
                                    Some( *ref_ ) 
                                }
                            }else{
                                None
                            }
                        }
                    }
                }else{
                    let else_=match on_missing_field {
                        OnMissingField::ReturnOption=>unreachable!(),
                        OnMissingField::Panic=>quote!(
                            #module::_sabi_reexports::panic_on_missing_field_ty::<Self>(
                                #field_i,
                                self.inner._prefix_type_layout,
                            )
                        ),
                        OnMissingField::PanicWith{function}=>quote!(
                            function(
                                #field_i,
                                <Self as #module::_sabi_reexports::PrefixTypeTrait>::layout(),
                                self.inner._prefix_type_layout,
                            )
                        ),
                        OnMissingField::With{function}=>quote!{
                            #function()
                        },
                        OnMissingField::Default_=>quote!{
                            Default::default()
                        },
                    };
                    quote!{
                        #vis fn #getter_name(&self)->#ty{
                            if #field_i < self.inner._prefix_type_field_count {
                                unsafe{
                                    let ref_=&(*self.as_full_unchecked()).original.#field_name;
                                    *ref_
                                }
                            }else{
                                #else_
                            }
                        }
                    }
                }

            });

        let field_count=struct_.fields.len();
        let field_name_0=struct_.fields.iter().map(|x| x.ident() );
        let field_name_1=struct_.fields.iter().map(|x| x.ident() );

        quote!(

            unsafe impl #impl_generics 
                #module::_sabi_reexports::PrefixTypeTrait 
            for #deriving_name #ty_generics 
            where 
                #( #where_clause_preds ,)*
                #prefix_struct #ty_generics:#module::__SharedStableAbi,
            {
                const PREFIX_TYPE_COUNT:usize=#field_count;
                type Prefix=#prefix_struct #ty_generics;
            }


            impl #impl_generics #prefix_struct #ty_generics #where_clause {
                #(
                    #accessors
                )*
                // Returns a `*const _` instead of a `&_` because the compiler 
                // might assume in the future that references point to fully 
                // initialized values.
                unsafe fn as_full_unchecked(
                    &self
                )->*const #module::__WithMetadata_<#deriving_name #ty_generics,Self>{
                    self 
                    as *const Self
                    as *const #module::__WithMetadata_<#deriving_name #ty_generics,Self>
                }
            }

        ).to_tokens(ts);


    })
}


fn get_fields_tokenized<'a>(
    struct_:&'a Struct<'a>,
    taken_fields:usize,
    ctokens:&'a CommonTokens<'a>,
)->impl Iterator<Item= impl ToTokens+'a >+'a{
    struct_.fields.iter()
        .take(taken_fields)
        .map(move|field| field_tokenizer(field,ctokens) )
}


fn field_tokenizer<'a>(
    field:&'a Field<'a>,
    ctokens:&'a CommonTokens<'a>,
)->impl ToTokens+'a{
    let ct=ctokens;
    ToTokenFnMut::new(move|ts|{
        to_stream!(ts;field.vis,field.ident,ct.colon,field.ty,ct.comma);
    })
}