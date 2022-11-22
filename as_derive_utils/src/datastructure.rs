use syn::{
    self, Attribute, Data, DeriveInput, Field as SynField, Fields as SynFields, Generics, Ident,
    Type, Visibility,
};

use quote::ToTokens;

use proc_macro2::{Span, TokenStream};

use std::fmt::{self, Display};

mod field_map;
mod type_param_map;

pub use self::{field_map::FieldMap, type_param_map::TypeParamMap};

//////////////////////////////////////////////////////////////////////////////

/// A type definition(enum,struct,union).
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct DataStructure<'a> {
    pub vis: &'a Visibility,
    pub name: &'a Ident,
    pub generics: &'a Generics,
    pub lifetime_count: usize,
    pub field_count: usize,
    pub pub_field_count: usize,

    pub attrs: &'a [Attribute],

    /// Whether this is a struct/union/enum.
    pub data_variant: DataVariant,

    /// The variants in the type definition.
    ///
    /// If it is a struct or a union this only has 1 element.
    pub variants: Vec<Struct<'a>>,
}

impl<'a> DataStructure<'a> {
    pub fn new(ast: &'a DeriveInput) -> Self {
        let name = &ast.ident;

        let data_variant: DataVariant;

        let mut variants = Vec::new();

        match &ast.data {
            Data::Enum(enum_) => {
                let override_vis = Some(&ast.vis);

                for (variant, var) in enum_.variants.iter().enumerate() {
                    variants.push(Struct::new(
                        StructParams {
                            discriminant: var.discriminant.as_ref().map(|(_, v)| v),
                            variant,
                            attrs: &var.attrs,
                            name: &var.ident,
                            override_vis,
                        },
                        &var.fields,
                    ));
                }
                data_variant = DataVariant::Enum;
            }
            Data::Struct(struct_) => {
                let override_vis = None;

                variants.push(Struct::new(
                    StructParams {
                        discriminant: None,
                        variant: 0,
                        attrs: &[],
                        name,
                        override_vis,
                    },
                    &struct_.fields,
                ));
                data_variant = DataVariant::Struct;
            }

            Data::Union(union_) => {
                let override_vis = None;

                let fields = Some(&union_.fields.named);
                let sk = StructKind::Braced;
                let vari = Struct::with_fields(
                    StructParams {
                        discriminant: None,
                        variant: 0,
                        attrs: &[],
                        name,
                        override_vis,
                    },
                    sk,
                    fields,
                );
                variants.push(vari);
                data_variant = DataVariant::Union;
            }
        }

        let mut field_count = 0;
        let mut pub_field_count = 0;

        for vari in &variants {
            field_count += vari.fields.len();
            pub_field_count += vari.pub_field_count;
        }

        Self {
            vis: &ast.vis,
            name,
            attrs: &ast.attrs,
            generics: &ast.generics,
            lifetime_count: ast.generics.lifetimes().count(),
            data_variant,
            variants,
            field_count,
            pub_field_count,
        }
    }

    pub fn has_public_fields(&self) -> bool {
        self.pub_field_count != 0
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Whether the struct is tupled or not.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum StructKind {
    /// structs declared using the `struct Name( ... ) syntax.
    Tuple,
    /// structs declared using the `struct Name{ ... }` or `struct name;` syntaxes
    Braced,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum DataVariant {
    Struct,
    Enum,
    Union,
}

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub struct FieldIndex {
    pub variant: usize,
    pub pos: usize,
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
struct StructParams<'a> {
    discriminant: Option<&'a syn::Expr>,
    variant: usize,
    attrs: &'a [Attribute],
    name: &'a Ident,
    override_vis: Option<&'a Visibility>,
}

/// A struct/union or a variant of an enum.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Struct<'a> {
    /// The attributes of this `Struct`.
    ///
    /// If this is a struct/union:these is the same as DataStructure.attrs.
    ///
    /// If this is an enum:these are the attributes on the variant.
    pub attrs: &'a [Attribute],
    /// The name of this `Struct`.
    ///
    /// If this is a struct/union:these is the same as DataStructure.name.
    ///
    /// If this is an enum:this is the name of the variant.
    pub name: &'a Ident,
    pub kind: StructKind,
    pub fields: Vec<Field<'a>>,
    pub pub_field_count: usize,
    /// The value of this discriminant.
    ///
    /// If this is a Some(_):This is an enum with an explicit discriminant value.
    ///
    /// If this is an None:
    ///     This is either a struct/union or an enum variant without an explicit discriminant.
    pub discriminant: Option<&'a syn::Expr>,
}

impl<'a> Struct<'a> {
    fn new(p: StructParams<'a>, fields: &'a SynFields) -> Self {
        let kind = match *fields {
            SynFields::Named { .. } => StructKind::Braced,
            SynFields::Unnamed { .. } => StructKind::Tuple,
            SynFields::Unit { .. } => StructKind::Braced,
        };
        let fields = match fields {
            SynFields::Named(f) => Some(&f.named),
            SynFields::Unnamed(f) => Some(&f.unnamed),
            SynFields::Unit => None,
        };

        Self::with_fields(p, kind, fields)
    }

    fn with_fields<I>(p: StructParams<'a>, kind: StructKind, fields: Option<I>) -> Self
    where
        I: IntoIterator<Item = &'a SynField>,
    {
        let fields = match fields {
            Some(x) => Field::from_iter(p, x),
            None => Vec::new(),
        };

        let mut pub_field_count = 0usize;

        for field in &fields {
            if field.is_public() {
                pub_field_count += 1;
            }
        }

        Self {
            discriminant: p.discriminant,
            attrs: p.attrs,
            name: p.name,
            kind,
            pub_field_count,
            fields,
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Represent a struct field
///
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Field<'a> {
    pub index: FieldIndex,
    pub attrs: &'a [Attribute],
    pub vis: &'a Visibility,
    /// identifier for the field,which is either an index(in a tuple struct) or a name.
    pub ident: FieldIdent<'a>,
    pub ty: &'a Type,
}

impl<'a> Field<'a> {
    fn new(
        index: FieldIndex,
        field: &'a SynField,
        span: Span,
        override_vis: Option<&'a Visibility>,
    ) -> Self {
        let ident = match field.ident.as_ref() {
            Some(ident) => FieldIdent::Named(ident),
            None => FieldIdent::new_index(index.pos, span),
        };

        Self {
            index,
            attrs: &field.attrs,
            vis: override_vis.unwrap_or(&field.vis),
            ident,
            ty: &field.ty,
        }
    }

    pub fn is_public(&self) -> bool {
        matches!(self.vis, Visibility::Public { .. })
    }

    /// Gets the identifier of this field usable for the variable in a pattern.
    ///
    /// You can match on a single field struct (tupled or braced) like this:
    ///
    /// ```rust
    /// use as_derive_utils::datastructure::Struct;
    ///
    /// fn example(struct_: Struct<'_>) -> proc_macro2::TokenStream {
    ///     let field = &struct_.field[0];
    ///     let field_name = &field.ident;
    ///     let variable = field.pat_ident();
    ///    
    ///     quote::quote!( let Foo{#field_name: #variable} = bar; )
    /// }
    /// ```
    pub fn pat_ident(&self) -> &Ident {
        match &self.ident {
            FieldIdent::Index(_, ident) => ident,
            FieldIdent::Named(ident) => ident,
        }
    }

    fn from_iter<I>(p: StructParams<'a>, fields: I) -> Vec<Self>
    where
        I: IntoIterator<Item = &'a SynField>,
    {
        fields
            .into_iter()
            .enumerate()
            .map(|(pos, f)| {
                let fi = FieldIndex {
                    variant: p.variant,
                    pos,
                };
                Field::new(fi, f, p.name.span(), p.override_vis)
            })
            .collect()
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum FieldIdent<'a> {
    Index(usize, Ident),
    Named(&'a Ident),
}

impl<'a> Display for FieldIdent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldIdent::Index(x, ..) => Display::fmt(x, f),
            FieldIdent::Named(x) => Display::fmt(x, f),
        }
    }
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
        FieldIdent::Index(index, Ident::new(&format!("field_{}", index), span))
    }
}
