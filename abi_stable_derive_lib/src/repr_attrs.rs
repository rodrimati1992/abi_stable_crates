
use proc_macro2::TokenStream;
use quote::{quote,ToTokens};

use crate::to_token_fn::ToTokenFnMut;



#[derive(Debug,Default, Copy, Clone, PartialEq, Eq)]
pub struct UncheckedReprAttr{
    repr_kind:Option<UncheckedReprKind>,
    discriminant_repr:Option<DiscriminantRepr>,
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



#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReprAttr{
    C(Option<DiscriminantRepr>),
    Transparent,
    /// Means that only `repr(IntegerType)` was used.
    Int(DiscriminantRepr),
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
    // pub fn repr_kind(&self)->Option<UncheckedReprKind>{
    //     self.repr_kind
    // }
    // pub fn discriminant_repr(&self)->Option<DiscriminantRepr>{
    //     self.discriminant_repr
    // }
    pub fn set_repr_kind(&mut self,repr_kind:UncheckedReprKind){
        if let Some(from)=self.discriminant_repr {
            panic!(
                "\n\nattempting to override {:?} representation with {:?}\n\n",
                from,repr_kind
            );
        }
        self.repr_kind=Some(repr_kind);
    }
    pub fn set_discriminant_repr(&mut self,discriminant_repr:DiscriminantRepr){
        if let Some(x)=self.discriminant_repr {
            panic!(
                "\n\nattempting to override {:?} representation with {:?}\n\n",
                x,
                discriminant_repr
            );
        }
        self.repr_kind=self.repr_kind.or(Some(UncheckedReprKind::Int));

        self.discriminant_repr=Some(discriminant_repr);
    }
}


impl DiscriminantRepr{
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
    pub fn new(unchecked:UncheckedReprAttr)->Self{
        let ura:UncheckedReprKind=unchecked.repr_kind.expect(REPR_ERROR_MSG);
        let dr:Option<DiscriminantRepr>=unchecked.discriminant_repr;
        match (ura,dr) {
            (UncheckedReprKind::C,x)=>
                ReprAttr::C(x),
            (UncheckedReprKind::Transparent,None)=>
                ReprAttr::Transparent,
            (UncheckedReprKind::Transparent,Some(_))=>
                panic!("repr(transparent) cannot be combined with repr(IntegerType)"),
            (UncheckedReprKind::Int,None)=>
                panic!("Bug:(UncheckedReprKind::Int,None)"),
            (UncheckedReprKind::Int,Some(x))=>
                ReprAttr::Int(x),
        }
    }
    pub fn tokenize_discriminant_expr<'a>(self,expr:Option<&'a syn::Expr>)->impl ToTokens+'a {
        ToTokenFnMut::new(move|ts|{
            let expr=match expr {
                Some(x) => x,
                None => return,
            };

            let int_repr=match self {
                ReprAttr::C(x)=>x,
                ReprAttr::Int(x)=>Some(x),
                ReprAttr::Transparent=>unreachable!(),
            };
                


            let constructor=match int_repr.unwrap_or(DiscriminantRepr::Isize) {
                DiscriminantRepr::U8 =>quote!(__TLDiscriminant::from_u8 ),
                DiscriminantRepr::U16=>quote!(__TLDiscriminant::from_u16),
                DiscriminantRepr::U32=>quote!(__TLDiscriminant::from_u32),
                DiscriminantRepr::U64=>quote!(__TLDiscriminant::from_u64),
                DiscriminantRepr::I8 =>quote!(__TLDiscriminant::from_i8 ),
                DiscriminantRepr::I16=>quote!(__TLDiscriminant::from_i16),
                DiscriminantRepr::I32=>quote!(__TLDiscriminant::from_i32),
                DiscriminantRepr::I64=>quote!(__TLDiscriminant::from_i64),
                DiscriminantRepr::Usize=>quote!(__TLDiscriminant::Usize ),
                DiscriminantRepr::Isize=>quote!(__TLDiscriminant::Isize ),
            };

            quote!(
                .set_discriminant(#constructor(#expr))
            ).to_tokens(ts);
        })
    }
}



impl ToTokens for ReprAttr{
    fn to_tokens(&self, ts: &mut TokenStream) {
        match *self {
            ReprAttr::C(None)=>{
                quote!(__ReprAttr::C(__RNone))
            }
            ReprAttr::C(Some(int_repr))=>{
                let int_repr=discr_repr_tokenizer(int_repr);
                quote!(__ReprAttr::C(__RSome(#int_repr)))
            }
            ReprAttr::Transparent=>{
                quote!(__ReprAttr::Transparent)
            }
            ReprAttr::Int(int_repr)=>{
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