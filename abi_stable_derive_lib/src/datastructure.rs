use crate::*;
use crate::{
    fn_pointer_extractor::{FnInfo,Function, TypeVisitor},
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

mod field_map;

pub(crate) use self::field_map::FieldMap;

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct DataStructure<'a> {
    pub(crate) vis: &'a Visibility,
    pub(crate) name: &'a Ident,
    pub(crate) generics: &'a Generics,
    pub(crate) lifetime_count: usize,
    pub(crate) field_count: usize,
    pub(crate) pub_field_count: usize,
    pub(crate) fn_ptr_count:usize,

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
    pub(crate) fn new(
        ast: &'a mut DeriveInput, 
        arenas: &'a Arenas, 
        ctokens: &'a CommonTokens<'a>
    ) -> Self {
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
                let override_vis=Some(&ast.vis);

                for (variant,var) in (&mut enum_.variants).into_iter().enumerate() {
                    variants.push(Struct::new(
                        StructParams{
                            discriminant:var.discriminant
                                .as_ref()
                                .map(|(_,v)| v ),
                            variant:variant,
                            attrs:&var.attrs,
                            name:&var.ident,
                            override_vis:override_vis,
                        },
                        &mut var.fields,
                        &mut ty_visitor,
                    ));
                }
                data_variant = DataVariant::Enum;
            }
            Data::Struct(struct_) => {
                let override_vis=None;

                variants.push(Struct::new(
                    StructParams{
                        discriminant:None,
                        variant:0,
                        attrs:&ast.attrs,
                        name:name,
                        override_vis:override_vis,
                    },
                    &mut struct_.fields,
                    &mut ty_visitor,
                ));
                data_variant = DataVariant::Struct;
            }

            Data::Union(union_) => {
                let override_vis=None;

                let fields = Some(&union_.fields.named);
                let sk = StructKind::Braced;
                let vari = Struct::with_fields(
                    StructParams{
                        discriminant:None,
                        variant:0,
                        attrs:&ast.attrs, 
                        name:name, 
                        override_vis:override_vis,
                    },
                    sk, 
                    fields,
                    &mut ty_visitor
                );
                variants.push(vari);
                data_variant = DataVariant::Union;
            }
        }

        let mut field_count=0;
        let mut pub_field_count=0;
        let mut fn_ptr_count=0;

        for vari in &variants {
            field_count+=vari.fields.len();
            pub_field_count+=vari.pub_field_count;
            fn_ptr_count+=vari.fn_ptr_count;
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
            field_count,
            pub_field_count,
            fn_ptr_count,
        }
    }

    pub(crate) fn has_public_fields(&self)->bool{
        self.pub_field_count!=0
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


#[derive(Copy,Clone, Debug, PartialEq, Hash)]
pub(crate) struct FieldIndex {
    pub(crate) variant:usize,
    pub(crate) pos:usize,
}

//////////////////////////////////////////////////////////////////////////////


#[derive(Copy,Clone)]
pub(crate) struct StructParams<'a>{
    pub(crate) discriminant:Option<&'a syn::Expr>,
    pub(crate) variant:usize,
    pub(crate) attrs: &'a [Attribute],
    pub(crate) name: &'a Ident,
    pub(crate) override_vis:Option<&'a Visibility>,
}


#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct Struct<'a> {
    pub(crate) attrs: &'a [Attribute],
    pub(crate) name: &'a Ident,
    pub(crate) kind: StructKind,
    pub(crate) fields: Vec<Field<'a>>,
    pub(crate) pub_field_count:usize,
    pub(crate) fn_ptr_count:usize,
    pub(crate) discriminant:Option<&'a syn::Expr>,
    _priv: (),
}

impl<'a> Struct<'a> {
    pub(crate) fn new(
        p:StructParams<'a>,
        fields: &'a SynFields,
        tv: &mut TypeVisitor<'a>,
    ) -> Self {
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

        Self::with_fields(p, kind, fields, tv)
    }

    pub(crate) fn with_fields<I>(
        p:StructParams<'a>,
        kind: StructKind,
        fields: Option<I>,
        tv: &mut TypeVisitor<'a>,
    ) -> Self
    where
        I: IntoIterator<Item = &'a SynField>,
    {
        let fields=match fields {
            Some(x) => Field::from_iter(p, x, tv),
            None => Vec::new(),
        };

        let mut pub_field_count=0usize;
        let mut fn_ptr_count=0usize;

        for field in &fields {
            if field.is_public() {
                pub_field_count+=1;
            }
            fn_ptr_count+=field.functions.len();
        }

        Self {
            discriminant:p.discriminant,
            attrs:p.attrs,
            name:p.name,
            kind,
            pub_field_count,
            fn_ptr_count,
            fields,
            _priv: (),
        }
    }

    }

//////////////////////////////////////////////////////////////////////////////

/// Represent a struct field
///
#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct Field<'a> {
    pub(crate) index:FieldIndex,
    pub(crate) attrs: &'a [Attribute],
    pub(crate) vis: &'a Visibility,
    pub(crate) referenced_lifetimes: Vec<LifetimeIndex>,
    /// identifier for the field,which is either an index(in a tuple struct) or a name.
    pub(crate) ident: FieldIdent<'a>,
    pub(crate) ty: &'a Type,
    /// Whether the type of this field is just a function pointer.
    pub(crate) is_function:bool,
    /// The type used to get the AbiInfo of the field.
    /// This has all parameter and return types of function pointers removed.
    /// Extracted into the `functions` field of this struct.
    pub(crate) mutated_ty: Type,
    /// The function pointers from this field.
    pub(crate) functions:Vec<Function<'a>>,
}

impl<'a> Field<'a> {
    pub(crate) fn new(
        index: FieldIndex,
        field: &'a SynField,
        span: Span,
        override_vis:Option<&'a Visibility>,
        tv: &mut TypeVisitor<'a>,
    ) -> Self {
        let ident = match field.ident.as_ref() {
            Some(ident) => FieldIdent::Named(ident),
            None => FieldIdent::new_index(index.pos, span),
        };

        let mut mutated_ty=field.ty.clone();

        let visit_info = tv.visit_field(&mut mutated_ty);

        let is_function=match field.ty {
            Type::BareFn{..}=>true,
            _=>false,
        };

        Self {
            index,
            attrs: &field.attrs,
            vis: override_vis.unwrap_or(&field.vis),
            referenced_lifetimes: visit_info.referenced_lifetimes,
            ident,
            ty: &field.ty,
            is_function,
            mutated_ty,
            functions: visit_info.functions,
        }
    }

    pub(crate) fn is_public(&self)->bool{
        match self.vis {
            Visibility::Public{..}=>true,
            _=>false,
        }
    }


    pub(crate) fn ident(&self)->&Ident{
        match &self.ident {
            FieldIdent::Index(_,ident)=>ident,
            FieldIdent::Named(ident)=>ident,
        }
    }

    pub(crate) fn from_iter<I>(
        p:StructParams<'a>,
        fields: I, 
        tv: &mut TypeVisitor<'a>
    ) -> Vec<Self>
    where
        I: IntoIterator<Item = &'a SynField>,
    {
        fields
            .into_iter()
            .enumerate()
            .map(|(pos, f)|{ 
                let fi=FieldIndex{variant:p.variant,pos};
                Field::new(fi, f, p.name.span(),p.override_vis, tv)
            })
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

