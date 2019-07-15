use syn::{
    Attribute, Ident, Meta, MetaList, MetaNameValue, NestedMeta, 
    Lit,WherePredicate,Type,
    punctuated::Punctuated,
    token::Comma,
    TypeParamBound,
};

use std::{
    collections::{HashSet,HashMap},
    mem,
};

use core_extensions::IteratorExt;

use proc_macro2::Span;

use quote::ToTokens;

use crate::{
    attribute_parsing::with_nested_meta,
    datastructure::{DataStructure, DataVariant, Field,FieldMap},
    parse_utils::{
        parse_str_as_ident,
        parse_str_as_type,
        parse_lit_as_ident,
        parse_lit_as_expr,
        parse_lit_as_type,
        parse_lit_as_type_bounds,
    },
};

use super::{
    nonexhaustive::{UncheckedNonExhaustive,NonExhaustive,EnumInterface,IntOrType},
    reflection::{ModReflMode,FieldAccessor},
    prefix_types::{PrefixKind,FirstSuffixField,OnMissingField,AccessorOrMaybe,PrefixKindField},
    repr_attrs::{UncheckedReprAttr,UncheckedReprKind,DiscriminantRepr,ReprAttr},
};

use crate::*;

pub(crate) struct StableAbiOptions<'a> {
    pub(crate) debug_print:bool,
    pub(crate) kind: StabilityKind<'a>,
    pub(crate) repr: ReprAttr,

    /// The type parameters that have the __StableAbi constraint
    pub(crate) stable_abi_bounded:HashSet<&'a Ident>,
    pub(crate) stable_abi_bounded_ty:Vec<&'a Type>,

    pub(crate) unconstrained_type_params:HashMap<Ident,UnconstrainedTyParam<'a>>,

    pub(crate) extra_bounds:Vec<WherePredicate>,

    pub(crate) tags:Option<syn::Expr>,

    /// A hashset of the fields whose contents are opaque 
    /// (there are still some minimal checks going on).
    pub(crate) opaque_fields:FieldMap<bool>,

    pub(crate) override_field_accessor:FieldMap<Option<FieldAccessor<'a>>>,
    
    pub(crate) renamed_fields:FieldMap<Option<&'a Ident>>,
    pub(crate) changed_types:FieldMap<Option<&'a Type>>,

    pub(crate) mod_refl_mode:ModReflMode<usize>,

    pub(crate) phantom_fields:Vec<(&'a str,&'a Type)>,

}


pub(crate) struct UnconstrainedTyParam<'a>{
    pub(crate) static_equivalent:Option<&'a Type>
}


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
    ) -> Self {
        let mut phantom_fields=Vec::<(&'a str,&'a Type)>::new();

        let repr = ReprAttr::new(this.repr);

        let mut stable_abi_bounded=ds.generics
            .type_params()
            .map(|x| &x.ident )
            .collect::<HashSet<&'a Ident>>();

        for (type_param,_) in &this.unconstrained_type_params {
            if !stable_abi_bounded.remove(type_param) {
                panic!(
                    "'{}' declared as a phantom type parameter but is not a type parameter",
                    type_param 
                )
            }
        }

        let kind = match this.kind {
            _ if repr == ReprAttr::Transparent => {
                // let field=&ds.variants[0].fields[0];
                
                // let field_bound=syn::parse_str::<WherePredicate>(
                //     &format!("({}):__StableAbi",(&field.mutated_ty).into_token_stream())
                // ).expect(concat!(file!(),"-",line!()));
                // this.extra_bounds.push(field_bound);

                StabilityKind::Value
            }
            UncheckedStabilityKind::Value => StabilityKind::Value,
            UncheckedStabilityKind::Prefix(prefix)=>{
                StabilityKind::Prefix(PrefixKind{
                    first_suffix_field:this.first_suffix_field,
                    prefix_struct:prefix.prefix_struct,
                    fields:mem::replace(&mut this.prefix_kind_fields,FieldMap::empty())
                        .map(|fi,pk_field|{
                            AccessorOrMaybe::new(
                                fi,
                                this.first_suffix_field,
                                pk_field,
                                this.default_on_missing_fields,
                            ) 
                        }),
                    prefix_bounds:this.prefix_bounds,
                    field_bounds:this.field_bounds,
                })
            }
            UncheckedStabilityKind::NonExhaustive(nonexhaustive)=>{
                nonexhaustive
                    .piped(|x| NonExhaustive::new(x,ds,arenas) )
                    .piped(StabilityKind::NonExhaustive)
            }
        };

        match (repr,ds.data_variant) {
            (ReprAttr::Transparent,DataVariant::Struct)=>{}
            (ReprAttr::Transparent,_)=>{
                panic!("\nAbiStable does not suport non-struct #[repr(transparent)] types.\n");
            }
            (ReprAttr::Int{..},DataVariant::Enum)=>{}
            (ReprAttr::Int{..},_)=>{
                panic!("\n\
                    AbiStable does not suport non-enum #[repr(<some_integer_type>)] types.\
                \n");
            }
            (ReprAttr::C{..},_)=>{}
        }

        let mod_refl_mode=match this.mod_refl_mode {
            Some(ModReflMode::Module)=>ModReflMode::Module,
            Some(ModReflMode::Opaque)=>ModReflMode::Opaque,
            Some(ModReflMode::DelegateDeref(()))=>{
                let index=phantom_fields.len();
                let field_ty=syn::parse_str::<Type>("<Self as ::std::op::Deref>::Target")
                    .unwrap()
                    .piped(|x| arenas.alloc(x) );

                phantom_fields.push(("deref_target",field_ty));

                &[
                    "Self: ::std::ops::Deref",
                    "<Self as ::std::ops::Deref>::Target:__SharedStableAbi",
                ].iter()
                 .map(|x| syn::parse_str::<WherePredicate>(x).unwrap() )
                 .extending(&mut this.extra_bounds);

                 ModReflMode::DelegateDeref(index)
            }
            None if ds.has_public_fields() =>
                ModReflMode::Module,
            None=>
                ModReflMode::Opaque,
        };

        let stable_abi_bounded_ty:Vec<&'a syn::Type>=
            this.extra_phantom_fields.iter().map(|(_,ty)| *ty ).collect();

        phantom_fields.extend(this.extra_phantom_fields);

        StableAbiOptions {
            debug_print:this.debug_print,
            kind, repr , stable_abi_bounded , stable_abi_bounded_ty,
            extra_bounds :this.extra_bounds,
            unconstrained_type_params:this.unconstrained_type_params.into_iter().collect(),
            opaque_fields:this.opaque_fields,
            renamed_fields:this.renamed_fields,
            changed_types:this.changed_types,
            override_field_accessor:this.override_field_accessor,
            tags:this.tags,
            phantom_fields,
            mod_refl_mode,
        }
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


    first_suffix_field:FirstSuffixField,
    default_on_missing_fields:OnMissingField<'a>,
    prefix_kind_fields:FieldMap<PrefixKindField<'a>>,

    prefix_bounds:Vec<WherePredicate>,

    /// The type parameters that have no constraints
    unconstrained_type_params:Vec<(Ident,UnconstrainedTyParam<'a>)>,

    // Using raw pointers to do an identity comparison.
    opaque_fields:FieldMap<bool>,

    override_field_accessor:FieldMap<Option<FieldAccessor<'a>>>,
    
    renamed_fields:FieldMap<Option<&'a Ident>>,
    changed_types:FieldMap<Option<&'a Type>>,

    field_bounds:FieldMap<Vec<TypeParamBound>>,

    extra_phantom_fields:Vec<(&'a str,&'a Type)>,
    
    mod_refl_mode:Option<ModReflMode<()>>,
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



#[derive(Copy, Clone)]
enum ParseContext<'a> {
    TypeAttr{
        name:&'a Ident,
    },
    Field{
        field_index:usize,
        field:&'a Field<'a>,
    },
}

pub(crate) fn parse_attrs_for_stable_abi<'a,I>(
    attrs: I,
    ds: &'a DataStructure<'a>,
    arenas: &'a Arenas,
) -> StableAbiOptions<'a> 
where
    I:IntoIterator<Item=&'a Attribute>
{
    let mut this = StableAbiAttrs::default();

    this.opaque_fields=FieldMap::defaulted(ds);
    this.prefix_kind_fields=FieldMap::defaulted(ds);
    this.renamed_fields=FieldMap::defaulted(ds);
    this.override_field_accessor=FieldMap::defaulted(ds);
    this.field_bounds=FieldMap::defaulted(ds);
    this.changed_types=FieldMap::defaulted(ds);

    let name=ds.name;

    parse_inner(&mut this, attrs, ParseContext::TypeAttr{name}, arenas);

    for variant in &ds.variants {
        for (field_index,field) in variant.fields.iter().enumerate() {
            parse_inner(&mut this, field.attrs, ParseContext::Field{field,field_index}, arenas);
        }
    }

    StableAbiOptions::new(ds, this,arenas)
}

fn parse_inner<'a,I>(
    this: &mut StableAbiAttrs<'a>,
    attrs: I,
    pctx: ParseContext<'a>,
    arenas: &'a Arenas,
) 
where
    I:IntoIterator<Item=&'a Attribute>
{
    for attr in attrs {
        match attr.parse_meta().unwrap() {
            Meta::List(list) => {
                parse_attr_list(this,pctx, list, arenas);
            }
            _ => {}
        }
    }
}

fn parse_attr_list<'a>(
    this: &mut StableAbiAttrs<'a>,
    pctx: ParseContext<'a>,
    list: MetaList, 
    arenas: &'a Arenas
) {
    if list.ident == "repr" {
        with_nested_meta("repr", list.nested, |attr| match attr {
            Meta::Word(ref ident)=> {
                if ident=="C" {
                    this.repr.set_repr_kind(UncheckedReprKind::C);
                }else if ident=="transparent" {
                    this.repr.set_repr_kind(UncheckedReprKind::Transparent);
                }else if let Some(dr)=DiscriminantRepr::from_ident(ident) {
                    this.repr.set_discriminant_repr(dr);
                }else{
                    panic!(
                        "repr attribute not currently recognized by this macro:\n{:?}",
                        ident
                    )
                }
            }
            Meta::List(ref list) if list.ident == "align" => {}
            x => panic!(
                "repr attribute not currently recognized by this macro:\n{:?}",
                x
            ),
        })
    } else if list.ident == "sabi" {
        with_nested_meta("sabi", list.nested, |attr| {
            parse_sabi_attr(this,pctx, attr, arenas)
        });
    }
}

fn parse_sabi_attr<'a>(
    this: &mut StableAbiAttrs<'a>,
    pctx: ParseContext<'a>, 
    attr: Meta, 
    arenas: &'a Arenas
) {
    match (pctx, attr) {
        (ParseContext::Field{field,field_index}, Meta::Word(word)) => {
            if word == "unsafe_opaque_field" {
                this.opaque_fields[field]=true;
            }else if word == "last_prefix_field" {
                let field_pos=field_index+1;
                this.first_suffix_field=FirstSuffixField{field_pos};
            }else{
                panic!("unrecognized field attribute `#[sabi({})]` ",word);
            }
        }
        (
            ParseContext::Field{field,..}, 
            Meta::NameValue(MetaNameValue{lit:Lit::Str(ref value),ref ident,..})
        ) => {
            if ident=="rename" {
                let renamed=parse_lit_as_ident(&value)
                    .piped(|x| arenas.alloc(x) );
                this.renamed_fields.insert(field,Some(renamed));
            }else if ident=="unsafe_change_type" {
                let changed_type=parse_lit_as_type(&value)
                    .piped(|x| arenas.alloc(x) );
                this.changed_types.insert(field,Some(changed_type));
            }else if ident=="accessible_if" {
                let expr=arenas.alloc(parse_lit_as_expr(&value));
                this.prefix_kind_fields[field].accessible_if=Some(expr);
            }else if ident == "field_bound" {
                let bound=parse_lit_as_type_bounds(&value);
                this.field_bounds[field].extend(bound);
            }else{
                panic!(
                    "unrecognized field attribute `#[sabi({}={})]` ",
                    ident,
                    value.value()
                );
            }
        }
        (ParseContext::Field{field,..}, Meta::List(list)) => {
            if list.ident == "missing_field" {
                let on_missing_field=parse_missing_field(&list.nested,arenas);
                this.prefix_kind_fields[field].on_missing=Some(on_missing_field);
            }else if list.ident == "refl" {
                parse_refl_field(this,field,list.nested,arenas);
            }else{
                panic!("unrecognized field attribute `#[sabi({})]` ",list.ident);
            }
        }
        (ParseContext::TypeAttr{..},Meta::Word(ref word)) if word == "debug_print" =>{
            this.debug_print=true;
        }
        (
            ParseContext::TypeAttr{..},
            Meta::NameValue(MetaNameValue{lit:Lit::Str(ref unparsed_bound),ref ident,..})
        )if ident=="bound"||ident=="prefix_bound" =>
        {
            let bound=match unparsed_bound.parse::<WherePredicate>() {
                Ok(v)=>v,
                Err(e)=>panic!(
                    "\n\nInvalid bound:\n\t{}\nError:\n\t{}\n\n",
                    unparsed_bound.value(),
                    e
                ),
            };
            if ident=="bound"{
                this.extra_bounds.push(bound);
            }else if ident=="prefix_bound" {
                this.prefix_bounds.push(bound);
            }
        }
        (
            ParseContext::TypeAttr{..},
            Meta::NameValue(MetaNameValue{lit:Lit::Str(ref unparsed_field),ref ident,..})
        )if ident=="phantom_field" =>
        {
            let unparsed_field=unparsed_field.value();
            let mut iter=unparsed_field.splitn(2,':');
            let name=arenas.alloc(iter.next().unwrap_or("").to_string());
            let ty=arenas.alloc(parse_str_as_type(iter.next().unwrap_or("")));
            this.extra_phantom_fields.push((name,ty));
        }
        (
            ParseContext::TypeAttr{..},
            Meta::NameValue(MetaNameValue{lit:Lit::Str(ref unparsed_tag),ref ident,..})
        )if ident=="tag" =>
        {
            if this.tags.is_some() {
                panic!("\n\n\
Cannot specify multiple tags,\
you must choose whether you want array or set semantics \
when adding more tags.
For array semantics you can do:

- `tag![[ tag0,tag1 ]]` or `Tag::arr(&[ tag0,tag1 ])` :
    This will require that the tags match exactly between interface and implementation.

- `tag!{{ tag0,tag1 }}` or `Tag::set(&[ tag0,tag1 ])` :
    This will require that the tags in the interface are a subset of the implementation.

Tag:\n\t{}\n",
                    unparsed_tag.value(),
                );
            }

            let bound=match unparsed_tag.parse::<syn::Expr>() {
                Ok(v)=>v,
                Err(e)=>panic!(
                    "\n\nInvalid tag expression:\n\t{}\nError:\n\t{}\n\n",
                    unparsed_tag.value(),
                    e
                ),
            };
            this.tags=Some(bound);
        }
        (ParseContext::TypeAttr{name},Meta::List(list)) => {
            if list.ident == "override" {
                with_nested_meta("override", list.nested, |attr| {
                    parse_sabi_override(this, attr, arenas)
                });
            } else if list.ident == "missing_field" {
                this.default_on_missing_fields=parse_missing_field(&list.nested,arenas);
            } else if list.ident == "kind" {
                with_nested_meta("kind", list.nested, |attr| match attr {
                    Meta::Word(ref word) if word == "Value" => {
                        this.kind = UncheckedStabilityKind::Value;
                    }
                    Meta::List(ref list) if list.ident == "Prefix" => {
                        let prefix=parse_prefix_type_list(name,&list.nested,arenas);
                        this.kind = UncheckedStabilityKind::Prefix(prefix);
                    }
                    Meta::List(ref list) if list.ident == "WithNonExhaustive" => {
                        let nonexhaustive=parse_non_exhaustive_list(name,&list.nested,arenas);
                        this.kind = UncheckedStabilityKind::NonExhaustive(nonexhaustive);
                    }
                    x => panic!("invalid #[kind(..)] attribute:\n{:?}", x),
                });
            } else if list.ident == "module_reflection" {
                with_nested_meta("kind", list.nested, |attr| match attr {
                    Meta::Word(ref word) if word == "Module" => {
                        this.mod_refl_mode = Some(ModReflMode::Module);
                    }
                    Meta::Word(ref word) if word == "Opaque" => {
                        this.mod_refl_mode = Some(ModReflMode::Opaque);
                    }
                    Meta::Word(ref word) if word == "Deref" => {
                        this.mod_refl_mode = Some(ModReflMode::DelegateDeref(()));
                    }
                    x => panic!("invalid #[module_reflection(..)] attribute:\n{:?}", x),
                });
            } else if list.ident == "unconstrained" {
                with_nested_meta("unconstrained", list.nested, |attr| match attr {
                    Meta::Word(type_param)=>{
                        let unconstrained=UnconstrainedTyParam{
                            static_equivalent:None,
                        };
                        this.unconstrained_type_params.push((type_param,unconstrained));
                    }
                    Meta::List(type_param)=>{
                        let ty=type_param.ident;
                        let v=parse_unconstrained_ty_param(type_param.nested,arenas);
                        this.unconstrained_type_params.push((ty,v));
                    }
                    x => panic!(
                        "invalid #[unconstrained(..)] attribute\
                         (it must be the identifier of a type parameter):\n{:?}\n", 
                        x
                    ),
                })
            }else{
                panic!("Unrecodnized #[sabi(..)] attribute:\n{}",list.into_token_stream());
            }

        }
        (_,x) => panic!("not allowed inside the #[sabi(...)] attribute:\n{:?}", x),
    }
}


fn parse_refl_field<'a>(
    this: &mut StableAbiAttrs<'a>,
    field:&'a Field<'a>,
    list: Punctuated<NestedMeta, Comma>, 
    arenas: &'a Arenas
) {
    with_nested_meta("refl", list, |attr| match attr {
        Meta::NameValue(MetaNameValue{lit:Lit::Str(ref val),ref ident,..})=>{
            if ident=="pub_getter" {
                let function=arenas.alloc(val.value());
                this.override_field_accessor[field]=
                    Some(FieldAccessor::Method{ name:Some(function) });
            }else{
                panic!("invalid #[sabi(refl(..))] attribute:\n`{}={:?}`",ident,val)
            }
        }
        x => panic!("invalid #[sabi(refl(..))] attribute:\n{:?}", x),
    });
}



/// Parses the contents of #[sabi(missing_field( ... ))]
fn parse_missing_field<'a>(
    list: &Punctuated<NestedMeta, Comma>, 
    arenas: &'a Arenas
)->OnMissingField<'a>{
// `#[sabi(missing_field(panic))]`
// `#[sabi(missing_field(panic="somefunction"))]`
// `#[sabi(missing_field(option))]`
// `#[sabi(missing_field(with="somefunction"))]`
// `#[sabi(missing_field(default))]`
    let attribute_msg="

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


    let first_arg=list.into_iter().next();

    match first_arg {
        Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue{
            ident:ref nv_ident,
            lit:Lit::Str(ref str_),
            ..
        })))=>{
            if nv_ident=="with" {
                let function=str_.parse::<syn::Path>().unwrap()
                    .piped(|i| arenas.alloc(i) );
                OnMissingField::With{function}
            }else if nv_ident=="value" {
                let value=str_.parse::<syn::Expr>().unwrap()
                    .piped(|i| arenas.alloc(i) );
                OnMissingField::Value{value}
            }else{
                panic!(
                    "Invalid attribute:{}\n{}",
                    first_arg.into_token_stream(),
                    attribute_msg
                )
            }
        }
        Some(NestedMeta::Meta(Meta::Word(ref word)))if word=="option"=>{
            OnMissingField::ReturnOption
        }
        Some(NestedMeta::Meta(Meta::Word(ref word)))if word=="panic"=>{
            OnMissingField::Panic
        }
        Some(NestedMeta::Meta(Meta::Word(ref word)))if word=="default"=>{
            OnMissingField::Default_
        }
        Some(rem)=>panic!(
            "Invalid attribute:{}\n{}",
            rem.into_token_stream(),
            attribute_msg
        ),
        None=>panic!(
            "Error:Expected one attribute inside `missing_field(..)`\n{}",
            attribute_msg
        ),
    }
}


/// Parses the contents of #[sabi(kind(Prefix( ... )))]
fn parse_prefix_type_list<'a>(
    _name:&'a Ident,
    list: &Punctuated<NestedMeta, Comma>, 
    arenas: &'a Arenas
)->UncheckedPrefixKind<'a>{
    let mut iter=list.into_iter();

    let prefix_struct=match iter.next() {
        Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue{
            ident:ref nv_ident,
            lit:Lit::Str(ref type_str),
            ..
        })))
        if nv_ident=="prefix_struct" =>{
            let type_str=type_str.value();
            parse_str_as_ident(&type_str)
                .piped(|i| arenas.alloc(i) )
        }
        x => panic!(
            "invalid #[sabi(kind(Prefix(  )))] attribute\
             (it must be prefix_struct=\"NameOfPrefixStruct|default\" ):\n{:?}\n", 
            x
        )
    };
    
    UncheckedPrefixKind{
        prefix_struct,
    }
}


/// Parses the contents of #[sabi(kind(WithNonExhaustive( ... )))]
fn parse_non_exhaustive_list<'a>(
    _name:&'a Ident,
    list: &Punctuated<NestedMeta, Comma>, 
    arenas: &'a Arenas
)->UncheckedNonExhaustive<'a>{

    let trait_set=[
        "Clone","Display","Debug",
        "Eq","PartialEq","Ord","PartialOrd",
        "Hash","Deserialize","Serialize","Send","Sync","Error",
    ].iter()
     .map(|e| arenas.alloc(Ident::new(e,Span::call_site()) ) )
     .collect::<HashSet<&'a Ident>>();

    let mut this=UncheckedNonExhaustive::default();

    for elem in list {
        match elem {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue{ident,lit,..}))=>{
                match lit {
                    Lit::Str(str_lit)if ident=="align"=>{
                        let ty=arenas.alloc(parse_lit_as_type(str_lit));
                        this.alignment=Some(IntOrType::Type(ty));
                    }
                    Lit::Int(int_lit)if ident=="align"=>{
                        this.alignment=Some(IntOrType::Int(int_lit.value() as usize));
                    }
                    Lit::Str(str_lit)if ident=="size"=>{
                        let ty=arenas.alloc(parse_lit_as_type(str_lit));
                        this.size=Some(IntOrType::Type(ty));
                    }
                    Lit::Int(int_lit)if ident=="size"=>{
                        this.size=Some(IntOrType::Int(int_lit.value() as usize));
                    }
                    Lit::Str(str_lit)if ident=="assert_nonexhaustive"=>{
                        let ty=arenas.alloc(parse_lit_as_type(str_lit));
                        this.assert_nonexh.push(ty);
                    }
                    Lit::Str(str_lit)if ident=="interface"=>{
                        let ty=arenas.alloc(parse_lit_as_type(str_lit));
                        this.enum_interface=Some(EnumInterface::Old(ty));
                    }
                    x => panic!(
                        "invalid #[sabi(kind(WithNonExhaustive(  )))] attribute:\n{:?}\n", 
                        x
                    )
                }
            }
            NestedMeta::Meta(Meta::List(sublist))if sublist.ident=="assert_nonexhaustive" =>{
                for assertion in &sublist.nested {
                    match assertion {
                        NestedMeta::Literal(Lit::Str(str_lit))=>{
                            let ty=arenas.alloc(parse_lit_as_type(str_lit));
                            this.assert_nonexh.push(ty);
                        }
                        x => panic!(
                            "invalid #[sabi(kind(WithNonExhaustive(assert_nonexhaustive(  ))))] \
                             attribute:\n{:?}\n", 
                            x
                        )
                    }
                }
            }
            NestedMeta::Meta(Meta::List(sublist))if sublist.ident=="traits" =>{
                let enum_interface=match &mut this.enum_interface {
                    Some(EnumInterface::New(x))=>x,
                    Some(EnumInterface::Old{..})=>{
                        panic!("Cannot use both `interface=\"...\"` and `traits(...)`")
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
                        NestedMeta::Meta(Meta::Word(ident))=>{
                            (ident,true)
                        }
                        NestedMeta::Meta(Meta::NameValue(
                            MetaNameValue{ident,lit:Lit::Bool(bool_lit),..}
                        ))=>{
                            (ident,bool_lit.value)
                        }
                        x => panic!(
                            "invalid \
                             #[sabi(kind(WithNonExhaustive(traits(  ))))] attribute:\n{:?}\n", 
                            x
                        )
                    };

                    match trait_set.get(ident) {
                        Some(&trait_ident) => {
                            if is_impld {
                                &mut enum_interface.impld
                            }else{
                                &mut enum_interface.unimpld
                            }.push(trait_ident);
                        },
                        None =>panic!(
                            "invalid trait inside \
                             #[sabi(kind(WithNonExhaustive(traits(  ))))]:\n{:?}\n", 
                            ident
                        ),
                    }
                }
            }
            x => panic!(
                "invalid #[sabi(kind(WithNonExhaustive(  )))] attribute:\n{:?}\n", 
                x
            )
        }
    }

    this
}


/// Parses the contents of #[sabi(unconstrained(Type( ... )))]
fn parse_unconstrained_ty_param<'a>(
    list: Punctuated<NestedMeta, Comma>, 
    arenas: &'a Arenas
)-> UnconstrainedTyParam<'a> {
    match list.into_iter().next() {
        Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue{
            ident:ref nv_ident,
            lit:Lit::Str(ref type_str),
            ..
        })))
        if nv_ident=="StaticEquivalent" =>{
            let ty=match type_str.parse::<syn::Type>() {
                Ok(ty)=>ty,
                Err(e)=>panic!(
                    "Could not parse as a type:\n\t{}\nError:\n\t{}", 
                    type_str.value(),
                    e
                )
            };

            UnconstrainedTyParam{
                static_equivalent:Some(arenas.alloc(ty)),
            }
        }
        Some(x) => panic!(
            "invalid #[sabi(unconstrained(  )] attribute\
             (it must be StaticEquivalent=\"Type\" ):\n{:?}\n", 
            x
        ),
        None=>{
            UnconstrainedTyParam{
                static_equivalent:None,
            }
        }
    }
}


fn parse_sabi_override<'a>(_this: &mut StableAbiAttrs<'a>, _attr: Meta, _arenas: &'a Arenas) {}
