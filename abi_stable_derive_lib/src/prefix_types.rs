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
        let (ref impl_generics,ref ty_generics,ref where_clause) = ds.generics.split_for_impl();

        let stringified_deriving_name=deriving_name.to_string();

        let stringified_generics=(&ty_generics).into_token_stream().to_string();

        let prefix_struct_docs=format!("\
This is the prefix of 
[{deriving_name}{generics}](./struct.{deriving_name}.html),
only usable as `&{name}{generics}`.

**This is automatically generated documentation,by the StableAbi derive macro**.

### Creating a runtime value

Using `abi_stable::prefix_type::PrefixTypeTrait`.<br>
Call the `PrefixTypeTrait::leak_into_prefix` method on the 
`{deriving_name}{generics}` type,
which converts it to `&{name}{generics}`,
leaking it in the process.

### Creating a compiletime-constant

Using `abi_stable::prefix_type::{{PrefixTypeTrait,WithMetadata}}`.<br>
Use WithMetadata if you need a compiletime constant.<br>
First create a `&WithMetadata<{deriving_name}{generics}>` constant with (approximately):
```ignore
struct Dummy<'some>(PhantomData<&'some ()>);

impl<'some> Dummy<'some>{{
    const CONSTANT:&'some WithMetadata<{deriving_name}{generics}>=
        &WithMetadata::new(
            PrefixTypeTrait::METADATA, 
            value /* value : {deriving_name}{generics} */
        );
}}

```
then use the `as_prefix` method at runtime to cast it to `&{name}{generics}`.
            ",
            name=prefix.prefix_struct,
            deriving_name=stringified_deriving_name,
            generics=stringified_generics,
        );

        // Generating the `*_Prefix` struct
        {
            let vis=ds.vis;
            let prefix_struct=prefix.prefix_struct;
            let generics=ds.generics;
            quote!(
                #[doc=#prefix_struct_docs]
                #[repr(transparent)]
                #vis struct #prefix_struct #generics #where_clause {
                    inner:#module::__WithMetadata_<(),Self>,
                    _priv: ::std::marker::PhantomData<#deriving_name #ty_generics >,
                }
            ).to_tokens(ts);
        }
        
        let mut accessor_buffer=String::new();
        let prefix_struct=prefix.prefix_struct;

        let get_on_missing_field=|field:*const Field|->OnMissingField{
            prefix.on_missing_fields
                .get(&field)
                .cloned()
                .unwrap_or(prefix.default_on_missing_fields)
        };

        let accessor_docs=struct_.fields.iter().enumerate()
            .map(move|(field_i,field)|{
                use std::fmt::Write;
                let mut acc_doc_buffer =String::new();
                let _=write!(
                    acc_doc_buffer,
                    "Accessor method for the `{deriving_name}::{field_name}` field.",
                    deriving_name=deriving_name,
                    field_name=field.ident(),
                );
                let in_prefix=field_i < prefix.first_suffix_field;
                let on_missing_fields=get_on_missing_field(field);
                match (in_prefix,on_missing_fields) {
                    (true,_)=>
                        acc_doc_buffer.push_str(
                            "This is for a field which always exists."
                        ),
                    (false,OnMissingField::ReturnOption)=>
                        acc_doc_buffer.push_str(
                            "Returns `Some(field_value)` if the field exists,\
                             `None` if it does not.\
                            "
                        ),
                    (false,OnMissingField::Panic)=>
                        acc_doc_buffer.push_str(
                            "\n\n# Panic\n\nPanics if the field does not exist."
                        ),
                    (false,OnMissingField::With{function})=>
                        write!(
                            acc_doc_buffer,
                            "Returns `{function}()` if the field does not exist.",
                            function=(&function).into_token_stream().to_string()
                        ).drop_(),
                    (false,OnMissingField::Default_)=>
                        acc_doc_buffer.push_str(
                            "Returns `Default::default()` if the field does not exist."
                        ),
                };
                acc_doc_buffer
            });

        let accessors=struct_.fields.iter().enumerate()
            .map(move|(field_i,field)|{
                use std::fmt::Write;
                accessor_buffer.clear();
                write!(accessor_buffer,"{}",field.ident()).drop_();
                let vis=field.vis;
                let getter_name=syn::parse_str::<Ident>(&*accessor_buffer).unwrap();
                let field_name=field.ident();
                let ty=field.ty;

                let on_missing_field=get_on_missing_field(field);

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
                            #module::_sabi_reexports::panic_on_missing_field_ty::<
                                #deriving_name #ty_generics
                            >(
                                #field_i,
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
        
        let str_field_names=struct_.fields.iter().map(|x| x.ident().to_string() );
        
        let field_types=struct_.fields.iter().map(|x| x.ty );
        let str_field_types=struct_.fields.iter()
            .map(|x| x.ty.into_token_stream().to_string() );
        
        quote!(

            unsafe impl #impl_generics 
                #module::_sabi_reexports::PrefixTypeTrait 
            for #deriving_name #ty_generics 
            #where_clause
            {
                const PT_FIELD_COUNT:usize=#field_count;

                const PT_LAYOUT:&'static #module::__PTStructLayout ={
                    use #module::_sabi_reexports::renamed::{
                        __PTStructLayout,__PTStructLayoutParams,__PTField
                    };

                    &__PTStructLayout::new::<Self>(__PTStructLayoutParams{
                        name:#stringified_deriving_name,
                        generics:#stringified_generics,
                        package: env!("CARGO_PKG_NAME"),
                        package_version: #module::abi_stable::package_version_strings!(),
                        file:file!(),
                        line:line!(),
                        fields:&[
                            #(
                                __PTField::new::<#field_types>(
                                    #str_field_names, 
                                    #str_field_types,
                                ),
                            )*
                        ]
                    })
                };

                type Prefix=#prefix_struct #ty_generics;
            }

            impl #impl_generics #prefix_struct #ty_generics #where_clause {
                #(
                    #[doc=#accessor_docs]
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