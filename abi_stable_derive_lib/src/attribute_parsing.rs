use syn::{
    Attribute, Ident, Meta, MetaList, MetaNameValue, NestedMeta, 
    Lit,WherePredicate,
};

use hashbrown::HashSet;

use quote::ToTokens;

use crate::datastructure::{DataStructure, DataVariant, Field};
use crate::*;

pub(crate) struct StableAbiOptions<'a> {
    pub(crate) debug_print:bool,
    pub(crate) inside_abi_stable_crate:bool,
    pub(crate) kind: StabilityKind,
    pub(crate) repr: Repr,
    /// The type parameters that have the __StableAbi constraint
    pub(crate) stable_abi_bounded:HashSet<&'a Ident>,

    pub(crate) unconstrained_type_params:HashSet<Ident>,

    pub(crate) extra_bounds:Vec<WherePredicate>,

    /// A hashset of the fields whose contents are opaque 
    /// (there are still some minimal checks going on).
    pub(crate) opaque_fields:HashSet<*const Field<'a>>,
}

pub(crate) enum StabilityKind {
    Value,
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
             ",
        );

        let mut stable_abi_bounded=ds.generics
            .type_params()
            .map(|x| &x.ident )
            .collect::<HashSet<&'a Ident>>();

        for type_param in &this.unconstrained_type_params {
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
                    &format!("({}):__StableAbi",(&field.ty).into_token_stream())
                ).expect(concat!(file!(),"-",line!()));
                this.extra_bounds.push(field_bound);

                StabilityKind::Value
            }
            None | Some(UncheckedStabilityKind::Value) => StabilityKind::Value,
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
        }
    }
}

///////////////////////

#[derive(Default)]
struct StableAbiAttrs<'a> {
    debug_print:bool,
    inside_abi_stable_crate:bool,
    kind: Option<UncheckedStabilityKind>,
    repr: Option<Repr>,

    extra_bounds:Vec<WherePredicate>,

    /// The type parameters that have no constraints
    unconstrained_type_params:Vec<Ident>,

    pointer_field: Option<&'a Field<'a>>,

    // Using raw pointers to do an identity comparison.
    opaque_fields:HashSet<*const Field<'a>>,
}

#[derive(Copy, Clone)]
enum UncheckedStabilityKind {
    Value,
}

#[derive(Copy, Clone)]
enum ParseContext<'a> {
    TypeAttr,
    Field(&'a Field<'a>),
}

pub(crate) fn parse_attrs_for_stable_abi<'a>(
    attrs: &'a [Attribute],
    ds: &'a DataStructure<'a>,
    arenas: &'a Arenas,
) -> StableAbiOptions<'a> {
    let mut this = StableAbiAttrs::default();

    parse_inner(&mut this, attrs, ParseContext::TypeAttr, arenas);

    for variant in &ds.variants {
        for field in &variant.fields {
            parse_inner(&mut this, field.attrs, ParseContext::Field(field), arenas);
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

fn parse_attr_list<'a>(this: &mut StableAbiAttrs<'a>,pctx: ParseContext<'a>, list: MetaList, arenas: &'a Arenas) {
    if list.ident == "repr" {
        for repr in list.nested {
            match repr {
                NestedMeta::Meta(Meta::Word(ref ident)) if ident == "C" => {
                    this.repr = Some(Repr::C);
                }
                NestedMeta::Meta(Meta::Word(ref ident)) if ident == "transparent" => {
                    this.repr = Some(Repr::Transparent);
                }
                NestedMeta::Meta(Meta::List(ref list)) if list.ident == "align" => {
                    
                }
                x => panic!(
                    "repr attribute not currently recognized by this macro:\n{:?}",
                    x
                ),
            }
        }
    } else if list.ident == "sabi" {
        with_nested_meta("sabi", list.nested, |attr| {
            parse_sabi_attr(this,pctx, attr, arenas)
        });
    }
}

fn parse_sabi_attr<'a>(this: &mut StableAbiAttrs<'a>,pctx: ParseContext<'a>, attr: Meta, arenas: &'a Arenas) {
    match (pctx, attr) {
        (ParseContext::Field(field), Meta::Word(word)) => {
            if word == "pointer" {
                this.pointer_field = Some(field);
            }else if word == "unsafe_opaque_field" {
                this.opaque_fields.insert(field);
            }else{
                panic!("unrecognized field attribute `#[sabi({})]` ",word);
            }
        }
        (ParseContext::TypeAttr,Meta::Word(ref word)) if word == "inside_abi_stable_crate" =>{
            this.inside_abi_stable_crate=true;
        }
        (ParseContext::TypeAttr,Meta::Word(ref word)) if word == "debug_print" =>{
            this.debug_print=true;
        }
        (
            ParseContext::TypeAttr,
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
        (ParseContext::TypeAttr,Meta::List(list)) => {
            if list.ident == "override" {
                with_nested_meta("override", list.nested, |attr| {
                    parse_sabi_override(this, attr, arenas)
                });
            } else if list.ident == "kind" {
                with_nested_meta("kind", list.nested, |attr| match attr {
                    Meta::Word(ref word) if word == "Value" => {
                        this.kind = Some(UncheckedStabilityKind::Value);
                    }
                    x => panic!("invalid #[kind(..)] attribute:\n{:?}", x),
                });
            } else if list.ident == "phantom" {
                with_nested_meta("phantom", list.nested, |attr| match attr {
                    Meta::Word(type_param)=>{
                        this.unconstrained_type_params.push(type_param);
                    }
                    x => panic!(
                        "invalid #[phantom(..)] attribute\
                         (it must be the identifier of a type parameter):\n{:?}\n", 
                        x
                    ),
                })
            } 

        }
        (_,x) => panic!("not allowed inside the #[sabi(...)] attribute:\n{:?}", x),
    }
}

fn parse_sabi_override<'a>(_this: &mut StableAbiAttrs<'a>, _attr: Meta, _arenas: &'a Arenas) {}

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
