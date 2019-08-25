use super::*;

use std::{
    mem,
    ops::{Index,IndexMut},
};

/**
This is a map from fields to some value.

If you put this in a type,and use Default to initialize it,
you must remember to replace the `FieldMap` using either `FieldMap::defaulted` or `FieldMap::with`

*/
#[derive(Default,Clone, Debug, PartialEq, Hash)]
pub(crate) struct FieldMap<T> {
    // The outer vec is the enum variant (if it's a struct/union it's a single element Vec),
    // the inner one is the field within a variant/struct/union.
    fields:Vec<Vec<T>>,
}

impl<T> FieldMap<T>{
    /// Constructs an empty FieldMap.
    pub(crate) fn empty()->Self{
        Self{
            fields:Vec::new(),
        }
    }

    /// Constructs an FieldMap which maps each field in the DataStructure 
    /// to the default value for `T`.
    pub(crate) fn defaulted<'a>(ds:&'a DataStructure<'a>)->Self
    where 
        T:Default
    {
        Self::with(ds,|_| T::default() )
    }
    
    /// Constructs an FieldMap which maps each field in the DataStructure to a value
    /// (obtained by mapping each individual field to `T` using a closure).
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

    /// Maps each value in the map to another one,using a closure.
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

    /// Add a new field to the map along with a value that it maps into.
    pub(crate) fn insert(&mut self,field:&Field<'_>,value:T)->T{
        mem::replace(&mut self[field], value)
    }
}


impl<'a,T> Index<FieldIndex> for FieldMap<T>{
    type Output=T;

    fn index(&self,index:FieldIndex)->&T{
        &self.fields[index.variant][index.pos]
    }
}

impl<'a,T> IndexMut<FieldIndex> for FieldMap<T>{
    fn index_mut(&mut self,index:FieldIndex)->&mut T{
        &mut self.fields[index.variant][index.pos]
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