use core_extensions::matches;
use proc_macro2::{TokenStream,Span};
use quote::{quote,ToTokens};

use crate::{
    ignored_wrapper::Ignored,
    to_token_fn::ToTokenFnMut,
};

use super::common_tokens::CommonTokens;


/// Used to parse ReprAttr from attributes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UncheckedReprAttr{
    repr_kind:Option<UncheckedReprKind>,
    repr_span:Ignored<Span>,
    discriminant_repr:Option<DiscriminantRepr>,
}

impl Default for UncheckedReprAttr{
    fn default()->Self{
        Self{
            repr_kind:None,
            repr_span:Ignored::new(Span::call_site()),
            discriminant_repr:None,
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UncheckedReprKind{
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
pub enum ReprAttr{
    C(Option<DiscriminantRepr>,Ignored<Span>),
    Transparent(Ignored<Span>),
    /// Means that only `repr(IntegerType)` was used.
    Int(DiscriminantRepr,Ignored<Span>),
}


pub(crate) static REPR_ERROR_MSG:&str="\n\
    the #[repr(..)] attribute must be one of the supported attributes:\n\
    \t- #[repr(C)]\n\
    \t- #[repr(transparent)]\n\
    \t- #[repr(integer_type_up_to_64_bits)]:enums only\n\
    \t- #[repr(usize)]:enums only\n\
    \t- #[repr(isize)]:enums only\n\
    \t- #[repr(align(<some_integer>))]\n\
";


impl UncheckedReprAttr{
    pub fn set_repr_kind(
        &mut self,
        repr_kind:UncheckedReprKind,
        repr_span:proc_macro2::Span
    )-> Result<(),syn::Error> {
        if let Some(from)=self.discriminant_repr {
            return_syn_err!(
                repr_span,
                "Attempting to override {:?} representation with {:?}.",
                from,
                repr_kind
            );
        }
        self.repr_kind=Some(repr_kind);
        self.repr_span.value=repr_span;
        Ok(())
    }
    pub fn set_discriminant_repr(
        &mut self,
        discriminant_repr:DiscriminantRepr,
        repr_span:proc_macro2::Span
    )-> Result<(),syn::Error> {
        if let Some(x)=self.discriminant_repr {
            return_syn_err!(
                repr_span,
                "Attempting to override {:?} representation with {:?}.",
                x,
                discriminant_repr
            );
        }
        self.repr_kind=self.repr_kind.or(Some(UncheckedReprKind::Int));
        self.repr_span.value=repr_span;

        self.discriminant_repr=Some(discriminant_repr);
        Ok(())
    }
}


impl DiscriminantRepr{
    /// Gets a `DiscriminantRepr` from the identifier of an integer type.
    ///
    /// Returns None if the identifier is not a supported integer type.
    pub fn from_ident(ident:&syn::Ident)->Option<Self>{
        if ident=="u8" {
            Some(DiscriminantRepr::U8)
        }else if ident=="i8" {
            Some(DiscriminantRepr::I8)
        }else if ident=="u16" {
            Some(DiscriminantRepr::U16)
        }else if ident=="i16" {
            Some(DiscriminantRepr::I16)
        }else if ident=="u32" {
            Some(DiscriminantRepr::U32)
        }else if ident=="i32" {
            Some(DiscriminantRepr::I32)
        }else if ident=="u64" {
            Some(DiscriminantRepr::U64)
        }else if ident=="i64" {
            Some(DiscriminantRepr::I64)
        }else if ident=="usize" {
            Some(DiscriminantRepr::Usize)
        }else if ident=="isize" {
            Some(DiscriminantRepr::Isize)
        }else{
            None
        }
    }
}


impl ReprAttr{
    pub fn new(unchecked:UncheckedReprAttr)-> Result<Self,syn::Error> {
        let span=unchecked.repr_span;
        let ura:UncheckedReprKind=unchecked.repr_kind.ok_or_else(||{
            syn_err!(*span,"{}",REPR_ERROR_MSG)
        })?;
        let dr:Option<DiscriminantRepr>=unchecked.discriminant_repr;
        Ok(match (ura,dr) {
            (UncheckedReprKind::C,x)=>
                ReprAttr::C(x,span),
            (UncheckedReprKind::Transparent,None)=>
                ReprAttr::Transparent(span),
            (UncheckedReprKind::Transparent,Some(_))=>{
                return_syn_err!(
                    *span,
                    "repr(transparent) cannot be combined with repr(IntegerType)",
                )
            }
            (UncheckedReprKind::Int,None)=>
                panic!("Bug:(UncheckedReprKind::Int,None)"),
            (UncheckedReprKind::Int,Some(x))=>
                ReprAttr::Int(x,span),
        })
    }

    /// Gets the type of the discriminant determined by this representation attribute.
    /// Returns None if the representation is `#[repr(transparent)]`.
    pub fn type_ident(&self)->Option<syn::Ident>{
        let int_repr=match *self {
            ReprAttr::C(None,_)=>
                DiscriminantRepr::Isize,
            ReprAttr::C(Some(int_repr),_)|ReprAttr::Int(int_repr,_)=>
                int_repr,
            ReprAttr::Transparent(_)=>
                return None,
        };

        let ty_lit=match int_repr {
            DiscriminantRepr::U8 =>"u8",
            DiscriminantRepr::U16=>"u16",
            DiscriminantRepr::U32=>"u32",
            DiscriminantRepr::U64=>"u64",
            DiscriminantRepr::I8 =>"i8",
            DiscriminantRepr::I16=>"i16",
            DiscriminantRepr::I32=>"i32",
            DiscriminantRepr::I64=>"i64",
            DiscriminantRepr::Usize=>"usize",
            DiscriminantRepr::Isize=>"isize",
        };

        Some(syn::Ident::new(ty_lit,Span::call_site()))
    }

    /// Returns a type which outputs a `DiscriminantRepr` with 
    /// a slice of the items in the iterator,
    /// where each Option is unwrapped by replacing `None`s 
    /// with the value of the last `Some()` incremented by the distance to the current element.
    pub(crate) fn tokenize_discriminant_exprs<'a,I>(
        self,
        exprs:I,
        ctokens:&'a CommonTokens,
    )->impl ToTokens+'a 
    where
        I:IntoIterator<Item=Option<&'a syn::Expr>>+'a
    {
        let mut exprs=exprs.into_iter();

        ToTokenFnMut::new(move|ts|{
            let int_repr=match self {
                ReprAttr::C(x,_)=>x,
                ReprAttr::Int(x,_)=>Some(x),
                ReprAttr::Transparent(_)=>unreachable!(),
            };

            match int_repr.unwrap_or(DiscriminantRepr::Isize) {
                DiscriminantRepr::U8 =>quote!(__TLDiscriminants::U8  ),
                DiscriminantRepr::U16=>quote!(__TLDiscriminants::U16 ),
                DiscriminantRepr::U32=>quote!(__TLDiscriminants::U32 ),
                DiscriminantRepr::U64=>quote!(__TLDiscriminants::U64 ),
                DiscriminantRepr::I8 =>quote!(__TLDiscriminants::I8  ),
                DiscriminantRepr::I16=>quote!(__TLDiscriminants::I16 ),
                DiscriminantRepr::I32=>quote!(__TLDiscriminants::I32 ),
                DiscriminantRepr::I64=>quote!(__TLDiscriminants::I64 ),
                DiscriminantRepr::Usize=>quote!(__TLDiscriminants::Usize  ),
                DiscriminantRepr::Isize=>quote!(__TLDiscriminants::Isize  ),
            }.to_tokens(ts);
            
            ctokens.paren.surround(ts,|ts|{
                quote!( abi_stable::rslice! ).to_tokens(ts);
                ctokens.bracket.surround(ts,|ts|{
                    tokenize_discriminant_exprs_inner(&mut exprs,ctokens,ts);
                });
            });
        })
    }


    /// Returns a type which outputs a slice with the items in the iterator,
    /// where each Option is unwrapped by replacing `None`s 
    /// with the value of the last `Some()` incremented by the distance to the current element.
    pub(crate) fn tokenize_discriminant_slice<'a,I>(
        self,
        exprs:I,
        ctokens:&'a CommonTokens,
    )->impl ToTokens+'a 
    where
        I:IntoIterator<Item=Option<&'a syn::Expr>>+'a
    {
        let mut exprs=exprs.into_iter();

        ToTokenFnMut::new(move|ts|{
            ctokens.and_.to_tokens(ts);
            ctokens.bracket.surround(ts,|ts|{
                tokenize_discriminant_exprs_inner(&mut exprs,ctokens,ts);
            });
        })
    }
}


#[allow(dead_code)]
impl ReprAttr{
    pub fn span(self)->Span{
        match self {
            |ReprAttr::C(_,span)
            |ReprAttr::Int(_,span)
            |ReprAttr::Transparent(span)
            =>*span
        }
    }

    pub fn is_repr_transparent(self)->bool{
        matches!(ReprAttr::Transparent{..}=self)
    }

    pub fn is_repr_c(self)->bool{
        matches!(ReprAttr::C{..}=self)
    }

    pub fn is_repr_int(self)->bool{
        matches!(ReprAttr::Int{..}=self)
    }
}


/// Outputs the items in the iterator separated by commas,
/// where each Option is unwrapped by replacing `None`s 
/// with the value of the last `Some()` incremented by the distance to the current element.
fn tokenize_discriminant_exprs_inner<'a,I>(
    exprs:I,
    ctokens:&'a CommonTokens,
    ts:&mut TokenStream
)where
    I:Iterator<Item=Option<&'a syn::Expr>>
{
    let zero_expr=crate::utils::expr_from_int(0);
    let mut last_explicit_discr=&zero_expr;
    let mut since_last_expr=0;

    for expr in exprs {
        match expr {
            Some(discr)=>{
                discr.to_tokens(ts);

                last_explicit_discr=discr;
                since_last_expr=1;
            }
            None=>{
                let offset=crate::utils::uint_lit(since_last_expr);

                ctokens.paren.surround(ts,|ts|{ 
                    last_explicit_discr.to_tokens(ts);
                });
                ctokens.add.to_tokens(ts);
                offset.to_tokens(ts);

                since_last_expr+=1;
            }
        }
        ctokens.comma.to_tokens(ts);
    }
}




impl ToTokens for ReprAttr{
    fn to_tokens(&self, ts: &mut TokenStream) {
        match *self {
            ReprAttr::C(None,_)=>{
                quote!(__ReprAttr::C(__RNone))
            }
            ReprAttr::C(Some(int_repr),_)=>{
                let int_repr=discr_repr_tokenizer(int_repr);
                quote!(__ReprAttr::C(__RSome(#int_repr)))
            }
            ReprAttr::Transparent(_)=>{
                quote!(__ReprAttr::Transparent)
            }
            ReprAttr::Int(int_repr,_)=>{
                let int_repr=discr_repr_tokenizer(int_repr);
                quote!(__ReprAttr::Int(#int_repr))
            }
        }.to_tokens(ts);
    }
}

fn discr_repr_tokenizer(repr:DiscriminantRepr)->impl ToTokens{
    ToTokenFnMut::new(move|ts|{
        match repr {
            DiscriminantRepr::U8=>quote!(__DiscriminantRepr::U8),
            DiscriminantRepr::I8=>quote!(__DiscriminantRepr::I8),
            DiscriminantRepr::U16=>quote!(__DiscriminantRepr::U16),
            DiscriminantRepr::I16=>quote!(__DiscriminantRepr::I16),
            DiscriminantRepr::U32=>quote!(__DiscriminantRepr::U32),
            DiscriminantRepr::I32=>quote!(__DiscriminantRepr::I32),
            DiscriminantRepr::U64=>quote!(__DiscriminantRepr::U64),
            DiscriminantRepr::I64=>quote!(__DiscriminantRepr::I64),
            DiscriminantRepr::Usize=>quote!(__DiscriminantRepr::Usize),
            DiscriminantRepr::Isize=>quote!(__DiscriminantRepr::Isize),
        }.to_tokens(ts);
    })
}