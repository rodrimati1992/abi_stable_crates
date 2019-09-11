use crate::{
    arenas::Arenas,
    composite_collections::{SmallStartLen as StartLen},
    lifetimes::{LifetimeIndex,LifetimeIndexArray,LifetimeIndexPair,LifetimeRange},
    literals_constructors::{rslice_tokenizer,rstr_tokenizer},
    ToTokenFnMut,
};

use super::{
    attribute_parsing::LayoutConstructor,
    CommonTokens,
};

use core_extensions::SelfOps;

use proc_macro2::TokenStream as TokenStream2;

use quote::{quote,ToTokens};

use std::{
    collections::HashMap,
    fmt::Display,
};



pub(crate) struct SharedVars<'a>{
    arenas:&'a Arenas,
    ctokens:&'a CommonTokens<'a>,
    strings: String,
    lifetime_indices: Vec<LifetimeIndex>,
    type_layouts_map: HashMap<(LayoutConstructor,&'a syn::Type),u16>,
    type_layouts: Vec<(LayoutConstructor,&'a syn::Type)>,
    constants: Vec<&'a syn::Expr>,
}

impl<'a> SharedVars<'a>{
    pub(crate) fn new(arenas:&'a Arenas, ctokens:&'a CommonTokens)->Self{
        Self{
            arenas,
            ctokens,
            strings: String::new(),
            lifetime_indices: Vec::new(),
            type_layouts: Vec::new(),
            type_layouts_map: HashMap::new(),
            constants: Vec::new(),
        }
    }

    pub(crate) fn arenas(&self)->&'a Arenas{
        self.arenas
    }

    pub(crate) fn ctokens(&self)->&'a CommonTokens<'a>{
        self.ctokens
    }

    pub(crate) fn push_str(&mut self,s:&str)->StartLen{
        let start=self.strings.len();
        self.strings.push_str(s);
        StartLen::new(start,self.strings.len()-start)
    }

    pub fn extend_with_display<I>(&mut self,separator:&str,iter:I)->StartLen
    where
        I:IntoIterator,
        I::Item:Display,
    {
        use std::fmt::Write;
        let start=self.strings.len();
        for elem in iter {
            let _=write!(self.strings,"{}",elem);
            self.strings.push_str(separator);
        }
        StartLen::new(start,self.strings.len()-start)
    }

    pub(crate) fn extend_with_lifetime_indices<I>(&mut self,iter:I)->LifetimeRange
    where
        I:IntoIterator<Item=LifetimeIndex>,
    {
        let start=self.lifetime_indices.len();
        self.lifetime_indices.extend(iter);
        let len=self.lifetime_indices.len()-start;

        if len <= 3 {
            let mut drainer=self.lifetime_indices.drain(start..);
            let li0=drainer.next().unwrap_or(LifetimeIndex::NONE);
            let li1=drainer.next().unwrap_or(LifetimeIndex::NONE);
            let li2=drainer.next().unwrap_or(LifetimeIndex::NONE);
            let array=LifetimeIndexArray::with_3(li0,li1,li2);
            LifetimeRange::with_array_length(array,len)
        }else{
            if (len&1)==1 {
                self.lifetime_indices.push(LifetimeIndex::NONE);
            }
            LifetimeRange::with_more_than_3( start..self.lifetime_indices.len() )
        }
    }

    pub(crate) fn push_type(&mut self,layout_ctor:LayoutConstructor,type_:&'a syn::Type)->u16{
        let type_layouts=&mut self.type_layouts;
        
        let key=(layout_ctor,type_);

        *self.type_layouts_map
            .entry(key)
            .or_insert_with(move||{
                let len=type_layouts.len();
                type_layouts.push(key);
                len as u16
            })
    }

    pub(crate) fn extend_type<I>(&mut self,layout_ctor:LayoutConstructor,types:I)->StartLen
    where
        I:IntoIterator<Item=&'a syn::Type>,
    {
        let start=self.type_layouts.len();
        for ty in types {
            self.type_layouts.push((layout_ctor,ty));
        }
        let end=self.type_layouts.len();
        StartLen::new(start,end-start)
    }

    pub fn get_type(&self,index:usize)->Option<&'a syn::Type>{
        self.type_layouts.get(index).map(|(_,ty)| *ty )
    }

    pub(crate) fn extend_with_constants<I>(&mut self,iter:I)->StartLen
    where
        I:IntoIterator<Item=&'a syn::Expr>,
    {
        let start=self.constants.len();
        for expr in iter {
            self.constants.push(expr);
        }
        let end=self.constants.len();
        StartLen::new(start,end-start)
    }
}

impl<'a> ToTokens for SharedVars<'a>{
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let ct=self.ctokens;
        let lifetime_indices=self.lifetime_indices
            .chunks(2)
            .map(|chunk|{
                let first=chunk[0];
                let second=chunk.get(1).map_or(LifetimeIndex::NONE,|x|*x);
                LifetimeIndexPair::new(first,second).bits
            })
            .piped(rslice_tokenizer);

        let strings= self.strings.as_str().piped(rstr_tokenizer);
        let type_layouts= self.type_layouts.iter()
            .map(|&(layout_ctor,ty)| make_get_type_layout_tokenizer(ty,layout_ctor,ct) )
            .piped(rslice_tokenizer);

        let constants=self.constants.iter()
            .map(|param| quote!( __ConstGeneric::new(&#param,__GetConstGenericVTable::VTABLE) ) )
            .piped(rslice_tokenizer);

        quote!(
            abi_stable::type_layout::SharedVars::new(
                #strings,
                #lifetime_indices,
                #type_layouts,
                #constants,
            )
        ).to_tokens(ts);
    }
}



#[must_use]
fn make_get_type_layout_tokenizer<'a,T:'a>(
    ty:T,
    field_transparency:LayoutConstructor,
    ct:&'a CommonTokens<'a>,
)->impl ToTokens+'a
where T:ToTokens
{
    ToTokenFnMut::new(move|ts|{
        to_stream!{ts; 
            ct.get_type_layout_ctor,
            ct.colon2,
            ct.lt,ty,ct.gt,
            ct.colon2,
        };
        match field_transparency {
            LayoutConstructor::Regular=> &ct.cap_stable_abi,
            LayoutConstructor::SharedStableAbi=> &ct.cap_shared_stable_abi,
            LayoutConstructor::Opaque=> &ct.cap_opaque_field,
            LayoutConstructor::SabiOpaque=> &ct.cap_sabi_opaque_field,
        }.to_tokens(ts);
    })
}
