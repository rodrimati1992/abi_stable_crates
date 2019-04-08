use crate::*;
use crate::{
    fn_pointer_extractor::{FnInfo, TypeVisitor},
    lifetimes::LifetimeIndex,
};

use std::fmt::Write;
use std::{cmp, hash};

use arrayvec::ArrayString;

use syn::{
    self, Attribute, Data, DeriveInput, Field as SynField, Fields as SynFields, Generics, Ident,
    Type, Visibility,
};

use quote::ToTokens;

use proc_macro2::{Span, TokenStream};

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct DataStructure<'a> {
    pub(crate) vis: &'a Visibility,
    pub(crate) name: &'a Ident,
    pub(crate) generics: &'a Generics,
    pub(crate) lifetime_count: usize,

    pub(crate) fn_info: FnInfo<'a>,

    pub(crate) attrs: &'a [Attribute],

    pub(crate) data_variant: DataVariant,
    pub(crate) enum_: Option<Enum<'a>>,
    pub(crate) variants: Vec<Struct<'a>>,
}

#[derive(Clone, Debug)]
pub(crate) struct Enum<'a> {
    pub(crate) name: &'a Ident,
    pub(crate) path: TokenStream,
}

impl<'a> cmp::PartialEq for Enum<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> hash::Hash for Enum<'a> {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: hash::Hasher,
    {
        self.name.hash(hasher);
    }
}

impl<'a> DataStructure<'a> {
    pub(crate) fn new(ast: &'a mut DeriveInput, arenas: &'a Arenas, ctokens: &'a CommonTokens<'a>) -> Self {
        let name = &ast.ident;
        let enum_ = match ast.data {
            Data::Enum(_) => Some(Enum {
                name,
                path: quote! { #name:: },
            }),
            _ => None,
        };

        let data_variant: DataVariant;

        let mut ty_visitor = TypeVisitor::new(arenas, ctokens, &ast.generics);

        let mut variants = Vec::new();

        match &mut ast.data {
            Data::Enum(enum_) => {
                for var in &mut enum_.variants {
                    variants.push(Struct::new(
                        &var.attrs,
                        &var.ident,
                        &mut var.fields,
                        &mut ty_visitor,
                    ));
                }
                data_variant = DataVariant::Enum;
            }
            Data::Struct(struct_) => {
                variants.push(Struct::new(
                    &ast.attrs,
                    name,
                    &mut struct_.fields,
                    &mut ty_visitor,
                ));
                data_variant = DataVariant::Struct;
            }

            Data::Union(union_) => {
                let fields = Some(&mut union_.fields.named);
                let sk = StructKind::Braced;
                let vari = Struct::with_fields(&ast.attrs, name, sk, fields, &mut ty_visitor);
                variants.push(vari);
                data_variant = DataVariant::Union;
            }
        }

        Self {
            vis: &ast.vis,
            name,
            fn_info: ty_visitor.into_fn_info(),
            attrs: &ast.attrs,
            generics: &ast.generics,
            lifetime_count:ast.generics.lifetimes().count(),
            data_variant,
            enum_,
            variants,
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Whether the struct is tupled or not.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) enum StructKind {
    /// structs declared using the `struct Name( ... ) syntax.
    Tuple,
    /// structs declared using the `struct Name{ ... }` or `struct name;` syntaxes
    Braced,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) enum DataVariant {
    Struct,
    Enum,
    Union,
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct Struct<'a> {
    pub(crate) attrs: &'a [Attribute],
    pub(crate) name: &'a Ident,
    pub(crate) kind: StructKind,
    pub(crate) fields: Vec<Field<'a>>,
    _priv: (),
}

impl<'a> Struct<'a> {
    pub(crate) fn new(
        attrs: &'a [Attribute],
        name: &'a Ident,
        fields: &'a mut SynFields,
        tv: &mut TypeVisitor<'a>,
    ) -> Self {
        let kind = match *fields {
            SynFields::Named { .. } => StructKind::Braced,
            SynFields::Unnamed { .. } => StructKind::Tuple,
            SynFields::Unit { .. } => StructKind::Braced,
        };
        let fields = match fields {
            SynFields::Named(f) => Some(&mut f.named),
            SynFields::Unnamed(f) => Some(&mut f.unnamed),
            SynFields::Unit => None,
        };

        Self::with_fields(attrs, name, kind, fields, tv)
    }

    pub(crate) fn with_fields<I>(
        attrs: &'a [Attribute],
        name: &'a Ident,
        kind: StructKind,
        fields: Option<I>,
        tv: &mut TypeVisitor<'a>,
    ) -> Self
    where
        I: IntoIterator<Item = &'a mut SynField>,
    {
        Self {
            attrs,
            name,
            kind,
            fields: fields.map_or(Vec::new(), |x| Field::from_iter(name, x, tv)),
            _priv: (),
        }
    }

    }

//////////////////////////////////////////////////////////////////////////////

/// Represent a struct field
///
#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct Field<'a> {
    pub(crate) attrs: &'a [Attribute],
    pub(crate) vis: &'a Visibility,
    pub(crate) referenced_lifetimes: Vec<LifetimeIndex>,
    /// identifier for the field,which is either an index(in a tuple struct) or a name.
    pub(crate) ident: FieldIdent<'a>,
    pub(crate) ty: &'a Type,
}

impl<'a> Field<'a> {
    pub(crate) fn new(
        index: usize,
        field: &'a mut SynField,
        span: Span,
        tv: &mut TypeVisitor<'a>,
    ) -> Self {
        let ident = match field.ident.as_ref() {
            Some(ident) => FieldIdent::Named(ident),
            None => FieldIdent::new_index(index, span),
        };

        let visit_info = tv.visit_field(&mut field.ty);

        Self {
            attrs: &field.attrs,
            vis: &field.vis,
            referenced_lifetimes: visit_info.referenced_lifetimes,
            ident,
            ty: &field.ty,
        }
    }

    pub(crate) fn ident(&self)->&Ident{
        match &self.ident {
            FieldIdent::Index(_,ident)=>ident,
            FieldIdent::Named(ident)=>ident,
        }
    }

    pub(crate) fn from_iter<I>(name: &'a Ident, fields: I, tv: &mut TypeVisitor<'a>) -> Vec<Self>
    where
        I: IntoIterator<Item = &'a mut SynField>,
    {
        fields
            .into_iter()
            .enumerate()
            .map(|(i, f)| Field::new(i, f, name.span(), tv))
            .collect()
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) enum FieldIdent<'a> {
    Index(usize, Ident),
    Named(&'a Ident),
}

impl<'a> ToTokens for FieldIdent<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match *self {
            FieldIdent::Index(ind, ..) => syn::Index::from(ind).to_tokens(tokens),
            FieldIdent::Named(name) => name.to_tokens(tokens),
        }
    }
}

impl<'a> FieldIdent<'a> {
    fn new_index(index: usize, span: Span) -> Self {
        let mut buff = ArrayString::<[u8; 16]>::new();
        let _ = write!(buff, "field_{}", index);
        FieldIdent::Index(index, Ident::new(&buff, span))
    }
}
