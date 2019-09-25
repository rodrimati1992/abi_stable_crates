use syn::{
    Attribute, Ident, Meta, MetaList, MetaNameValue, NestedMeta, 
    Lit,WherePredicate,Type,
    punctuated::Punctuated,
    token::Comma,
    TypeParamBound,
};

use std::{
    collections::HashSet,
    mem,
};

use core_extensions::{matches,IteratorExt};

use proc_macro2::Span;

use quote::ToTokens;

use crate::{
    attribute_parsing::{with_nested_meta},
    impl_interfacetype::{ImplInterfaceType,parse_impl_interfacetype},
    datastructure::{DataStructure, DataVariant, Field, FieldMap, TypeParamMap},
    parse_utils::{
        parse_str_as_ident,
        parse_str_as_type,
        parse_lit_as_expr,
        parse_lit_as_type,
        parse_lit_as_type_bounds,
        ParsePunctuated,
    },
    utils::{LinearResult,SynPathExt,SynResultExt},
};

use super::{
    nonexhaustive::{
        UncheckedNonExhaustive,NonExhaustive,EnumInterface,IntOrType,
        UncheckedVariantConstructor,
    },
    reflection::{ModReflMode,FieldAccessor},
    prefix_types::{PrefixKind,FirstSuffixField,OnMissingField,AccessorOrMaybe,PrefixKindField},
    repr_attrs::{UncheckedReprAttr,UncheckedReprKind,DiscriminantRepr,ReprAttr,REPR_ERROR_MSG},
};

use crate::*;

pub(crate) struct StableAbiOptions<'a> {
    pub(crate) debug_print:bool,
    pub(crate) kind: StabilityKind<'a>,
    pub(crate) repr: ReprAttr,

    pub(crate) type_param_bounds:TypeParamMap<'a,ASTypeParamBound>,

    pub(crate) extra_bounds:Vec<WherePredicate>,

    pub(crate) tags:Option<syn::Expr>,
    pub(crate) extra_checks:Option<syn::Expr>,

    pub(crate) layout_ctor:FieldMap<LayoutConstructor>,

    pub(crate) override_field_accessor:FieldMap<Option<FieldAccessor<'a>>>,
    
    pub(crate) renamed_fields:FieldMap<Option<&'a Ident>>,
    pub(crate) changed_types:FieldMap<Option<&'a Type>>,

    pub(crate) doc_hidden_attr:Option<&'a TokenStream2>,

    pub(crate) mod_refl_mode:ModReflMode<usize>,

    pub(crate) impl_interfacetype:Option<ImplInterfaceType>,

    pub(crate) phantom_fields:Vec<(&'a Ident,&'a Type)>,
    pub(crate) phantom_type_params:Vec<&'a Type>,
    pub(crate) phantom_const_params:Vec<&'a syn::Expr>,

    pub(crate) allow_type_macros:bool,
    pub(crate) with_field_indices:bool,
    
}


//////////////////////

#[derive(Debug,Clone,Copy,Eq,PartialEq,Hash)]
pub(crate) enum ASTypeParamBound{
    NoBound,
    GetStaticEquivalent,
    StableAbi,
    SharedStableAbi,
}

impl Default for ASTypeParamBound{
    fn default()->Self{
        ASTypeParamBound::StableAbi
    }
}


//////////////////////


#[derive(Debug,Clone,Copy,Eq,PartialEq,Hash)]
pub(crate) enum LayoutConstructor{
    Regular,
    SharedStableAbi,
    Opaque,
    SabiOpaque,
}

impl LayoutConstructor{
    pub(crate) fn is_opaque(self)->bool{
        matches!(LayoutConstructor::Opaque{..}= self )
    }
}

impl From<ASTypeParamBound> for LayoutConstructor{
    fn from(bound:ASTypeParamBound)->Self{
        match bound {
            ASTypeParamBound::NoBound=>
                LayoutConstructor::Opaque,
            ASTypeParamBound::GetStaticEquivalent=>
                LayoutConstructor::Opaque,
            ASTypeParamBound::StableAbi=>
                LayoutConstructor::Regular,
            ASTypeParamBound::SharedStableAbi=>
                LayoutConstructor::SharedStableAbi,
        }
    }
}

impl Default for LayoutConstructor{
    fn default()->Self{
        LayoutConstructor::Regular
    }
}


//////////////////////


pub(crate) enum StabilityKind<'a> {
    Value,
    Prefix(PrefixKind<'a>),
    NonExhaustive(NonExhaustive<'a>),
}

impl<'a> StabilityKind<'a>{
    pub(crate) fn field_accessor(
        &self,
        mod_refl_mode:ModReflMode<usize>,
        field:&Field<'a>,
    )->FieldAccessor<'a>{
        let is_public=field.is_public() && mod_refl_mode!=ModReflMode::Opaque;
        match (is_public,self) {
            (false,_)=>
                FieldAccessor::Opaque,
            (true,StabilityKind::Value)|(true,StabilityKind::NonExhaustive{..})=>
                FieldAccessor::Direct,
            (true,StabilityKind::Prefix(prefix))=>
                prefix.field_accessor(field),
        }
    }
}


impl<'a> StableAbiOptions<'a> {
    fn new(
        ds: &'a DataStructure<'a>, 
        mut this: StableAbiAttrs<'a>,
        arenas: &'a Arenas,
    ) -> Result<Self,syn::Error> {
        let mut phantom_fields=Vec::<(&'a Ident,&'a Type)>::new();

        let repr = ReprAttr::new(this.repr)?;

        let mut errors=LinearResult::ok(());

        let kind = match this.kind {
            _ if repr.is_repr_transparent() => {
                // let field=&ds.variants[0].fields[0];
                
                // let accessor_bound=syn::parse_str::<WherePredicate>(
                //     &format!("({}):__StableAbi",(&field.mutated_ty).into_token_stream())
                // ).expect(concat!(file!(),"-",line!()));
                // this.extra_bounds.push(accessor_bound);

                StabilityKind::Value
            }
            UncheckedStabilityKind::Value => StabilityKind::Value,
            UncheckedStabilityKind::Prefix(prefix)=>{
                StabilityKind::Prefix(PrefixKind::new(
                    this.first_suffix_field,
                    prefix.prefix_struct,
                    mem::replace(&mut this.prefix_kind_fields,FieldMap::empty())
                        .map(|fi,pk_field|{
                            AccessorOrMaybe::new(
                                fi,
                                this.first_suffix_field,
                                pk_field,
                                this.default_on_missing_fields.unwrap_or_default(),
                            ) 
                        }),
                    this.prefix_bounds,
                    this.accessor_bounds,
                ))
            }
            UncheckedStabilityKind::NonExhaustive(nonexhaustive)=>{
                let variant_constructor=this.variant_constructor;
                nonexhaustive
                    .piped(|x| NonExhaustive::new(x,variant_constructor,ds,arenas) )?
                    .piped(StabilityKind::NonExhaustive)
            }
        };

        match (repr,ds.data_variant) {
            (ReprAttr::Transparent{..},DataVariant::Struct)=>{}
            (ReprAttr::Transparent(span),_)=>{
                errors.push_err(syn_err!(
                    *span,
                    "\nAbiStable does not suport non-struct #[repr(transparent)] types.\n"
                ));
            }
            (ReprAttr::Int{..},DataVariant::Enum)=>{}
            (ReprAttr::Int(_,span),_)=>{
                errors.push_err(syn_err!(
                    *span,
                    "AbiStable does not suport non-enum #[repr(<some_integer_type>)] types."
                ));
            }
            (ReprAttr::C{..},_)=>{}
        }

        let mod_refl_mode=match this.mod_refl_mode {
            Some(ModReflMode::Module)=>ModReflMode::Module,
            Some(ModReflMode::Opaque)=>ModReflMode::Opaque,
            Some(ModReflMode::DelegateDeref(()))=>{
                let index=phantom_fields.len();
                let field_ty=syn::parse_str::<Type>("<Self as ::std::op::Deref>::Target")
                    .expect("BUG")
                    .piped(|x| arenas.alloc(x) );

                let dt=arenas.alloc(parse_str_as_ident("deref_target"));
                phantom_fields.push((dt,field_ty));

                &[
                    "Self: ::std::ops::Deref",
                    "<Self as ::std::ops::Deref>::Target:__SharedStableAbi",
                ].iter()
                 .map(|x| syn::parse_str::<WherePredicate>(x).expect("BUG") )
                 .extending(&mut this.extra_bounds);

                 ModReflMode::DelegateDeref(index)
            }
            None if ds.has_public_fields() =>
                ModReflMode::Module,
            None=>
                ModReflMode::Opaque,
        };

        phantom_fields.extend(this.extra_phantom_fields);
        phantom_fields.extend(
            this.phantom_type_params.iter().cloned()
                .enumerate()
                .map(|(i,ty)|{
                    let x=format!("_phantom_ty_param_{}",i);
                    let name=arenas.alloc(parse_str_as_ident(&x));
                    (name,ty)
                })
        );

        let doc_hidden_attr=if this.is_hidden {
            Some(arenas.alloc(quote!(#[doc(hidden)])))
        }else{
            None
        };

        errors.into_result()?;

        Ok(StableAbiOptions {
            debug_print: this.debug_print,
            kind, repr ,
            extra_bounds : this.extra_bounds,
            type_param_bounds: this.type_param_bounds,
            layout_ctor: this.layout_ctor,
            renamed_fields: this.renamed_fields,
            changed_types: this.changed_types,
            override_field_accessor: this.override_field_accessor,
            tags: this.tags,
            extra_checks: this.extra_checks,
            impl_interfacetype: this.impl_interfacetype,
            phantom_fields,
            phantom_type_params: this.phantom_type_params,
            phantom_const_params: this.phantom_const_params,
            allow_type_macros: this.allow_type_macros,
            with_field_indices: this.with_field_indices,
            mod_refl_mode,
            doc_hidden_attr,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct StableAbiAttrs<'a> {
    debug_print:bool,
    kind: UncheckedStabilityKind<'a>,
    repr: UncheckedReprAttr,

    extra_bounds:Vec<WherePredicate>,

    tags:Option<syn::Expr>,
    extra_checks:Option<syn::Expr>,


    first_suffix_field:FirstSuffixField,
    default_on_missing_fields:Option<OnMissingField<'a>>,
    prefix_kind_fields:FieldMap<PrefixKindField<'a>>,

    prefix_bounds:Vec<WherePredicate>,

    type_param_bounds:TypeParamMap<'a,ASTypeParamBound>,

    layout_ctor:FieldMap<LayoutConstructor>,

    variant_constructor:Vec<Option<UncheckedVariantConstructor>>,

    override_field_accessor:FieldMap<Option<FieldAccessor<'a>>>,
    
    renamed_fields:FieldMap<Option<&'a Ident>>,
    changed_types:FieldMap<Option<&'a Type>>,

    accessor_bounds:FieldMap<Vec<TypeParamBound>>,

    extra_phantom_fields:Vec<(&'a Ident,&'a Type)>,
    phantom_type_params:Vec<&'a Type>,
    phantom_const_params:Vec<&'a syn::Expr>,

    impl_interfacetype:Option<ImplInterfaceType>,
    
    mod_refl_mode:Option<ModReflMode<()>>,

    allow_type_macros:bool,
    with_field_indices:bool,
    is_hidden:bool,

    errors:LinearResult<()>,
}


#[derive(Clone)]
enum UncheckedStabilityKind<'a> {
    Value,
    Prefix(UncheckedPrefixKind<'a>),
    NonExhaustive(UncheckedNonExhaustive<'a>),
}

#[derive(Copy, Clone)]
struct UncheckedPrefixKind<'a>{
    prefix_struct:&'a Ident,
}

impl<'a> Default for UncheckedStabilityKind<'a>{
    fn default()->Self{
        UncheckedStabilityKind::Value
    }
}



#[derive(Debug,Copy, Clone)]
enum ParseContext<'a> {
    TypeAttr{
        name:&'a Ident,
    },
    Variant{
        variant_index:usize,
    },
    Field{
        field_index:usize,
        field:&'a Field<'a>,
    },
}

/// Parses the attributes for the `StableAbi` derive macro.
pub(crate) fn parse_attrs_for_stable_abi<'a,I>(
    attrs: I,
    ds: &'a DataStructure<'a>,
    arenas: &'a Arenas,
) -> Result<StableAbiOptions<'a>,syn::Error>
where
    I:IntoIterator<Item=&'a Attribute>
{
    let mut this = StableAbiAttrs::default();

    this.layout_ctor=FieldMap::defaulted(ds);
    this.prefix_kind_fields=FieldMap::defaulted(ds);
    this.renamed_fields=FieldMap::defaulted(ds);
    this.override_field_accessor=FieldMap::defaulted(ds);
    this.accessor_bounds=FieldMap::defaulted(ds);
    this.changed_types=FieldMap::defaulted(ds);
    this.variant_constructor.resize(ds.variants.len(),None);
    
    this.type_param_bounds=TypeParamMap::defaulted(ds);

    let name=ds.name;

    parse_inner(&mut this, attrs, ParseContext::TypeAttr{name}, arenas)?;

    for (variant_index,variant) in ds.variants.iter().enumerate() {
        parse_inner(&mut this, variant.attrs, ParseContext::Variant{variant_index}, arenas)?;
        for (field_index,field) in variant.fields.iter().enumerate() {
            parse_inner(&mut this, field.attrs, ParseContext::Field{field,field_index}, arenas)?;
        }
    }

    this.errors.take()?;

    StableAbiOptions::new(ds, this, arenas)
}

/// Parses an individual attribute
fn parse_inner<'a,I>(
    this: &mut StableAbiAttrs<'a>,
    attrs: I,
    pctx: ParseContext<'a>,
    arenas: &'a Arenas,
)-> Result<(),syn::Error>
where
    I:IntoIterator<Item=&'a Attribute>
{
    for attr in attrs {
        match attr.parse_meta() {
            Ok(Meta::List(list)) => {
                parse_attr_list(this,pctx, list, arenas)
                    .combine_into_err(&mut this.errors);
            }
            Err(e)=>{
                this.errors.push_err(e);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Parses an individual attribute list (A `#[attribute( .. )] attribute`).
fn parse_attr_list<'a>(
    this: &mut StableAbiAttrs<'a>,
    pctx: ParseContext<'a>,
    list: MetaList, 
    arenas: &'a Arenas
)-> Result<(),syn::Error> {
    if list.path.equals_str("repr") {
        fn make_err(tokens:&dyn ToTokens)->syn::Error{
            spanned_err!(
                tokens,
                "repr attribute not currently recognized by this macro.{}",
                REPR_ERROR_MSG
            )
        }
        with_nested_meta("repr", list.nested, |attr| match attr {
            Meta::Path(ref path)=> {
                let ident=path.get_ident().ok_or_else(|| make_err(path) )?;
                let span=ident.span();

                if ident=="C" {
                    this.repr.set_repr_kind(UncheckedReprKind::C,span)
                }else if ident=="transparent" {
                    this.repr.set_repr_kind(UncheckedReprKind::Transparent,span)
                }else if let Some(dr)=DiscriminantRepr::from_ident(ident) {
                    this.repr.set_discriminant_repr(dr,span)
                }else{
                    Err(make_err(ident))
                }.combine_into_err(&mut this.errors);
                Ok(())
            }
            Meta::List(ref list) if list.path.equals_str("align") => {
                Ok(())
            }
            x => {
                Err(make_err(&x))
            }
        }).combine_into_err(&mut this.errors);
    } else if list.path.equals_str("doc") {
        with_nested_meta("doc", list.nested, |attr| {
            match attr {
                Meta::Path(ref path)=> {
                    if path.equals_str("hidden") {
                        this.is_hidden=true;
                    }
                }
                _=>{}
            }
            Ok(())
        })?;
    } else if list.path.equals_str("sabi") {
        with_nested_meta("sabi", list.nested, |attr| {
            parse_sabi_attr(this,pctx, attr, arenas)
                .combine_into_err(&mut this.errors);
            Ok(())
        })?;
    }
    Ok(())
}

/// Parses the contents of a `#[sabi( .. )]` attribute.
fn parse_sabi_attr<'a>(
    this: &mut StableAbiAttrs<'a>,
    pctx: ParseContext<'a>, 
    attr: Meta, 
    arenas: &'a Arenas
)-> Result<(),syn::Error> {
    fn make_err(tokens:&dyn ToTokens)->syn::Error{
        spanned_err!(tokens,"unrecognized attribute")
    }
    match (pctx, attr) {
        (ParseContext::Field{field,field_index}, Meta::Path(path)) => {
            let word=path.get_ident().ok_or_else(|| make_err(&path) )?;

            if word == "unsafe_opaque_field" {
                this.layout_ctor[field]=LayoutConstructor::Opaque;
            }else if word=="unsafe_sabi_opaque_field" {
                this.layout_ctor[field]=LayoutConstructor::SabiOpaque;
            }else if word == "last_prefix_field" {
                let field_pos=field_index+1;
                this.first_suffix_field=FirstSuffixField{field_pos};
            }else{
                return Err(make_err(&path))?;
            }
        }
        (
            ParseContext::Field{field,..}, 
            Meta::NameValue(MetaNameValue{lit:Lit::Str(ref value),ref path,..})
        ) => {
            let ident=path.get_ident().ok_or_else(|| make_err(&path) )?;
            
            if ident=="rename" {
                let renamed=value
                    .parse::<Ident>()?
                    .piped(|x| arenas.alloc(x) );
                this.renamed_fields.insert(field,Some(renamed));
            }else if ident=="unsafe_change_type" {
                let changed_type=parse_lit_as_type(&value)?
                    .piped(|x| arenas.alloc(x) );
                this.changed_types.insert(field,Some(changed_type));
            }else if ident=="accessible_if" {
                let expr=arenas.alloc(parse_lit_as_expr(&value)?);
                this.prefix_kind_fields[field].accessible_if=Some(expr);
            }else if ident == "accessor_bound" {
                let bound=parse_lit_as_type_bounds(&value)?;
                this.accessor_bounds[field].extend(bound);
            }else if ident == "bound" {
                let bounds=parse_lit_as_type_bounds(&value)?;
                let preds=where_predicate_from(field.ty.clone(), bounds);
                this.extra_bounds.push(preds);
            }else{
                return Err(make_err(&path))?;
            }
        }
        (ParseContext::Field{field,..}, Meta::List(list)) => {
            let ident=list.path.get_ident().ok_or_else(|| make_err(&list.path) )?;
            
            if ident=="missing_field" {
                let on_missing_field=parse_missing_field(&list.nested,arenas)?;
                let on_missing=&mut this.prefix_kind_fields[field].on_missing;
                if on_missing.is_some(){
                    return_spanned_err!(
                        ident,
                        "Cannot use this attribute multiple times on the same field"
                    );
                }
                *on_missing=Some(on_missing_field);
            }else if ident=="refl" {
                parse_refl_field(this,field,list.nested,arenas)?;
            }else{
                return_spanned_err!(ident,"unrecognized field attribute");
            }
        }
        (ParseContext::TypeAttr{..},Meta::Path(ref path)) if path.equals_str("debug_print") =>{
            this.debug_print=true;
        }
        (
            ParseContext::TypeAttr{..},
            Meta::NameValue(MetaNameValue{lit:Lit::Str(ref unparsed_lit),ref path,..})
        )=>{
            let ident=path.get_ident().ok_or_else(|| make_err(path) )?;

            if ident=="bound"||ident=="prefix_bound" {
                let ident=path.get_ident().ok_or_else(|| make_err(path) )?;

                let bound=unparsed_lit.parse::<WherePredicate>()?;
                if ident=="bound"{
                    this.extra_bounds.push(bound);
                }else if ident=="prefix_bound" {
                    this.prefix_bounds.push(bound);
                }
            }else if ident=="bounds"||ident=="prefix_bounds" {
                let ident=path.get_ident().ok_or_else(|| make_err(path) )?;

                let bound=unparsed_lit
                    .parse::<ParsePunctuated<WherePredicate,Comma>>()?
                    .list;
                if ident=="bounds"{
                    this.extra_bounds.extend(bound);
                }else if ident=="prefix_bounds" {
                    this.prefix_bounds.extend(bound);
                }
            }else if ident=="phantom_field"{
                let unparsed_field=unparsed_lit.value();
                let mut iter=unparsed_field.splitn(2,':');
                let name={
                    let x=iter.next().unwrap_or("");
                    let x=syn::parse_str::<Ident>(x)?;
                    arenas.alloc(x)
                };
                let ty=arenas.alloc(parse_str_as_type(iter.next().unwrap_or(""))?);
                this.extra_phantom_fields.push((name,ty));
            }else if ident=="phantom_type_param"{
                let ty=arenas.alloc(parse_lit_as_type(unparsed_lit)?);
                this.phantom_type_params.push(ty);
            }else if ident=="phantom_const_param"{
                let constant=arenas.alloc(parse_lit_as_expr(unparsed_lit)?);
                this.phantom_const_params.push(constant);
            }else if ident=="tag"||ident=="extra_checks" {
                let bound=unparsed_lit.parse::<syn::Expr>();

                if ident=="tag" {
                    if this.tags.is_some() {
                        return_spanned_err!(
                            unparsed_lit,
                            "\
                            Cannot specify multiple tags,\
                            you must choose whether you want array or set semantics \
                            when adding more tags.\n\
                            \n\
                            For multiple elements you can do:\n\
                            \n\
                            - `tag![[ tag0,tag1 ]]` or `Tag::arr(&[ tag0,tag1 ])` :\n\
                                \tThis will require that the tags match exactly between \
                                interface and implementation.\n\
                            \n\
                            - `tag!{{ tag0,tag1 }}` or `Tag::set(&[ tag0,tag1 ])` :\n\
                                \tThis will require that the tags in the interface are \
                                a subset of the implementation.\n\
                            ",
                        );
                    }
                    this.tags=Some(bound?);
                }else if ident=="extra_checks" {
                    if this.extra_checks.is_some() {
                        return_spanned_err!(
                            ident,
                            "Cannot use the `#[sabi(extra_checks=\"\")]` \
                             attribute multiple times,\
                            "
                        );
                    }
                    
                    this.extra_checks=Some(bound?);
                }

            }else{
                return Err(make_err(path));
            }
        }
        (ParseContext::TypeAttr{name},Meta::List(list)) => {
            let ident=list.path.get_ident().ok_or_else(|| make_err(&list.path) )?;

            if ident == "missing_field" {
                let on_missing=&mut this.default_on_missing_fields;
                if on_missing.is_some(){
                    return_spanned_err!(
                        ident,
                        "Cannot use this attribute multiple times on the container"
                    );
                }
                *on_missing=Some(parse_missing_field(&list.nested,arenas)?);
            } else if ident == "kind" {
                with_nested_meta("kind", list.nested, |attr|{
                    match attr {
                        Meta::Path(ref path) if path.equals_str("Value") => {
                            this.kind = UncheckedStabilityKind::Value;
                        }
                        Meta::List(ref list)=>{
                            let ident=match list.path.get_ident() {
                                Some(x)=>x,
                                None=>return_spanned_err!(list,"invalid #[kind(..)] attribute"),
                            };

                            if ident == "Prefix" {
                                let prefix=parse_prefix_type_list(name,&list.nested,arenas)?;
                                this.kind = UncheckedStabilityKind::Prefix(prefix);
                            }else if ident == "WithNonExhaustive" {
                                let nonexhaustive=
                                    parse_non_exhaustive_list(name,&list.nested,arenas)?;
                                this.kind = UncheckedStabilityKind::NonExhaustive(nonexhaustive);
                            }else{
                                this.errors.push_err(spanned_err!(
                                    ident,
                                    "invalid #[kind(..)] attribute"
                                ));
                            }
                        }
                        x => this.errors.push_err(spanned_err!(
                            x,
                            "invalid #[kind(..)] attribute",
                        )),
                    }
                    Ok(())
                })?;
            } else if ident == "module_reflection" {
                fn mrefl_err(tokens:&dyn ToTokens)->syn::Error{
                    spanned_err!(
                        tokens,
                        "invalid #[module_reflection(..)] attribute."
                    )
                }

                with_nested_meta("module_reflection", list.nested, |attr| {
                    if this.mod_refl_mode.is_some() {
                        return_spanned_err!(ident,"Cannot use this attribute multiple times");
                    }
                    
                    match attr {
                        Meta::Path(ref path)=>{
                            let word=path.get_ident().ok_or_else(|| mrefl_err(path) )?;

                            if word == "Module" {
                                this.mod_refl_mode = Some(ModReflMode::Module);
                            }else if word == "Opaque" {
                                this.mod_refl_mode = Some(ModReflMode::Opaque);
                            }else if word == "Deref" {
                                this.mod_refl_mode = Some(ModReflMode::DelegateDeref(()));
                            }else{
                                this.errors.push_err(mrefl_err(word));
                            }
                        } 
                        ref x => this.errors.push_err(mrefl_err(x)),
                    }
                    Ok(())
                })?;
            } else if ident == "not_stableabi" {
                fn nsabi_err(tokens:&dyn ToTokens)->syn::Error{
                    spanned_err!(
                        tokens,
                        "invalid #[not_stableabi(..)] attribute\
                         (it must be the identifier of a type parameter)."
                    )
                }

                with_nested_meta("not_stableabi", list.nested, |attr|{
                    match attr {
                        Meta::Path(path)=>{
                            let type_param=path.into_ident().map_err(|p| nsabi_err(&p) )?;

                            *this.type_param_bounds.get_mut(&type_param)?=
                                ASTypeParamBound::GetStaticEquivalent;
                        }
                        x => this.errors.push_err(nsabi_err(&x)),
                    }
                    Ok(())
                })?;
            } else if ident == "shared_stableabi" {
                fn nsabi_err(tokens:&dyn ToTokens)->syn::Error{
                    spanned_err!(
                        tokens,
                        "invalid #[shared_stableabi(..)] attribute\
                         (it must be the identifier of a type parameter)."
                    )
                }

                with_nested_meta("shared_stableabi", list.nested, |attr|{
                    match attr {
                        Meta::Path(path)=>{
                            let type_param=path.into_ident().map_err(|p| nsabi_err(&p) )?;

                            *this.type_param_bounds.get_mut(&type_param)?=
                                ASTypeParamBound::SharedStableAbi;
                        }
                        x => this.errors.push_err(nsabi_err(&x)),
                    }
                    Ok(())
                })?;
            } else if ident == "unsafe_unconstrained" {
                fn uu_err(tokens:&dyn ToTokens)->syn::Error{
                    spanned_err!(
                        tokens,
                        "invalid #[unsafe_unconstrained(..)] attribute\
                         (it must be the identifier of a type parameter)."
                    )
                }

                with_nested_meta("unsafe_unconstrained", list.nested, |attr| {
                    match attr {
                        Meta::Path(path)=>{
                            let type_param=path.into_ident().map_err(|p| uu_err(&p) )?;

                            *this.type_param_bounds.get_mut(&type_param)?=
                                ASTypeParamBound::NoBound;
                        }
                        x => this.errors.push_err(spanned_err!(
                            x,
                            "invalid #[unsafe_unconstrained(..)] attribute\
                             (it must be the identifier of a type parameter)."
                        )),
                    }
                    Ok(())   
                })?;
            } else if ident == "impl_InterfaceType" {
                if this.impl_interfacetype.is_some() {
                    return_spanned_err!(ident,"Cannot use this attribute multiple times")
                }
                this.impl_interfacetype=Some(parse_impl_interfacetype(&list.nested)?);
            }else{
                return_spanned_err!(
                    list,
                    "Unrecodnized #[sabi(..)] attribute",
                );
            }
        }
        (ParseContext::TypeAttr{..},Meta::Path(ref path))=>{
            let word=path.get_ident().ok_or_else(|| make_err(&path) )?;

            if word == "with_constructor" {
                this.variant_constructor.iter_mut()
                    .for_each(|x|*x=Some(UncheckedVariantConstructor::Regular));
            }else if word=="with_boxed_constructor" {
                this.variant_constructor.iter_mut()
                    .for_each(|x|*x=Some(UncheckedVariantConstructor::Boxed));
            }else if word=="unsafe_opaque_fields" {
                this.layout_ctor
                    .iter_mut()
                    .for_each(|(_,x)|*x=LayoutConstructor::Opaque);
            }else if word=="unsafe_sabi_opaque_fields" {
                this.layout_ctor
                    .iter_mut()
                    .for_each(|(_,x)|*x=LayoutConstructor::SabiOpaque);
            }else if word=="unsafe_allow_type_macros" {
                this.allow_type_macros=true;
            }else if word=="with_field_indices" {
                this.with_field_indices=true;
            }else{
                return Err(make_err(&path));
            }
        }
        (ParseContext::Variant{variant_index},Meta::Path(ref path))=>{
            let word=path.get_ident().ok_or_else(|| make_err(&path) )?;
            
            if word=="with_constructor" {
                this.variant_constructor[variant_index]=Some(UncheckedVariantConstructor::Regular);
            }else if word=="with_boxed_constructor" {
                this.variant_constructor[variant_index]=Some(UncheckedVariantConstructor::Boxed);
            }else{
                return Err(make_err(&path));
            }
        }
        (_,x) => return Err(make_err(&x)),
    }
    Ok(())
}


/// Parses the `#[sabi(refl="...")` attribute.
fn parse_refl_field<'a>(
    this: &mut StableAbiAttrs<'a>,
    field:&'a Field<'a>,
    list: Punctuated<NestedMeta, Comma>, 
    arenas: &'a Arenas
)-> Result<(),syn::Error> {
    fn make_err(tokens:&dyn ToTokens)->syn::Error{
        spanned_err!(tokens,"invalid #[sabi(refl(..))] attribute.")
    }

    with_nested_meta("refl", list, |attr| {
        match attr {
            Meta::NameValue(MetaNameValue{lit:Lit::Str(ref val),ref path,..})=>{
                let ident=path.get_ident().ok_or_else(|| make_err(path) )?;

                if ident=="pub_getter" {
                    let function=arenas.alloc(val.parse::<Ident>()?);
                    this.override_field_accessor[field]=
                        Some(FieldAccessor::Method{ name:Some(function) });
                }else{
                    this.errors.push_err(make_err(path));
                }
            }
            ref x => this.errors.push_err(make_err(x))
        }
        Ok(())
    })
}



/// Parses the contents of #[sabi(missing_field( ... ))]
fn parse_missing_field<'a>(
    list: &Punctuated<NestedMeta, Comma>, 
    arenas: &'a Arenas
)-> Result<OnMissingField<'a>,syn::Error> {
    const ATTRIBUTE_MSG:&'static str="

Valid Attributes:

    `#[sabi(missing_field(panic))]`
    This panics if the field doesn't exist.

    `#[sabi(missing_field(option))]`
    This returns Some(field_value) if the field exists,None if the field doesn't exist.
    This is the default.

    `#[sabi(missing_field(with=\"somefunction\"))]`
    This returns `somefunction()` if the field doesn't exist.
    
    `#[sabi(missing_field(value=\"some_expression\"))]`
    This returns `(some_expression)` if the field doesn't exist.
    
    `#[sabi(missing_field(default))]`
    This returns `Default::default` if the field doesn't exist.

";
    let mf_err=|tokens:&dyn ToTokens|->syn::Error{
        spanned_err!(
            tokens,
            "Invalid attribute.\n{}",
            ATTRIBUTE_MSG,
        )
    };

    let first_arg=list.into_iter().next();

    match first_arg {
        Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue{
            path,
            lit:Lit::Str(ref str_),
            ..
        })))=>{
            let nv_ident=path.get_ident().ok_or_else(|| mf_err(&path) )?;

            if nv_ident=="with" {
                let function=str_.parse::<syn::Path>()?
                    .piped(|i| arenas.alloc(i) );
                Ok(OnMissingField::With{function})
            }else if nv_ident=="value" {
                let value=str_.parse::<syn::Expr>()?
                    .piped(|i| arenas.alloc(i) );
                Ok(OnMissingField::Value{value})
            }else{
                Err(mf_err(&first_arg))
            }
        }
        Some(NestedMeta::Meta(Meta::Path(ref path)))=>{
            let word=path.get_ident().ok_or_else(|| mf_err(&first_arg) )?;

            if word=="option" {
                Ok(OnMissingField::ReturnOption)
            }else if word=="panic" {
                Ok(OnMissingField::Panic)
            }else if word=="default" {
                Ok(OnMissingField::Default_)
            }else{
                Err(mf_err(word))
            }
        }
        Some(rem)=>{
            Err(mf_err(&rem))
        }
        None=>Err(spanned_err!(
            list,
            "Error:Expected one attribute inside `missing_field(..)`\n{}",
            ATTRIBUTE_MSG
        )),
    }
}


/// Parses the contents of #[sabi(kind(Prefix( ... )))]
fn parse_prefix_type_list<'a>(
    _name:&'a Ident,
    list: &Punctuated<NestedMeta, Comma>, 
    arenas: &'a Arenas
)-> Result<UncheckedPrefixKind<'a>,syn::Error> {
    let mut iter=list.into_iter();

    let prefix_struct=match iter.next() {
        Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue{
            ref path,
            lit:Lit::Str(ref type_str),
            ..
        })))
        if path.equals_str("prefix_struct") =>{
            let type_str=type_str.value();
            parse_str_as_ident(&type_str)
                .piped(|i| arenas.alloc(i) )
        }
        ref x => return_spanned_err!(
            x,
            "invalid #[sabi(kind(Prefix(  )))] attribute\
             (it must be prefix_struct=\"NameOfPrefixStruct\" )."
        )
    };
    
    Ok(UncheckedPrefixKind{
        prefix_struct,
    })
}


/// Parses the contents of #[sabi(kind(WithNonExhaustive( ... )))]
fn parse_non_exhaustive_list<'a>(
    _name:&'a Ident,
    list: &Punctuated<NestedMeta, Comma>, 
    arenas: &'a Arenas
)-> Result<UncheckedNonExhaustive<'a>,syn::Error> {

    fn make_err(tokens:&dyn ToTokens,param:&str)->syn::Error{
        spanned_err!(
            tokens,
            "invalid #[sabi(kind(WithNonExhaustive({})))] attribute",
            param,
        )
    }

    let trait_set_strs=[
        "Clone","Display","Debug",
        "Eq","PartialEq","Ord","PartialOrd",
        "Hash","Deserialize","Serialize","Send","Sync","Error",
    ];    

    let trait_set=trait_set_strs
        .iter()
        .map(|e| arenas.alloc(Ident::new(e,Span::call_site()) ) )
        .collect::<HashSet<&'a Ident>>();

    let trait_err=|trait_ident:&dyn ToTokens|->syn::Error{
        spanned_err!(
            trait_ident,
            "Invalid trait in  #[sabi(kind(WithNonExhaustive(traits())))].\n\
             Valid traits:\n\t{}\
            ",
            trait_set_strs.join("\n\t")
        )
    };

    fn both_err(ident:&Ident)->syn::Error{
        spanned_err!(
            ident,
            "Cannot use both `interface=\"...\"` and `traits(...)`"
        )
    }
    

    let mut this=UncheckedNonExhaustive::default();

    let mut errors=LinearResult::ok(());

    for elem in list {
        match elem {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue{path,lit,..}))=>{
                let ident=path.get_ident().ok_or_else(|| make_err(&path,"") )?;

                match lit {
                    Lit::Int(int_lit)=>{
                        let int=IntOrType::Int(int_lit.base10_parse::<usize>()?);
                        if ident=="align" {
                            this.alignment=Some(int);
                        }else if ident=="size" {
                            this.size=Some(int);
                        }else{
                            return Err(make_err(ident,""))
                        }
                    }
                    Lit::Str(str_lit)=>{
                        let ty=arenas.alloc(parse_lit_as_type(str_lit)?);

                        if ident=="align" {
                            this.alignment=Some(IntOrType::Type(ty));
                        }else if ident=="size" {
                            this.size=Some(IntOrType::Type(ty));
                        }else if ident=="assert_nonexhaustive" {
                            this.assert_nonexh.push(ty);
                        }else if ident=="interface" {
                            if this.enum_interface.is_some() {
                                return Err(both_err(ident));
                            }
                            this.enum_interface=Some(EnumInterface::Old(ty));
                        }else{
                            errors.push_err(make_err(ident,""))
                        }
                    }
                    ref x => errors.push_err(make_err(x,"")),
                }
            }
            NestedMeta::Meta(Meta::List(sublist))=>{
                let ident=sublist.path.get_ident().ok_or_else(|| make_err(&sublist,"") )?;

                if ident=="assert_nonexhaustive" {
                    for assertion in &sublist.nested {
                        match assertion {
                            NestedMeta::Lit(Lit::Str(str_lit))=>{
                                let ty=arenas.alloc(parse_lit_as_type(str_lit)?);
                                this.assert_nonexh.push(ty);
                            }
                            x => errors.push_err(make_err(x,"assert_nonexhaustive( )"))
                        }
                    }
                }else if ident=="traits" {
                    let enum_interface=match &mut this.enum_interface {
                        Some(EnumInterface::New(x))=>x,
                        Some(EnumInterface::Old{..})=>{
                            return Err(both_err(ident));
                        }
                        x@None=>{
                            *x=Some(EnumInterface::New(Default::default()));
                            match x {
                                Some(EnumInterface::New(x))=>x,
                                _=>unreachable!()
                            }
                        }
                    };

                    for subelem in &sublist.nested {
                        let (ident,is_impld)=match subelem {
                            NestedMeta::Meta(Meta::Path(path))=>{
                                let ident=path.get_ident()
                                    .ok_or_else(|| trait_err(ident) )?;
                                (ident,true)
                            }
                            NestedMeta::Meta(Meta::NameValue(
                                MetaNameValue{path,lit:Lit::Bool(bool_lit),..}
                            ))=>{
                                let ident=path.get_ident()
                                    .ok_or_else(|| trait_err(ident) )?;
                                (ident,bool_lit.value)
                            }
                            x =>{
                                errors.push_err(trait_err(x));
                                continue
                            }
                        };

                        match trait_set.get(ident) {
                            Some(&trait_ident) => {
                                if is_impld {
                                    &mut enum_interface.impld
                                }else{
                                    &mut enum_interface.unimpld
                                }.push(trait_ident);
                            },
                            None =>errors.push_err(trait_err(ident))
                        }
                    }
                }else{
                    errors.push_err(make_err(ident,""));
                }
            }
            x => errors.push_err(make_err(x,""))
        }
    }

    errors.into_result().map(|_| this )
}


////////////////////////////////////////////////////////////////////////////////


fn where_predicate_from(
    ty:syn::Type,
    bounds:Punctuated<TypeParamBound, syn::token::Add>
) ->syn::WherePredicate {
    let x=syn::PredicateType{
        lifetimes: None,
        bounded_ty: ty,
        colon_token: Default::default(),
        bounds,
    };
    syn::WherePredicate::Type(x)
}






