use syn::{
    Attribute, Ident, Meta, MetaList, MetaNameValue, NestedMeta, 
    Lit,WherePredicate,Type,
    punctuated::Punctuated,
    token::Comma,
};

use std::collections::{HashSet,HashMap};

use quote::ToTokens;

use crate::{
    datastructure::{DataStructure, DataVariant, Field},
    prefix_types::{PrefixKind,LastPrefixField,OnMissingField},
};
use crate::*;

pub(crate) struct StableAbiOptions<'a> {
    pub(crate) debug_print:bool,
    pub(crate) inside_abi_stable_crate:bool,
    pub(crate) kind: StabilityKind<'a>,
    pub(crate) repr: Repr,
    /// The type parameters that have the __StableAbi constraint
    pub(crate) stable_abi_bounded:HashSet<&'a Ident>,

    pub(crate) unconstrained_type_params:HashMap<Ident,UnconstrainedTyParam<'a>>,

    pub(crate) extra_bounds:Vec<WherePredicate>,

    /// A hashset of the fields whose contents are opaque 
    /// (there are still some minimal checks going on).
    pub(crate) opaque_fields:HashSet<*const Field<'a>>,

    pub(crate) renamed_fields:HashMap<*const Field<'a>,&'a Ident>,
    pub(crate) repr_attrs:Vec<MetaList>,
}


pub(crate) struct UnconstrainedTyParam<'a>{
    pub(crate) static_equivalent:Option<&'a Type>
}


pub(crate) enum StabilityKind<'a> {
    Value,
    Prefix(PrefixKind<'a>),
}


#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) enum Repr {
    Transparent,
    C,
}


impl<'a> StableAbiOptions<'a> {
    fn new(ds: &'a DataStructure<'a>, mut this: StableAbiAttrs<'a>) -> Self {
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
                    last_prefix_field:this.last_prefix_field,
                    first_suffix_field:this.last_prefix_field.map_or(0,|x|x.field_index+1),
                    prefix_struct:prefix.prefix_struct,
                    default_on_missing_fields:this.default_on_missing_fields,
                    on_missing_fields:this.on_missing_fields,
                })
            }

        };

        if repr == Repr::Transparent && ds.data_variant != DataVariant::Struct {
            panic!("\nAbiStable does not suport non-struct #[repr(transparent)] types.\n");
        }

        StableAbiOptions {
            debug_print:this.debug_print,
            inside_abi_stable_crate:this.inside_abi_stable_crate,
            kind, repr , stable_abi_bounded , 
            extra_bounds :this.extra_bounds,
            unconstrained_type_params:this.unconstrained_type_params.into_iter().collect(),
            opaque_fields:this.opaque_fields,
            renamed_fields:this.renamed_fields,
            repr_attrs:this.repr_attrs,
        }
    }
}

///////////////////////

#[derive(Default)]
struct StableAbiAttrs<'a> {
    debug_print:bool,
    inside_abi_stable_crate:bool,
    kind: UncheckedStabilityKind<'a>,
    repr: Option<Repr>,

    extra_bounds:Vec<WherePredicate>,

    /// The last field of the prefix of a Prefix-type.
    last_prefix_field:Option<LastPrefixField<'a>>,
    default_on_missing_fields:OnMissingField<'a>,
    on_missing_fields:HashMap<*const Field<'a>,OnMissingField<'a>>,

    /// The type parameters that have no constraints
    unconstrained_type_params:Vec<(Ident,UnconstrainedTyParam<'a>)>,

    // Using raw pointers to do an identity comparison.
    opaque_fields:HashSet<*const Field<'a>>,
    
    renamed_fields:HashMap<*const Field<'a>,&'a Ident>,
    repr_attrs:Vec<MetaList>,
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

    let name=ds.name;

    parse_inner(&mut this, attrs, ParseContext::TypeAttr{name}, arenas);

    for variant in &ds.variants {
        for (field_index,field) in variant.fields.iter().enumerate() {
            parse_inner(&mut this, field.attrs, ParseContext::Field{field,field_index}, arenas);
        }
    }

    StableAbiOptions::new(ds, this)
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
                this.opaque_fields.insert(field);
            }else if word == "last_prefix_field" {
                this.last_prefix_field=Some(LastPrefixField{field_index,field});
            }else{
                panic!("unrecognized field attribute `#[sabi({})]` ",word);
            }
        }
        (
            ParseContext::Field{field,field_index}, 
            Meta::NameValue(MetaNameValue{lit:Lit::Str(ref value),ref ident,..})
        ) => {
            if ident=="rename" {
                let renamed=parse_lit_as_ident(&value)
                    .piped(|x| arenas.alloc(x) );
                this.renamed_fields.insert(field,renamed);
            }
        }
        (ParseContext::Field{field,..}, Meta::List(list)) => {
            if list.ident == "missing_field" {
                let on_missing_field=parse_missing_field(&list.nested,arenas);
                this.on_missing_fields.insert(field,on_missing_field);
            }else{
                panic!("unrecognized field attribute `#[sabi({})]` ",list.ident);
            }
        }
        (ParseContext::TypeAttr{..},Meta::Word(ref word)) if word == "inside_abi_stable_crate" =>{
            this.inside_abi_stable_crate=true;
        }
        (ParseContext::TypeAttr{..},Meta::Word(ref word)) if word == "debug_print" =>{
            this.debug_print=true;
        }
        (
            ParseContext::TypeAttr{..},
            Meta::NameValue(MetaNameValue{lit:Lit::Str(ref unparsed_bound),ref ident,..})
        )if ident=="bound" =>
        {
            let bound=match unparsed_bound.parse::<WherePredicate>() {
                Ok(v)=>v,
                Err(e)=>panic!(
                    "\n\nInvalid bound:\n\t{}\nError:\n\t{}\n\n",
                    unparsed_bound.value(),
                    e
                ),
            };
            this.extra_bounds.push(bound);
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
    This always panics if the field doesn't exist.

    `#[sabi(missing_field(panic=\"somefunction\"))]`
    This panics,passing a metadata struct in so that it can have an 
    informative error message.

    `#[sabi(missing_field(option))]`
    This returns None if the field doesn't exist.
    This is the default.

    `#[sabi(missing_field(with=\"somefunction\"))]`
    This calls the function `somefunction` if the field doesn't exist.
    For panicking use `#[sabi(missing_field(panic=\"somefunction\"))]` instead.

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
            if nv_ident=="panic"{
                OnMissingField::PanicWith{function}
            }else if nv_ident=="with" {
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
    name:&'a Ident,
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
            if type_str=="default" {
                let ident=format!("{}_Prefix",type_str);
                syn::parse_str::<Ident>(&ident).unwrap()
            }else{
                parse_str_as_ident(&type_str)
            }.piped(|i| arenas.alloc(i) )
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
    match lit.parse::<syn::Ident>() {
        Ok(ident)=>ident,
        Err(e)=>panic!(
            "Could not parse as an identifier:\n\t{}\nError:\n\t{}", 
            lit.value(),
            e
        )
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
