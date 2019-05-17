use syn::{
    Attribute, Ident, Meta, MetaList, MetaNameValue, NestedMeta, 
    Lit,WherePredicate,Type,
    punctuated::Punctuated,
    token::Comma,
};

use std::{
    collections::{HashSet,HashMap},
    mem,
};

use quote::ToTokens;

use core_extensions::IteratorExt;

use crate::{
    reflection::{ModReflMode,FieldAccessor},
    datastructure::{DataStructure, DataVariant, Field,FieldMap},
    prefix_types::{PrefixKind,FirstSuffixField,OnMissingField,AccessorOrMaybe,PrefixKindField},
};
use crate::*;

pub(crate) struct StableAbiOptions<'a> {
    pub(crate) debug_print:bool,
    pub(crate) kind: StabilityKind<'a>,
    pub(crate) repr: Repr,
    /// The type parameters that have the __StableAbi constraint
    pub(crate) stable_abi_bounded:HashSet<&'a Ident>,

    pub(crate) unconstrained_type_params:HashMap<Ident,UnconstrainedTyParam<'a>>,

    pub(crate) extra_bounds:Vec<WherePredicate>,

    pub(crate) tags:Option<syn::Expr>,

    /// A hashset of the fields whose contents are opaque 
    /// (there are still some minimal checks going on).
    pub(crate) opaque_fields:FieldMap<bool>,

    pub(crate) override_field_accessor:FieldMap<Option<FieldAccessor<'a>>>,
    
    pub(crate) renamed_fields:FieldMap<Option<&'a Ident>>,

    #[allow(dead_code)]
    pub(crate) repr_attrs:Vec<MetaList>,

    pub(crate) mod_refl_mode:ModReflMode<usize>,

    pub(crate) phantom_fields:Vec<(&'a str,&'a Type)>,

}


pub(crate) struct UnconstrainedTyParam<'a>{
    pub(crate) static_equivalent:Option<&'a Type>
}


pub(crate) enum StabilityKind<'a> {
    Value,
    Prefix(PrefixKind<'a>),
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
            (true,StabilityKind::Value)=>
                FieldAccessor::Direct,
            (true,StabilityKind::Prefix(prefix))=>
                prefix.field_accessor(field),
        }
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) enum Repr {
    Transparent,
    C,
}


impl<'a> StableAbiOptions<'a> {
    fn new(
        ds: &'a DataStructure<'a>, 
        mut this: StableAbiAttrs<'a>,
        arenas: &'a Arenas,
    ) -> Self {
        let mut phantom_fields=Vec::<(&'a str,&'a Type)>::new();

        let repr = this.repr.expect(
            "\n\
             the #[repr(..)] attribute must be one of the supported attributes:\n\
             \t- #[repr(C)]\n\
             \t- #[repr(transparent)]\n\
             \t- #[repr(align(<some_integer>))]\n\
             ",
        );

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
            _ if repr == Repr::Transparent => {
                let field=&ds.variants[0].fields[0];
                
                let field_bound=syn::parse_str::<WherePredicate>(
                    &format!("({}):__StableAbi",(&field.mutated_ty).into_token_stream())
                ).expect(concat!(file!(),"-",line!()));
                this.extra_bounds.push(field_bound);

                StabilityKind::Value
            }
            UncheckedStabilityKind::Value => StabilityKind::Value,
            UncheckedStabilityKind::Prefix(prefix)=>{
                StabilityKind::Prefix(PrefixKind{
                    first_suffix_field:this.first_suffix_field,
                    prefix_struct:prefix.prefix_struct,
                    default_on_missing_fields:this.default_on_missing_fields,
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
                })
            }

        };

        if repr == Repr::Transparent && ds.data_variant != DataVariant::Struct {
            panic!("\nAbiStable does not suport non-struct #[repr(transparent)] types.\n");
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

        StableAbiOptions {
            debug_print:this.debug_print,
            kind, repr , stable_abi_bounded , 
            extra_bounds :this.extra_bounds,
            unconstrained_type_params:this.unconstrained_type_params.into_iter().collect(),
            opaque_fields:this.opaque_fields,
            renamed_fields:this.renamed_fields,
            override_field_accessor:this.override_field_accessor,
            repr_attrs:this.repr_attrs,
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
    repr: Option<Repr>,

    extra_bounds:Vec<WherePredicate>,

    tags:Option<syn::Expr>,


    /// The last field of the prefix of a Prefix-type.
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
    repr_attrs:Vec<MetaList>,

    mod_refl_mode:Option<ModReflMode<()>>,
}


#[derive(Copy, Clone)]
enum UncheckedStabilityKind<'a> {
    Value,
    Prefix(UncheckedPrefixKind<'a>),
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

pub(crate) fn parse_attrs_for_stable_abi<'a>(
    attrs: &'a [Attribute],
    ds: &'a DataStructure<'a>,
    arenas: &'a Arenas,
) -> StableAbiOptions<'a> {
    let mut this = StableAbiAttrs::default();

    this.opaque_fields=FieldMap::defaulted(ds);
    this.prefix_kind_fields=FieldMap::defaulted(ds);
    this.renamed_fields=FieldMap::defaulted(ds);
    this.override_field_accessor=FieldMap::defaulted(ds);

    let name=ds.name;

    parse_inner(&mut this, attrs, ParseContext::TypeAttr{name}, arenas);

    for variant in &ds.variants {
        for (field_index,field) in variant.fields.iter().enumerate() {
            parse_inner(&mut this, field.attrs, ParseContext::Field{field,field_index}, arenas);
        }
    }

    StableAbiOptions::new(ds, this,arenas)
}

fn parse_inner<'a>(
    this: &mut StableAbiAttrs<'a>,
    attrs: &'a [Attribute],
    pctx: ParseContext<'a>,
    arenas: &'a Arenas,
) {
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
        for repr in &list.nested {
            match &repr {
                NestedMeta::Meta(Meta::Word(ident)) if ident == "C" => {
                    this.repr = Some(Repr::C);
                }
                NestedMeta::Meta(Meta::Word(ident)) if ident == "transparent" => {
                    this.repr = Some(Repr::Transparent);
                }
                NestedMeta::Meta(Meta::List(list)) if list.ident == "align" => {
                    
                }
                x => panic!(
                    "repr attribute not currently recognized by this macro:\n{:?}",
                    x
                ),
            }
        }
        this.repr_attrs.push(list);
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
            }else if ident=="accessible_if" {
                let expr=arenas.alloc(parse_lit_as_expr(&value));
                this.prefix_kind_fields[field].accessible_if=Some(expr);
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
    use syn::{MetaNameValue as MNV};

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
    
    `#[sabi(missing_field(default))]`
    This returns `Default::default` if the field doesn't exist.

";


    let first_arg=list.into_iter().next();

    match first_arg {
        Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue{
            ident:ref nv_ident,
            lit:Lit::Str(ref type_str),
            ..
        })))=>{
            let function=type_str.parse::<syn::Path>().unwrap()
                .piped(|i| arenas.alloc(i) );
            if nv_ident=="with" {
                OnMissingField::With{function}
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

////////////////////////////////////////////////////////////////////


fn parse_str_as_ident(lit:&str)->syn::Ident{
    match syn::parse_str::<syn::Ident>(lit) {
        Ok(ident)=>ident,
        Err(e)=>panic!(
            "Could not parse as an identifier:\n\t{}\nError:\n\t{}", 
            lit,
            e
        )
    }
}

fn parse_lit_as_ident(lit:&syn::LitStr)->syn::Ident{
    parse_str_lit_as(lit,"Could not parse as an identifier")
}

fn parse_lit_as_expr(lit:&syn::LitStr)->syn::Expr{
    parse_str_lit_as(lit,"Could not parse as an expression")
}


fn parse_str_lit_as<P>(lit:&syn::LitStr,err_description:&str)->P
where P:syn::parse::Parse
{
    match lit.parse::<P>() {
        Ok(x)=>x,
        Err(e)=>panic!("{}:\n\t{}\nError:\n\t{}", err_description,lit.value(),e)
    }
}


////////////////////////////////////////////////////////////////////

/// Iterates over an iterator of syn::NestedMeta,
/// unwrapping it into a syn::Meta and passing it into the `f` closure.
fn with_nested_meta<I, F>(attr_name: &str, iter: I, mut f: F)
where
    F: FnMut(Meta),
    I: IntoIterator<Item = NestedMeta>,
{
    for repr in iter {
        match repr {
            NestedMeta::Meta(attr) => {
                f(attr);
            }
            NestedMeta::Literal(lit) => {
                panic!(
                    "\
                     the #[{}(...)] attribute does not allow \
                     literals in the attribute list:\n{:?}\
                     ",
                    attr_name, lit
                );
            }
        }
    }
}
