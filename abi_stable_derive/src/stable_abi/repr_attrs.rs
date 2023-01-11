#[allow(unused_imports)]
use core_extensions::{matches, SelfOps};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::ParseBuffer;

use as_derive_utils::{
    parse_utils::ParseBufferExt, return_syn_err, syn_err, to_token_fn::ToTokenFnMut,
};

use crate::{ignored_wrapper::Ignored, literals_constructors::rslice_tokenizer};

use super::common_tokens::CommonTokens;

/// Used to parse ReprAttr from attributes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UncheckedReprAttr {
    is_aligned: Option<u32>,
    is_packed: Option<u32>,
    repr_kind: Option<UncheckedReprKind>,
    repr_span: Ignored<Span>,
    discriminant_repr: Option<DiscriminantRepr>,
}

impl Default for UncheckedReprAttr {
    fn default() -> Self {
        Self {
            is_aligned: None,
            is_packed: None,
            repr_kind: None,
            repr_span: Ignored::new(Span::call_site()),
            discriminant_repr: None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UncheckedReprKind {
    C,
    Transparent,
    /// Means that only `repr(IntegerType)` was used.
    Int,
}

/// How the discriminant of an enum is represented.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DiscriminantRepr {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Usize,
    /// This is the default discriminant type for `repr(C)`.
    Isize,
}

/// The representation attribute of the type.
///
/// This doesn't include `#[repr(align())]` since the alignment is
/// stored as part of TypeLayout anyway.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Repr {
    C(Option<DiscriminantRepr>),
    Transparent,
    /// Means that only `repr(IntegerType)` was used.
    Int(DiscriminantRepr),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReprAttr {
    pub span: Ignored<Span>,
    pub is_aligned: Option<u32>,
    pub is_packed: Option<u32>,
    pub variant: Repr,
}

pub(crate) static REPR_ERROR_MSG: &str = "\n\
    the #[repr(..)] attribute must be one of the supported attributes:\n\
    \t- #[repr(C)]\n\
    \t- #[repr(transparent)]\n\
    \t- #[repr(integer_type_up_to_64_bits)]:enums only\n\
    \t- #[repr(usize)]:enums only\n\
    \t- #[repr(isize)]:enums only\n\
    \t- #[repr(align(<some_integer>))]\n\
";

impl UncheckedReprAttr {
    pub fn set_aligned(&mut self, alignment: u32) -> Result<(), syn::Error> {
        self.is_aligned = Some(alignment);
        Ok(())
    }
    pub fn set_packed(&mut self, packing: Option<u32>) -> Result<(), syn::Error> {
        self.is_packed = packing.or(Some(1));
        Ok(())
    }
    pub fn set_repr_kind(
        &mut self,
        repr_kind: UncheckedReprKind,
        repr_span: proc_macro2::Span,
    ) -> Result<(), syn::Error> {
        if let Some(from) = self.discriminant_repr {
            return_syn_err!(
                repr_span,
                "Attempting to override {:?} representation with {:?}.",
                from,
                repr_kind
            );
        }
        self.repr_kind = Some(repr_kind);
        self.repr_span.value = repr_span;
        Ok(())
    }
    pub fn set_discriminant_repr(
        &mut self,
        discriminant_repr: DiscriminantRepr,
        repr_span: proc_macro2::Span,
    ) -> Result<(), syn::Error> {
        if let Some(x) = self.discriminant_repr {
            return_syn_err!(
                repr_span,
                "Attempting to override {:?} representation with {:?}.",
                x,
                discriminant_repr
            );
        }
        self.repr_kind = self.repr_kind.or(Some(UncheckedReprKind::Int));
        self.repr_span.value = repr_span;

        self.discriminant_repr = Some(discriminant_repr);
        Ok(())
    }
}

mod kw {
    syn::custom_keyword! {u8}
    syn::custom_keyword! {i8}
    syn::custom_keyword! {u16}
    syn::custom_keyword! {i16}
    syn::custom_keyword! {u32}
    syn::custom_keyword! {i32}
    syn::custom_keyword! {u64}
    syn::custom_keyword! {i64}
    syn::custom_keyword! {usize}
    syn::custom_keyword! {isize}
}

impl DiscriminantRepr {
    pub fn from_parser(input: &'_ ParseBuffer<'_>) -> Option<Self> {
        if input.peek_parse(kw::u8).ok()?.is_some() {
            Some(DiscriminantRepr::U8)
        } else if input.peek_parse(kw::i8).ok()?.is_some() {
            Some(DiscriminantRepr::I8)
        } else if input.peek_parse(kw::u16).ok()?.is_some() {
            Some(DiscriminantRepr::U16)
        } else if input.peek_parse(kw::i16).ok()?.is_some() {
            Some(DiscriminantRepr::I16)
        } else if input.peek_parse(kw::u32).ok()?.is_some() {
            Some(DiscriminantRepr::U32)
        } else if input.peek_parse(kw::i32).ok()?.is_some() {
            Some(DiscriminantRepr::I32)
        } else if input.peek_parse(kw::u64).ok()?.is_some() {
            Some(DiscriminantRepr::U64)
        } else if input.peek_parse(kw::i64).ok()?.is_some() {
            Some(DiscriminantRepr::I64)
        } else if input.peek_parse(kw::usize).ok()?.is_some() {
            Some(DiscriminantRepr::Usize)
        } else if input.peek_parse(kw::isize).ok()?.is_some() {
            Some(DiscriminantRepr::Isize)
        } else {
            None
        }
    }
}

impl ReprAttr {
    pub fn new(unchecked: UncheckedReprAttr) -> Result<Self, syn::Error> {
        let span = unchecked.repr_span;
        let is_aligned = unchecked.is_aligned;
        let is_packed = unchecked.is_packed;
        let ura: UncheckedReprKind = unchecked
            .repr_kind
            .ok_or_else(|| syn_err!(*span, "{}", REPR_ERROR_MSG))?;
        let dr: Option<DiscriminantRepr> = unchecked.discriminant_repr;
        let variant = match (ura, dr) {
            (UncheckedReprKind::C, x) => Repr::C(x),
            (UncheckedReprKind::Transparent, None) => Repr::Transparent,
            (UncheckedReprKind::Transparent, Some(_)) => {
                return_syn_err!(
                    *span,
                    "repr(transparent) cannot be combined with repr(IntegerType)",
                )
            }
            (UncheckedReprKind::Int, None) => panic!("Bug:(UncheckedReprKind::Int,None)"),
            (UncheckedReprKind::Int, Some(x)) => Repr::Int(x),
        };
        Ok(Self {
            span,
            variant,
            is_aligned,
            is_packed,
        })
    }

    /// Gets the type of the discriminant determined by this representation attribute.
    /// Returns None if the representation is `#[repr(transparent)]`.
    pub fn type_ident(&self) -> Option<syn::Ident> {
        let int_repr = match self.variant {
            Repr::C(None) => DiscriminantRepr::Isize,
            Repr::C(Some(int_repr)) | Repr::Int(int_repr) => int_repr,
            Repr::Transparent => return None,
        };

        let ty_lit = match int_repr {
            DiscriminantRepr::U8 => "u8",
            DiscriminantRepr::U16 => "u16",
            DiscriminantRepr::U32 => "u32",
            DiscriminantRepr::U64 => "u64",
            DiscriminantRepr::I8 => "i8",
            DiscriminantRepr::I16 => "i16",
            DiscriminantRepr::I32 => "i32",
            DiscriminantRepr::I64 => "i64",
            DiscriminantRepr::Usize => "usize",
            DiscriminantRepr::Isize => "isize",
        };

        Some(syn::Ident::new(ty_lit, Span::call_site()))
    }

    /// Returns a type which outputs a `DiscriminantRepr` with
    /// a slice of the items in the iterator,
    /// where each Option is unwrapped by replacing `None`s
    /// with the value of the last `Some()` incremented by the distance to the current element.
    pub(crate) fn tokenize_discriminant_exprs<'a, I>(
        self,
        exprs: I,
        ctokens: &'a CommonTokens,
    ) -> impl ToTokens + 'a
    where
        I: IntoIterator<Item = Option<&'a syn::Expr>> + 'a,
    {
        let mut exprs = exprs.into_iter();

        ToTokenFnMut::new(move |ts| {
            let int_repr = match self.variant {
                Repr::C(x) => x,
                Repr::Int(x) => Some(x),
                Repr::Transparent => unreachable!(),
            };

            match int_repr.unwrap_or(DiscriminantRepr::Isize) {
                DiscriminantRepr::U8 => quote!(__TLDiscriminants::from_u8_slice),
                DiscriminantRepr::U16 => quote!(__TLDiscriminants::from_u16_slice),
                DiscriminantRepr::U32 => quote!(__TLDiscriminants::from_u32_slice),
                DiscriminantRepr::U64 => quote!(__TLDiscriminants::from_u64_slice),
                DiscriminantRepr::I8 => quote!(__TLDiscriminants::from_i8_slice),
                DiscriminantRepr::I16 => quote!(__TLDiscriminants::from_i16_slice),
                DiscriminantRepr::I32 => quote!(__TLDiscriminants::from_i32_slice),
                DiscriminantRepr::I64 => quote!(__TLDiscriminants::from_i64_slice),
                DiscriminantRepr::Usize => quote!(__TLDiscriminants::from_usize_slice),
                DiscriminantRepr::Isize => quote!(__TLDiscriminants::from_isize_slice),
            }
            .to_tokens(ts);

            ctokens.paren.surround(ts, |ts| {
                tokenize_discriminant_exprs_inner(&mut exprs, SliceType::RSlice, ctokens, ts);
            });
        })
    }

    /// Returns a type which outputs a slice with the items in the iterator,
    /// where each Option is unwrapped by replacing `None`s
    /// with the value of the last `Some()` incremented by the distance to the current element.
    pub(crate) fn tokenize_discriminant_slice<'a, I>(
        self,
        exprs: I,
        ctokens: &'a CommonTokens,
    ) -> impl ToTokens + 'a
    where
        I: IntoIterator<Item = Option<&'a syn::Expr>> + 'a,
    {
        let mut exprs = exprs.into_iter();

        ToTokenFnMut::new(move |ts| {
            tokenize_discriminant_exprs_inner(&mut exprs, SliceType::StdSlice, ctokens, ts);
        })
    }
}

#[allow(dead_code)]
impl ReprAttr {
    pub fn span(self) -> Span {
        *self.span
    }

    pub fn is_repr_transparent(self) -> bool {
        matches!(self.variant, Repr::Transparent { .. })
    }

    pub fn is_repr_c(self) -> bool {
        matches!(self.variant, Repr::C { .. })
    }

    pub fn is_repr_int(self) -> bool {
        matches!(self.variant, Repr::Int { .. })
    }
}

#[derive(Copy, Clone)]
enum SliceType {
    StdSlice,
    RSlice,
}

/// Outputs the items in the iterator separated by commas,
/// where each Option is unwrapped by replacing `None`s
/// with the value of the last `Some()` incremented by the distance to the current element.
fn tokenize_discriminant_exprs_inner<'a, I>(
    exprs: I,
    type_: SliceType,
    ctokens: &'a CommonTokens,
    ts: &mut TokenStream,
) where
    I: Iterator<Item = Option<&'a syn::Expr>>,
{
    let zero_expr = crate::utils::expr_from_int(0);
    let mut last_explicit_discr = &zero_expr;
    let mut since_last_expr = 0;

    let iter = exprs.map(|expr| match expr {
        Some(discr) => {
            let ts = quote!(#discr);
            last_explicit_discr = discr;
            since_last_expr = 1;
            ts
        }
        None => {
            let offset = crate::utils::uint_lit(since_last_expr);
            let ts = quote!( (#last_explicit_discr)+#offset );
            since_last_expr += 1;
            ts
        }
    });
    match type_ {
        SliceType::StdSlice => {
            ctokens.and_.to_tokens(ts);
            ctokens.bracket.surround(ts, |ts| {
                for elem in iter {
                    elem.to_tokens(ts);
                    ctokens.comma.to_tokens(ts)
                }
            });
        }
        SliceType::RSlice => {
            rslice_tokenizer(iter).to_tokens(ts);
        }
    }
}

impl ToTokens for ReprAttr {
    fn to_tokens(&self, ts: &mut TokenStream) {
        match self.variant {
            Repr::C(None) => {
                quote!(__ReprAttr::C)
            }
            Repr::C(Some(int_repr)) => {
                let int_repr = discr_repr_tokenizer(int_repr);
                quote!(__ReprAttr::CAndInt(#int_repr))
            }
            Repr::Transparent => {
                quote!(__ReprAttr::Transparent)
            }
            Repr::Int(int_repr) => {
                let int_repr = discr_repr_tokenizer(int_repr);
                quote!(__ReprAttr::Int(#int_repr))
            }
        }
        .to_tokens(ts);
    }
}

fn discr_repr_tokenizer(repr: DiscriminantRepr) -> impl ToTokens {
    ToTokenFnMut::new(move |ts| {
        match repr {
            DiscriminantRepr::U8 => quote!(__DiscriminantRepr::U8),
            DiscriminantRepr::I8 => quote!(__DiscriminantRepr::I8),
            DiscriminantRepr::U16 => quote!(__DiscriminantRepr::U16),
            DiscriminantRepr::I16 => quote!(__DiscriminantRepr::I16),
            DiscriminantRepr::U32 => quote!(__DiscriminantRepr::U32),
            DiscriminantRepr::I32 => quote!(__DiscriminantRepr::I32),
            DiscriminantRepr::U64 => quote!(__DiscriminantRepr::U64),
            DiscriminantRepr::I64 => quote!(__DiscriminantRepr::I64),
            DiscriminantRepr::Usize => quote!(__DiscriminantRepr::Usize),
            DiscriminantRepr::Isize => quote!(__DiscriminantRepr::Isize),
        }
        .to_tokens(ts);
    })
}
