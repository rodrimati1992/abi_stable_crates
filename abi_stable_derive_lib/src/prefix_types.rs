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
    /// The identifier of the struct with all the prefix fields as well as the Prefix-type metadata.
    pub(crate) prefix_struct:&'a Ident,
    /// The identifier of the struct with all the fields as well as the Prefix-type metadata.
    pub(crate) with_metadata:&'a Ident,
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
Returns a value which if the type is a prefix-type ,outputs 
`struct *_WithMetadata`,`struct *_Prefix`,and impl blocks of those types,
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

        // The fields that store metadata of the prefix-type
        let metadata_fields=quote!{
            _prefix_type_field_count:usize,
            _prefix_type_layout:#module::TypeLayout,
        };

        // Generating the `*_Prefix` struct
        to_stream!(ts;ds.vis,ct.struct_,prefix.prefix_struct);
        ds.generics.to_tokens(ts);
        ds.generics.where_clause.to_tokens(ts);
        ct.brace.surround(ts,|ts|{
            metadata_fields.to_tokens(ts);

            for field in get_fields_tokenized(struct_,prefix.first_suffix_field,ct) {
                field.to_tokens(ts);
            }
        });

        // Generating the `*_WithMetadata` struct.
        // Which contains all the fields of the deriving struct.
        to_stream!(ts;ds.vis,ct.struct_,prefix.with_metadata);
        ds.generics.to_tokens(ts);
        ds.generics.where_clause.to_tokens(ts);
        ct.brace.surround(ts,|ts|{
            metadata_fields.to_tokens(ts);

            for field in get_fields_tokenized(struct_,!0,ct) {
                field.to_tokens(ts);
            }
        });

        let mut buffer=String::new();
        let deriving_name=ds.name;
        let prefix_struct=prefix.prefix_struct;
        let with_metadata=prefix.with_metadata;

        let accessors=struct_.fields.iter().enumerate()
            .map(move|(field_i,field)|{
                use std::fmt::Write;
                write!(buffer,"get_{}",field.ident()).drop_();
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
                            self.#field_name
                        }
                    }
                }else if is_optional{
                    quote!{
                        #vis fn #getter_name(&self)->Option< #ty >{
                            if #field_i < self._prefix_type_field_count {
                                unsafe{ Some(self.as_full_unchecked().#field_name) }
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
                                _expected_field_index,
                                self._prefix_type_layout,
                            )
                        ),
                        OnMissingField::PanicWith{function}=>quote!(
                            function(
                                _expected_field_index,
                                <Self as #module::_sabi_reexports::PrefixTypeTrait>::layout(),
                                self._prefix_type_layout,
                            )
                        ),
                        OnMissingField::With{function}=>quote!{
                            function()
                        },
                        OnMissingField::Default_=>quote!{
                            Default::default()
                        },
                    };
                    quote!{
                        #vis fn #getter_name(&self)->#ty{
                            if #field_i < self._prefix_type_field_count {
                                unsafe{
                                    self.as_full_unchecked().#field_name
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

        let (impl_generics, ty_generics, where_clause) = ds.generics.split_for_impl();

        quote!(
            impl #impl_generics #prefix_struct #ty_generics #where_clause {
                #(
                    #accessors
                )*

                unsafe fn as_full_unchecked(&self)->& #with_metadata #ty_generics {
                    &*(self as *const _ as *const #with_metadata #ty_generics)
                }
            }

            impl #impl_generics #with_metadata #ty_generics #where_clause {
                pub fn new(from:#deriving_name #ty_generics )->Self{
                    Self{
                        _prefix_type_field_count:#field_count,
                        _prefix_type_layout:
                            <
                                #prefix_struct #ty_generics as 
                                #module::_sabi_reexports::SharedStableAbi
                            >::S_ABI_INFO.get().layout,
                        #(
                            #field_name_0:from.#field_name_1,
                        )*
                    }
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
        to_stream!(ts;field.vis,field.ident,ct.colon2,field.ty,ct.comma);
    })
}