/*!

Code generation for prefix-types.

*/


use abi_stable_shared::const_utils::low_bit_mask_u64;

use core_extensions::{
    prelude::*,
    matches,
};

use syn::{
    punctuated::Punctuated,
    Ident,
    WherePredicate,
    TypeParamBound,
    Visibility,
};

use quote::{ToTokens,quote_spanned};



use crate::*;

use crate::{
    datastructure::{DataStructure,Field,FieldMap,FieldIndex},
    literals_constructors::rstr_tokenizer,
    to_token_fn::ToTokenFnMut,
    parse_utils::parse_str_as_ident,
};

use super::{
    CommonTokens,
    attribute_parsing::{StabilityKind,StableAbiOptions},
    reflection::FieldAccessor,
};

/// Configuration for code generation related to prefix-types.
pub(crate) struct PrefixKind<'a>{
    pub(crate) first_suffix_field:FirstSuffixField,
    pub(crate) prefix_struct:&'a Ident,
    pub(crate) prefix_bounds:Vec<WherePredicate>,
    pub(crate) fields:FieldMap<AccessorOrMaybe<'a>>,
    pub(crate) accessor_bounds:FieldMap<Vec<TypeParamBound>>,
    pub(crate) field_conditionality_ident:&'a Ident,
}



/// Used while parsing the prefix-type-related attributes on fields.
#[derive(Copy,Default, Clone)]
pub(crate) struct PrefixKindField<'a>{
    pub(crate) accessible_if:Option<&'a syn::Expr>,
    pub(crate) on_missing:Option<OnMissingField<'a>>,
}


/// The different types of prefix-type accessors.
#[derive(Debug,Copy, Clone, PartialEq, Eq)]
pub enum AccessorOrMaybe<'a>{
    /// Unconditionally returns the field.
    Accessor,
    /// Either optionally returns the field,or it does some action when it's missing.
    Maybe(MaybeAccessor<'a>)
}

/// Describes a field accessor which is either optional or 
/// does some action when the field is missing.
#[derive(Debug,Copy, Clone, Default, PartialEq, Eq)]
pub struct MaybeAccessor<'a>{
    /// If Some,it uses a bool constant to determine whether a field is accessible.
    accessible_if:Option<&'a syn::Expr>,
    /// What the accessor method does when the field is missing.
    on_missing:OnMissingField<'a>,
}



#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub(crate) struct FirstSuffixField{
    pub(crate) field_pos:usize,
}


/// What happens in a Prefix-type field getter if the field does not exist.
#[derive(Debug,Copy,Clone, PartialEq, Eq, Hash)]
pub(crate) enum OnMissingField<'a>{
    /// Returns an `Option<FieldType>`,where it returns None if the field is absent.
    ReturnOption,
    /// Panics with a default message.
    Panic,
    /// Evaluates `function()`,and returns the return value of the function.
    With{
        function:&'a syn::Path,
    },
    /// Returns `some_expression`.
    Value{
        value:&'a syn::Expr,
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
        if field_i.pos < first_suffix_field.field_pos && 
            pkf.accessible_if.is_none() &&
            pkf.on_missing!=Some(OnMissingField::ReturnOption)
        {
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

    /// Converts this to a MaybeAccessor,returning None if it is not the `Maybe` variant.
    pub(crate) fn to_maybe_accessor(&self)->Option<MaybeAccessor>{
        match *self {
            AccessorOrMaybe::Maybe(x)=>Some(x),
            _=>None,
        }
    }

    pub(crate) fn is_maybe_accessor(&self)->bool{
        matches!(AccessorOrMaybe::Maybe{..}= self )
    }
}


impl<'a> PrefixKind<'a>{
    /// Gets the accessibility for a field,used for (very basic) runtime reflection.
    pub(crate) fn field_accessor(&self,field:&Field<'a>)->FieldAccessor<'a>{
        use self::{OnMissingField as OMF};

        match self.fields[field] {
            AccessorOrMaybe::Accessor=>
                FieldAccessor::Method{name:None},
            AccessorOrMaybe::Maybe(MaybeAccessor{on_missing,..})=>
                match on_missing {
                    OMF::ReturnOption=>
                        FieldAccessor::MethodOption,
                    OMF::Panic{..}|OMF::With{..}|OMF::Value{..}|OMF::Default_=>
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
    mono_type_layout:&'a Ident,
    ds:&'a DataStructure<'a>,
    config:&'a StableAbiOptions<'a>,
    _ctokens:&'a CommonTokens<'a>,
)-> Result<impl ToTokens+'a,syn::Error> {
    if matches!(StabilityKind::Prefix{..}=&config.kind) && 
        ds.variants.get(0).map_or(false,|struct_| struct_.fields.len() > 64 )
    {
        return_spanned_err!(
            ds.name,
            "`#[sabi(kind(Prefix(..)))]` structs cannot have more than 64 fields."
        );
    }

    Ok(ToTokenFnMut::new(move|ts|{
        let struct_=match ds.variants.get(0) {
            Some(x)=>x,
            None=>return,
        };

        let prefix=match &config.kind {
            StabilityKind::Prefix(prefix)=>prefix,
            _=>return,
        };

        let deriving_name=ds.name;
        let (ref impl_generics,ref ty_generics,ref where_clause) = ds.generics.split_for_impl();

        let empty_preds=Punctuated::new();
        let where_preds=where_clause.as_ref().map_or(&empty_preds,|x|&x.predicates).into_iter();
        let where_preds_b=where_preds.clone();
        let where_preds_c=where_preds.clone();
        let prefix_bounds=&prefix.prefix_bounds;

        let stringified_deriving_name=deriving_name.to_string();
        
        let stringified_generics=(&ty_generics).into_token_stream().to_string();
        let stringified_generics_tokenizer=rstr_tokenizer(&stringified_generics);

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


        let mut uncond_acc_docs=Vec::<String>::new();
        let mut cond_acc_docs=Vec::<String>::new();
        let mut field_mask_idents=Vec::new();
        let mut field_index_for=Vec::new();

        // Creates the docs for the accessor functions.
        // Creates the identifiers for constants associated with each field.
        for (index,field) in struct_.fields.iter().enumerate() {
            use std::fmt::Write;
            use self::{AccessorOrMaybe as AOM};

            let mut acc_doc_buffer =String::new();
            let acc_on_missing=prefix.fields[field];
            if is_ds_pub{
                let _=write!(
                    acc_doc_buffer,
                    "Accessor method for the `{deriving_name}::{field_name}` field.",
                    deriving_name=deriving_name,
                    field_name=field.ident(),
                );


                match acc_on_missing {
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
                    AOM::Maybe(MaybeAccessor{on_missing:OnMissingField::Value{..},..})=>
                        acc_doc_buffer.push_str("\
                            Returns a default value (not Default::default()) \
                            if the field does not exist.\
                        "),
                    AOM::Maybe(MaybeAccessor{on_missing:OnMissingField::Default_,..})=>
                        acc_doc_buffer.push_str(
                            "Returns `Default::default()` if the field does not exist."
                        ),
                };
            }
            
            let field_name=field.ident();
            {
                let mut new_ident=parse_str_as_ident(&format!("field_index_for_{}",field_name));
                new_ident.set_span(field_name.span());
                field_index_for.push(new_ident);
            }
            
            let field_mask=format!("__AB_PTT_FIELD_{}_ACCESSIBILTIY_MASK",index);
            let mut field_mask=syn::parse_str::<Ident>(&field_mask).expect("BUG");
            field_mask.set_span(field.ident().span());
            field_mask_idents.push(field_mask);

            match acc_on_missing {
                AOM::Accessor =>{
                    uncond_acc_docs.push(acc_doc_buffer);
                }
                AOM::Maybe{..}=>{
                    cond_acc_docs.push(acc_doc_buffer);
                }
            }
        }

        

        let field_count=struct_.fields.len();

        let mut unconditional_accessors=Vec::new();
        let mut conditional_accessors=Vec::new();
        
        // Creates TokenStreams for each accessor function.
        for (field_i,field)in struct_.fields.iter().enumerate() {
            use std::fmt::Write;
            accessor_buffer.clear();
            write!(accessor_buffer,"{}",field.ident()).drop_();
            let vis=field.vis;
            let mut getter_name=syn::parse_str::<Ident>(&*accessor_buffer).expect("BUG");
            getter_name.set_span(field.ident().span());
            let field_name=field.ident();
            let field_span=field_name.span();
            let ty=field.ty;

            let accessor_bounds=&prefix.accessor_bounds[field];

            let field_where_clause=if accessor_bounds.is_empty() {
                None 
            }else{ 
                Some(quote!(where #ty:)) 
            };

            match prefix.fields[field] {
                AccessorOrMaybe::Accessor=>{
                    unconditional_accessors.push(quote_spanned!{field_span=>
                        #vis fn #getter_name(&self)->#ty
                        #field_where_clause #( #accessor_bounds+ )*
                        {
                            unsafe{ 
                                let ref_=&(*self.as_full_unchecked()).#field_name;
                                *ref_ 
                            }
                        }
                    })
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
                        OnMissingField::ReturnOption=>quote_spanned!{field_span=>
                            return None 
                        },
                        OnMissingField::Panic=>quote_spanned!(field_span=>
                            #module::_sabi_reexports::panic_on_missing_field_ty::<
                                #deriving_name #ty_generics
                            >(
                                #field_i,
                                self.inner._prefix_type_layout,
                            )
                        ),
                        OnMissingField::With{function}=>quote_spanned!{field_span=>
                            #function()
                        },
                        OnMissingField::Value{value}=>quote_spanned!{field_span=>
                            (#value)
                        },
                        OnMissingField::Default_=>quote_spanned!{field_span=>
                            Default::default()
                        },
                    };

                    let with_val=if is_optional {
                        quote_spanned!(field_span=> Some(val) )
                    }else{
                        quote_spanned!(field_span=> val )
                    };

                    let field_mask_ident=&field_mask_idents[field_i];

                    conditional_accessors.push(quote_spanned!{field_span=>
                        #vis fn #getter_name(&self)->#return_ty
                        #field_where_clause #( #accessor_bounds+ )*
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
                    });
                }
            }
        }

        let mut cond_field_indices=Vec::<usize>::new();
        let mut enable_field_if=Vec::<&syn::Expr>::new();
        let mut unconditional_bit_mask=0u64;
        let mut conditional_bit_mask=0u64;

        for (index,field) in prefix.fields.iter() {
            let field_i=index.pos;
            match (|| field.to_maybe_accessor()?.accessible_if )() {
                Some(cond)=>{
                    cond_field_indices.push(field_i);
                    enable_field_if.push(cond);
                    conditional_bit_mask|=(1 as u64)<<field_i;
                }
                None=>{
                    unconditional_bit_mask|=1u64<<field_i;
                }
            }
        }

        let prefix_field_conditionality_mask=
            conditional_bit_mask &low_bit_mask_u64(prefix.first_suffix_field.field_pos as u32);

        let cond_field_indices=cond_field_indices.iter();
        let enable_field_if=enable_field_if.iter();

        let mut str_field_names=String::new();
        for field in &struct_.fields {
            use std::fmt::Write;
            writeln!(str_field_names,"{};",field.ident());
        }
        str_field_names.pop(); //Removing the last ';'
        let str_field_names_tokenizer=rstr_tokenizer(&str_field_names);
        
        let conditional_enumerate=0usize..;

        let field_i_a=0u8..;
        let field_i_b=0u8..;

        let mut pt_layout_ident=parse_str_as_ident(&format!("__sabi_PT_LAYOUT{}",deriving_name));
        pt_layout_ident.set_span(deriving_name.span());

        let field_conditionality_ident=prefix.field_conditionality_ident;

        quote!(

            #[allow(non_upper_case_globals)]
            const #pt_layout_ident:&'static #module::__PTStructLayout ={
                &#module::_sabi_reexports::PTStructLayout::new(
                    #stringified_generics_tokenizer,
                    #module::#mono_type_layout,
                    #str_field_names_tokenizer,
                    #prefix_field_conditionality_mask,
                )
            };

            // This is so that the field conditionality is only computed once.
            const #field_conditionality_ident:#module::_sabi_reexports::FieldConditionality={
                #module::_sabi_reexports::FieldConditionality::from_u64(
                    #prefix_field_conditionality_mask
                )
            };

            unsafe impl #impl_generics
                #module::_sabi_reexports::PrefixTypeTrait 
            for #deriving_name #ty_generics 
            where
                #(#where_preds,)*
                #(#prefix_bounds,)*
            {
                // Describes the accessibility of all the fields,
                // used to initialize the `WithMetadata<Self>::_prefix_type_field_acc` field.
                const PT_FIELD_ACCESSIBILITY:#module::_sabi_reexports::FieldAccessibility={
                    #module::_sabi_reexports::FieldAccessibility::from_u64(
                        #unconditional_bit_mask
                        #(
                            |(((#enable_field_if)as u64) << #cond_field_indices)
                        )*
                    )
                };
                // A description of the struct used for error messages.
                const PT_LAYOUT:&'static #module::__PTStructLayout =#pt_layout_ident;

                // This is a struct whose only non-zero-sized field is `WithMetadata_<(),Self>`.
                type Prefix=#prefix_struct #ty_generics;
            }

            #[allow(non_upper_case_globals)]
            impl #impl_generics #prefix_struct #ty_generics 
            where 
                #(#where_preds_b,)*
            {
                #(
                    #[doc=#uncond_acc_docs]
                    #unconditional_accessors
                )*

                #(
                    // This is the field index,starting with 0,from the top field.
                    const #field_index_for:u8=
                        #field_i_a;
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

        quote!(            
            #[allow(non_upper_case_globals)]
            impl #impl_generics #prefix_struct #ty_generics 
            where 
                #(#where_preds_c,)*
                #(#prefix_bounds,)*
                #deriving_name #ty_generics: #module::_sabi_reexports::PrefixTypeTrait,
            {
                // The accessibility of all fields,
                // used bellow to initialize the mask for each individual field.
                //
                // If the nth bit is:
                //    0:the field is inaccessible.
                //    1:the field is accessible.
                const __AB_PTT_FIELD_ACCESSIBILTIY_MASK:u64=
                    <#deriving_name #ty_generics as 
                        #module::_sabi_reexports::PrefixTypeTrait 
                    >::PT_FIELD_ACCESSIBILITY.bits();

                #(
                    // The mask to get whether the field is accessible in the accessor method,
                    // by doing `self._prefix_type_field_acc.bits() & Self::#field_mask_ident`.
                    const #field_mask_idents:u64=
                        (1<<#field_i_b) & Self::__AB_PTT_FIELD_ACCESSIBILTIY_MASK;
                )*

                /// Accessor to get the layout of the type,used for error messages.
                #[inline(always)]
                pub fn _prefix_type_layout(&self)-> &'static #module::__PTStructLayout {
                    self.inner._prefix_type_layout
                }

                #(
                    #[doc=#cond_acc_docs]
                    #conditional_accessors
                )*
            }

        ).to_tokens(ts);


    }))
}
