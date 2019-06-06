


use syn::{
    punctuated::Punctuated,
    Ident,
    WherePredicate,
    TypeParamBound,
    Visibility,
};

use quote::ToTokens;



use core_extensions::{
    prelude::*,
    matches,
};




use crate::*;

use crate::{
    datastructure::{DataStructure,Field,FieldMap,FieldIndex},
    to_token_fn::ToTokenFnMut,
    parse_utils::parse_str_as_ident,
};

use super::{
    attribute_parsing::{StabilityKind,StableAbiOptions},
    reflection::FieldAccessor,
};

pub(crate) struct PrefixKind<'a>{
    pub(crate) first_suffix_field:FirstSuffixField,
    pub(crate) prefix_struct:&'a Ident,
    pub(crate) prefix_bounds:Vec<WherePredicate>,
    pub(crate) fields:FieldMap<AccessorOrMaybe<'a>>,
    pub(crate) field_bounds:FieldMap<Vec<TypeParamBound>>,

}




#[derive(Copy,Default, Clone)]
pub(crate) struct PrefixKindField<'a>{
    pub(crate) accessible_if:Option<&'a syn::Expr>,
    pub(crate) on_missing:Option<OnMissingField<'a>>,
}


/// The different types of prefix-type accessors.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AccessorOrMaybe<'a>{
    /// Unconditionally returns the field.
    Accessor,
    /// Either optionally returns the field,or it does some action when it's missing.
    Maybe(MaybeAccessor<'a>)
}


#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct MaybeAccessor<'a>{
    accessible_if:Option<&'a syn::Expr>,
    on_missing:OnMissingField<'a>,
}



#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub(crate) struct FirstSuffixField{
    pub(crate) field_pos:usize,
}


/// What happens in a Prefix-type field getter if the field does not exist.
#[derive(Copy,Clone, PartialEq, Eq, Hash)]
pub(crate) enum OnMissingField<'a>{
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


impl<'a> AccessorOrMaybe<'a>{
    pub(crate) fn new(
        field_i:FieldIndex,
        first_suffix_field:FirstSuffixField,
        pkf:PrefixKindField<'a>,
        default_omf:OnMissingField<'a>,
    )->Self{
        if field_i.pos < first_suffix_field.field_pos && pkf.accessible_if.is_none() {
            AccessorOrMaybe::Accessor
        }else{
            AccessorOrMaybe::Maybe(MaybeAccessor{
                accessible_if:pkf.accessible_if,
                on_missing:pkf.on_missing.unwrap_or(default_omf)
            })
        }
    }

    pub(crate) fn is_conditional(&self)->bool{
        self.to_maybe_accessor().map_or(false,|x| x.accessible_if.is_some() )
    }

    pub(crate) fn to_maybe_accessor(&self)->Option<MaybeAccessor>{
        match *self {
            AccessorOrMaybe::Maybe(x)=>Some(x),
            _=>None,
        }
    }
}


impl<'a> PrefixKind<'a>{
    pub(crate) fn field_accessor(&self,field:&Field<'a>)->FieldAccessor<'a>{
        use self::{OnMissingField as OMF};

        match self.fields[field] {
            AccessorOrMaybe::Accessor=>
                FieldAccessor::Method{name:None},
            AccessorOrMaybe::Maybe(MaybeAccessor{on_missing,..})=>
                match on_missing {
                    OMF::ReturnOption=>
                        FieldAccessor::MethodOption,
                    OMF::Panic{..}|OMF::With{..}|OMF::Default_=>
                        FieldAccessor::Method{name:None},
                },
        }
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
    _ctokens:&'a CommonTokens<'a>,
)->impl ToTokens+'a {
    // let ct=ctokens;
    ToTokenFnMut::new(move|ts|{
        let struct_=match ds.variants.get(0) {
            Some(x)=>x,
            None=>return,
        };

        let prefix=match &config.kind {
            StabilityKind::Prefix(prefix)=>prefix,
            _=>return,
        };
        
        if struct_.fields.len() > 64 {
            panic!("\n\n`#[sabi(kind(Prefix(..)))]` structs cannot have more than 64 fields\n\n");
        }

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

        let empty_preds=Punctuated::new();
        let where_preds=where_clause.as_ref().map_or(&empty_preds,|x|&x.predicates);

        let stringified_deriving_name=deriving_name.to_string();

        let stringified_generics=(&ty_generics).into_token_stream().to_string();

        let is_ds_pub=matches!(Visibility::Public{..}=ds.vis);

        let prefix_struct_docs=if is_ds_pub {
            format!("\
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
            )
        }else{
            String::new()
        };
            

        // Generating the `<prefix_struct>` struct
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

        let accessor_docs=struct_.fields.iter()
            .map(move|field|{
                use std::fmt::Write;
                let mut acc_doc_buffer =String::new();
                if is_ds_pub{
                    let _=write!(
                        acc_doc_buffer,
                        "Accessor method for the `{deriving_name}::{field_name}` field.",
                        deriving_name=deriving_name,
                        field_name=field.ident(),
                    );

                    use self::{AccessorOrMaybe as AOM};

                    match prefix.fields[field] {
                        AOM::Accessor=>
                            acc_doc_buffer.push_str(
                                "This is for a field which always exists."
                            ),
                        AOM::Maybe(MaybeAccessor{on_missing:OnMissingField::ReturnOption,..})=>
                            acc_doc_buffer.push_str(
                                "Returns `Some(field_value)` if the field exists,\
                                 `None` if it does not.\
                                "
                            ),
                        AOM::Maybe(MaybeAccessor{on_missing:OnMissingField::Panic,..})=>
                            acc_doc_buffer.push_str(
                                "\n\n# Panic\n\nPanics if the field does not exist."
                            ),
                        AOM::Maybe(MaybeAccessor{on_missing:OnMissingField::With{function},..})=>
                            write!(
                                acc_doc_buffer,
                                "Returns `{function}()` if the field does not exist.",
                                function=(&function).into_token_stream().to_string()
                            ).drop_(),
                        AOM::Maybe(MaybeAccessor{on_missing:OnMissingField::Default_,..})=>
                            acc_doc_buffer.push_str(
                                "Returns `Default::default()` if the field does not exist."
                            ),
                    };
                }
                acc_doc_buffer
            });

        let field_count=struct_.fields.len();
        
        let ref field_mask_idents=(0..field_count)
            .map(|i|{
                let field_mask=format!("__AB_PTT_FIELD_{}_ACCESSIBILTIY_MASK",i);
                syn::parse_str::<Ident>(&field_mask).unwrap()
            })
            .collect::<Vec<Ident>>();

        

        let accessors=struct_.fields.iter().enumerate()
            .map(move|(field_i,field)|{
                use std::fmt::Write;
                accessor_buffer.clear();
                write!(accessor_buffer,"{}",field.ident()).drop_();
                let vis=field.vis;
                let getter_name=syn::parse_str::<Ident>(&*accessor_buffer).unwrap();
                let field_name=field.ident();
                let ty=field.ty;

                let field_bounds=&prefix.field_bounds[field];

                let field_where_clause=if field_bounds.is_empty() {
                    None 
                }else{ 
                    Some(quote!(where #ty:)) 
                };

                match prefix.fields[field] {
                    AccessorOrMaybe::Accessor=>{
                        quote!{
                            #vis fn #getter_name(&self)->#ty
                            #field_where_clause #( #field_bounds+ )*
                            {
                                unsafe{ 
                                    let ref_=&(*self.as_full_unchecked()).#field_name;
                                    *ref_ 
                                }
                            }
                        }
                    },
                    AccessorOrMaybe::Maybe(maybe_accessor)=>{
                        let on_missing_field=maybe_accessor.on_missing;
                        let is_optional=on_missing_field==OnMissingField::ReturnOption;

                        let return_ty=if is_optional {
                            quote!( Option< #ty > )
                        }else{
                            quote!( #ty)
                        };

                        let else_=match on_missing_field {
                            OnMissingField::ReturnOption=>quote!{
                                return None 
                            },
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

                        let with_val=if is_optional {
                            quote!( Some(val) )
                        }else{
                            quote!( val )
                        };

                        let field_mask_ident=&field_mask_idents[field_i];

                        quote!{
                            #vis fn #getter_name(&self)->#return_ty
                            #field_where_clause #( #field_bounds+ )*
                            {
                                let acc_bits=self.inner._prefix_type_field_acc.bits();
                                let val=if (Self::#field_mask_ident & acc_bits)==0 {
                                    #else_
                                }else{
                                    unsafe{
                                        (*self.as_full_unchecked()).#field_name
                                    }
                                };
                                #with_val
                            }
                        }
                    }
                }
            });


        let prefix_bounds=&prefix.prefix_bounds;

        let conditional_fields=
            struct_.fields.iter().enumerate()
                .filter_map(|(i,field)|{
                    let cond=prefix.fields[field].to_maybe_accessor()?.accessible_if?;
                    Some((i,cond))
                })
                .collect::<Vec<(usize,&syn::Expr)>>();

        let disabled_field_indices=conditional_fields.iter().map(|&(field_i,_)| field_i );

        let enable_field_if=conditional_fields.iter().map(|&(_,cond)| cond );

        let field_name_list=struct_.fields.iter()
            .map(|x| x.ident().to_string() )
            .collect::<Vec<String>>();

        let str_field_names=field_name_list.join(";");
        
        let is_prefix_field_conditional=struct_.fields.iter()
            .take(prefix.first_suffix_field.field_pos)
            .map(|f| prefix.fields[f].is_conditional() );

        let field_index_for=field_name_list.iter()
            .map(|field_name| parse_str_as_ident(&format!("field_index_for_{}",field_name)) );

        let field_i=0usize..;

        let field_i_b=0u8..;

        let pt_layout_ident=parse_str_as_ident(&format!("__sabi_PT_LAYOUT{}",deriving_name));

        quote!(

            const #pt_layout_ident:&'static #module::__PTStructLayout ={
                use #module::_sabi_reexports::renamed::{
                    __PTStructLayout,__PTStructLayoutParams,
                };

                &__PTStructLayout::new(__PTStructLayoutParams{
                    name:#stringified_deriving_name,
                    generics:#stringified_generics,
                    package: env!("CARGO_PKG_NAME"),
                    package_version: #module::abi_stable::package_version_strings!(),
                    file:file!(),
                    line:line!(),
                    field_names:#str_field_names,
                })
            };

            unsafe impl #impl_generics
                #module::_sabi_reexports::PrefixTypeTrait 
            for #deriving_name #ty_generics 
            where
                #(#where_preds,)*
                #(#prefix_bounds,)*
            {
                const PT_FIELD_ACCESSIBILITY:#module::_sabi_reexports::FieldAccessibility={
                    use self::#module::_sabi_reexports::{
                        FieldAccessibility as __FieldAccessibility,
                        IsAccessible as __IsAccessible,
                    };
                    __FieldAccessibility::with_field_count(#field_count)
                    #(
                        .set_accessibility(
                            #disabled_field_indices,
                            __IsAccessible::new(#enable_field_if)
                        )
                    )*
                };
                
                const PT_COND_PREFIX_FIELDS:&'static [#module::_sabi_reexports::IsConditional]={
                    use #module::_sabi_reexports::IsConditional as __IsConditional;

                    &[
                        #( __IsConditional::new( #is_prefix_field_conditional ) ,)*
                    ]
                };

                const PT_LAYOUT:&'static #module::__PTStructLayout =#pt_layout_ident;

                type Prefix=#prefix_struct #ty_generics;
            }

            impl #impl_generics #prefix_struct #ty_generics 
            where 
                #(#where_preds,)*
                #deriving_name #ty_generics: #module::_sabi_reexports::PrefixTypeTrait,
            {
                const __AB_PTT_FIELD_ACCESSIBILTIY_MASK:u64=
                    <#deriving_name #ty_generics as 
                        #module::_sabi_reexports::PrefixTypeTrait 
                    >::PT_FIELD_ACCESSIBILITY.bits();

                /// Accessor to get the layout of the type.
                #[inline(always)]
                pub fn _prefix_type_layout(&self)-> &'static #module::__PTStructLayout {
                    self.inner._prefix_type_layout
                }

                #(
                    const #field_index_for:u8=
                        #field_i_b;

                    // #(
                        const #field_mask_idents:u64=
                            (1<<#field_i) & Self::__AB_PTT_FIELD_ACCESSIBILTIY_MASK;
                    // )*

                    #[doc=#accessor_docs]
                    #accessors
                )*



                // Returns a `*const _` instead of a `&_` because the compiler 
                // might assume in the future that references point to fully 
                // initialized values.
                unsafe fn as_full_unchecked(
                    &self
                )->*const #deriving_name #ty_generics {
                    let ptr=self 
                        as *const Self
                        as *const #module::__WithMetadata_<#deriving_name #ty_generics,Self>;
                    #module::__WithMetadata_::into_full(ptr)
                }
            }

        ).to_tokens(ts);


    })
}
