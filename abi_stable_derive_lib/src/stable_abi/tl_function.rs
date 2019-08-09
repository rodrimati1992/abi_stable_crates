/*!
Contains types related to the type layout of function pointers.
*/

use syn::Type;

use super::*;

use crate::{
    composite_collections::{SmallStartLen as StartLen},
    datastructure::{FieldMap,Field},
    fn_pointer_extractor::{Function,TypeVisitor},
    lifetimes::LifetimeIndex,
};


///////////////////////////////////////////////////////////////////////////////

/// Associates extra information related to function pointers to a type declaration.
#[allow(dead_code)]
pub(crate) struct VisitedFieldMap<'a>{
    pub(crate) map:FieldMap<VisitedField<'a>>,
    pub(crate) fn_ptr_count:usize,
    priv_:(),
}


impl<'a> VisitedFieldMap<'a>{
    pub(crate) fn new(
        ds:&'a DataStructure<'a>,
        config:&'a StableAbiOptions<'a>,
        arenas: &'a Arenas, 
        ctokens: &'a CommonTokens<'a>
    )->Self{
        let mut tv = TypeVisitor::new(arenas, ctokens.as_ref(), ds.generics);
        let mut fn_ptr_count = 0;
        let map=FieldMap::<VisitedField<'a>>::with(ds,|field|{
            let mut mutated_ty=config.changed_types[field].unwrap_or(field.ty).clone();
            let is_opaque=config.opaque_fields[field];

            let is_function=match mutated_ty {
                Type::BareFn{..}=>!is_opaque,
                _=>false,
            };

            let visit_info = tv.visit_field(&mut mutated_ty);


            let functions=if is_opaque { Vec::new() }else{ visit_info.functions };
            fn_ptr_count+=functions.len();

            VisitedField{
                inner:field,
                referenced_lifetimes: visit_info.referenced_lifetimes,
                is_function,
                mutated_ty,
                functions,
            }
        });
        Self{
            map,
            fn_ptr_count,
            priv_:(),
        }
    }
}


///////////////////////////////////////////////////////////////////////////////


/// A `Field<'a>` with extra information.
#[allow(dead_code)]
pub struct VisitedField<'a>{
    pub(crate) inner:&'a Field<'a>,
    /// The lifetimes used in the field's type.
    pub(crate) referenced_lifetimes: Vec<LifetimeIndex>,
    /// identifier for the field,which is either an index(in a tuple struct) or a name.
    /// Whether the type of this field is just a function pointer.
    pub(crate) is_function:bool,
    /// The type used to get the AbiInfo of the field.
    /// This has all parameter and return types of function pointers removed.
    /// Extracted into the `functions` field of this struct.
    pub(crate) mutated_ty: Type,
    /// The function pointers from this field.
    pub(crate) functions:Vec<Function<'a>>,
}



///////////////////////////////////////////////////////////////////////////////


/// This is how a function pointer is stored,
/// in which every field is a range into `TLFunctions`.
#[derive(Copy,Clone,Debug,PartialEq,Eq,Ord,PartialOrd)]
pub struct CompTLFunction<'a>{
    pub(crate) ctokens:&'a CommonTokens<'a>,
    pub(crate) name:StartLen,
    pub(crate) bound_lifetimes:StartLen,
    pub(crate) param_names:StartLen,
    pub(crate) param_abi_infos:StartLen,
    pub(crate) paramret_lifetime_indices:StartLen,
    pub(crate) return_abi_info:Option<u16>,
}


impl<'a> CompTLFunction<'a>{
    /// Constructs a default CompTLFunction.
    pub(crate) fn new(ctokens:&'a CommonTokens)->Self{
        CompTLFunction{
            ctokens,
            name:StartLen::EMPTY,
            bound_lifetimes:StartLen::EMPTY,
            param_names:StartLen::EMPTY,
            param_abi_infos:StartLen::EMPTY,
            paramret_lifetime_indices:StartLen::EMPTY,
            return_abi_info:None,
        }
    }
}


impl<'a> ToTokens for CompTLFunction<'a> {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let ct=self.ctokens;
        to_stream!(ts;ct.comp_tl_functions,ct.colon2,ct.new);
        ct.paren.surround(ts,|ts|{
            self.name.tokenize(ct.as_ref(),ts);
            ct.comma.to_tokens(ts);

            self.bound_lifetimes.tokenize(ct.as_ref(),ts);
            ct.comma.to_tokens(ts);

            self.param_names.tokenize(ct.as_ref(),ts);
            ct.comma.to_tokens(ts);

            self.param_abi_infos.tokenize(ct.as_ref(),ts);
            ct.comma.to_tokens(ts);

            self.paramret_lifetime_indices.tokenize(ct.as_ref(),ts);
            ct.comma.to_tokens(ts);

            match self.return_abi_info {
                Some(x) => {
                    ct.rsome.to_tokens(ts);
                    ct.paren.surround(ts,|ts|{
                        x.to_tokens(ts);
                    });
                },
                None => {
                    ct.rnone.to_tokens(ts);
                },
            }
            ct.comma.to_tokens(ts);
        });
    }
}

