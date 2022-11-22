use super::*;

use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    rc::Rc,
};

/// This is a map from type parameters to some value.
///
/// If you put this in a type,and use Default to initialize it,
/// you must remember to replace the `TypeParamMap` using either
/// `TypeParamMap::defaulted` or `TypeParamMap::with`
///
#[derive(Default, Clone, Debug, PartialEq)]
pub struct TypeParamMap<'a, T> {
    idents: Rc<Idents<'a>>,
    ty_params: Vec<T>,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct Idents<'a> {
    map: HashMap<&'a Ident, usize>,
    list: Vec<&'a Ident>,
}

#[allow(dead_code)]
impl<'a, T> TypeParamMap<'a, T> {
    /// Constructs an TypeParamMap which maps each type parameter in the DataStructure
    /// to the default value for `T`.
    pub fn defaulted(ds: &'a DataStructure<'a>) -> Self
    where
        T: Default,
    {
        Self::with(ds, |_, _| T::default())
    }

    /// Constructs an TypeParamMap which maps each type parameter
    /// in the DataStructure to a value
    /// (obtained by mapping each individual type parameter to `T` using a closure).
    pub fn with<F>(ds: &'a DataStructure<'a>, mut f: F) -> Self
    where
        F: FnMut(usize, &'a Ident) -> T,
    {
        let type_param_count = ds.generics.type_params().count();

        let mut idents = Idents {
            map: HashMap::<&'a Ident, usize>::with_capacity(type_param_count),
            list: Vec::<&'a Ident>::with_capacity(type_param_count),
        };
        let mut ty_params = Vec::<T>::with_capacity(type_param_count);

        for (i, type_param) in ds.generics.type_params().enumerate() {
            let ident = &type_param.ident;
            idents.map.insert(ident, i);
            idents.list.push(ident);

            ty_params.push(f(i, ident));
        }
        Self {
            idents: Rc::new(idents),
            ty_params,
        }
    }

    /// Clones this map into a map of empty tuples.
    ///
    /// This operation is cheap,it only clones a reference counted pointer,
    /// and constructs a Vec of empty tuples(which is just a fancy counter).
    ///
    pub fn clone_unit(&self) -> TypeParamMap<'a, ()> {
        TypeParamMap {
            ty_params: vec![(); self.ty_params.len()],
            idents: self.idents.clone(),
        }
    }

    /// Maps each value in the map to another one,using a closure.
    pub fn map<F, U>(self, mut f: F) -> TypeParamMap<'a, U>
    where
        F: FnMut(usize, &'a Ident, T) -> U,
    {
        TypeParamMap {
            ty_params: self
                .ty_params
                .into_iter()
                .enumerate()
                .zip(&self.idents.list)
                .map(|((i, elem), ident)| f(i, ident, elem))
                .collect(),
            idents: self.idents,
        }
    }

    pub fn len(&self) -> usize {
        self.ty_params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ty_params.is_empty()
    }

    pub fn get_index(&self, ident: &Ident) -> Result<usize, syn::Error> {
        match self.idents.map.get(ident) {
            Some(x) => Ok(*x),
            None => Err(spanned_err!(
                ident,
                "identifier for an invalid type parameter"
            )),
        }
    }

    pub fn get<I>(&self, index: I) -> Result<&T, syn::Error>
    where
        Self: Getter<I, Elem = T>,
    {
        self.get_inner(index)
    }

    pub fn get_mut<I>(&mut self, index: I) -> Result<&mut T, syn::Error>
    where
        Self: Getter<I, Elem = T>,
    {
        self.get_mut_inner(index)
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = (&'a Ident, &'_ T)> + '_ {
        self.idents.list.iter().cloned().zip(self.ty_params.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&'a Ident, &'_ mut T)> + '_ {
        self.idents
            .list
            .iter()
            .cloned()
            .zip(self.ty_params.iter_mut())
    }
}

impl<'a, T> Index<usize> for TypeParamMap<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        &self.ty_params[index]
    }
}

impl<'a, T> IndexMut<usize> for TypeParamMap<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.ty_params[index]
    }
}

impl<'a, T> Index<&Ident> for TypeParamMap<'a, T> {
    type Output = T;

    fn index(&self, ident: &Ident) -> &T {
        let index = self.idents.map[ident];
        &self.ty_params[index]
    }
}

impl<'a, T> IndexMut<&Ident> for TypeParamMap<'a, T> {
    fn index_mut(&mut self, ident: &Ident) -> &mut T {
        let index = self.idents.map[ident];
        &mut self.ty_params[index]
    }
}

pub trait Getter<Index> {
    type Elem;

    fn get_inner(&self, index: Index) -> Result<&Self::Elem, syn::Error>;
    fn get_mut_inner(&mut self, index: Index) -> Result<&mut Self::Elem, syn::Error>;
}

impl<'a, T> Getter<usize> for TypeParamMap<'a, T> {
    type Elem = T;

    fn get_inner(&self, index: usize) -> Result<&Self::Elem, syn::Error> {
        self.ty_params
            .get(index)
            .ok_or_else(|| err_from_index(index, self.len()))
    }
    fn get_mut_inner(&mut self, index: usize) -> Result<&mut Self::Elem, syn::Error> {
        let len = self.len();
        self.ty_params
            .get_mut(index)
            .ok_or_else(|| err_from_index(index, len))
    }
}

impl<'a, T> Getter<&Ident> for TypeParamMap<'a, T> {
    type Elem = T;

    fn get_inner(&self, ident: &Ident) -> Result<&Self::Elem, syn::Error> {
        let index = self.get_index(ident)?;
        Ok(&self.ty_params[index])
    }
    fn get_mut_inner(&mut self, ident: &Ident) -> Result<&mut Self::Elem, syn::Error> {
        let index = self.get_index(ident)?;
        Ok(&mut self.ty_params[index])
    }
}

fn err_from_index(index: usize, len: usize) -> syn::Error {
    syn_err!(
        proc_macro2::Span::call_site(),
        "index for type parameters is out of bounds, index={} len={}",
        index,
        len,
    )
}
