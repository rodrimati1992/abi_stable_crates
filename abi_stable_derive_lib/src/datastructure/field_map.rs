use super::*;

use std::{
    mem,
    ops::{Index,IndexMut},
};

#[derive(Default,Clone, Debug, PartialEq, Hash)]
pub(crate) struct FieldMap<T> {
    fields:Vec<Vec<T>>,
}

impl<T> FieldMap<T>{
    pub(crate) fn empty()->Self{
        Self{
            fields:Vec::new(),
        }
    }

    pub(crate) fn defaulted<'a>(ds:&'a DataStructure<'a>)->Self
    where 
        T:Default
    {
        Self::with(ds,|_| T::default() )
    }
    
    pub(crate) fn with<'a,F>(ds:&'a DataStructure<'a>,mut f:F)->Self
    where
        F:FnMut(&'a Field<'a>)->T
    {
        Self{
            fields:ds.variants
                .iter()
                .map(|vari|{
                    vari.fields.iter().map(&mut f).collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        }
    }

    pub(crate) fn map<F,U>(self,mut f:F)->FieldMap<U>
    where
        F:FnMut(FieldIndex,T)->U
    {
        let fields=self.fields
            .into_iter()
            .enumerate()
            .map(|(var_i,variant)|{
                variant
                .into_iter()
                .enumerate()
                .map(|(pos,v)|{
                    let index=FieldIndex{
                        variant:var_i,
                        pos,
                    };
                    f(index,v)
                })
                .collect::<Vec<U>>()
            })
            .collect::<Vec<Vec<U>>>();
        FieldMap{fields}
    }

    pub(crate) fn insert(&mut self,field:&Field<'_>,value:T)->T{
        let index=field.index;
        mem::replace(&mut self[field], value)
    }
}


impl<'a,T> Index<&'a Field<'a>> for FieldMap<T>{
    type Output=T;

    fn index(&self,field:&'a Field<'a>)->&T{
        let index=field.index;
        &self.fields[index.variant][index.pos]
    }
}


impl<'a,T> IndexMut<&'a Field<'a>> for FieldMap<T>{
    fn index_mut(&mut self,field:&'a Field<'a>)->&mut T{
        let index=field.index;
        &mut self.fields[index.variant][index.pos]
    }
}