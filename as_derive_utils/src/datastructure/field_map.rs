use super::*;

use std::{
    mem,
    ops::{Index, IndexMut},
};

/// This is a map from fields to some value.
///
/// If you put this in a type,and use Default to initialize it,
/// you must remember to replace the `FieldMap` using either `FieldMap::defaulted` or
/// `FieldMap::with`
#[derive(Default, Clone, Debug, PartialEq, Hash)]
pub struct FieldMap<T> {
    // The outer vec is the enum variant (if it's a struct/union it's a single element Vec),
    // the inner one is the field within a variant/struct/union.
    fields: Vec<Vec<T>>,
}

impl<T> FieldMap<T> {
    /// Constructs an empty FieldMap.
    pub fn empty() -> Self {
        Self { fields: Vec::new() }
    }

    /// Constructs an FieldMap which maps each field in the DataStructure
    /// to the default value for `T`.
    pub fn defaulted<'a>(ds: &'a DataStructure<'a>) -> Self
    where
        T: Default,
    {
        Self::with(ds, |_| T::default())
    }

    /// Constructs an FieldMap which maps each field in the DataStructure to a value
    /// (obtained by mapping each individual field to `T` using a closure).
    pub fn with<'a, F>(ds: &'a DataStructure<'a>, mut f: F) -> Self
    where
        F: FnMut(&'a Field<'a>) -> T,
    {
        Self {
            fields: ds
                .variants
                .iter()
                .map(|vari| vari.fields.iter().map(&mut f).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        }
    }

    /// Maps each value in the map to another one,using a closure.
    pub fn map<F, U>(self, mut f: F) -> FieldMap<U>
    where
        F: FnMut(FieldIndex, T) -> U,
    {
        let fields = self
            .fields
            .into_iter()
            .enumerate()
            .map(|(var_i, variant)| {
                variant
                    .into_iter()
                    .enumerate()
                    .map(|(pos, v)| {
                        let index = FieldIndex {
                            variant: var_i,
                            pos,
                        };
                        f(index, v)
                    })
                    .collect::<Vec<U>>()
            })
            .collect::<Vec<Vec<U>>>();
        FieldMap { fields }
    }

    /// Whether the field index maps to a field.
    #[allow(dead_code)]
    pub fn contains_index(&self, index: FieldIndex) -> bool {
        self.fields
            .get(index.variant)
            .map_or(false, |variant| index.pos < variant.len())
    }

    /// Add a new field to the map along with a value that it maps into.
    pub fn insert(&mut self, field: &Field<'_>, value: T) -> T {
        mem::replace(&mut self[field], value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (FieldIndex, &'_ T)> + Clone + '_ {
        self.fields.iter().enumerate().flat_map(|(v_i, v)| {
            v.iter().enumerate().map(move |(f_i, f)| {
                let index = FieldIndex {
                    variant: v_i as _,
                    pos: f_i as _,
                };
                (index, f)
            })
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (FieldIndex, &'_ mut T)> + '_ {
        self.fields.iter_mut().enumerate().flat_map(|(v_i, v)| {
            v.iter_mut().enumerate().map(move |(f_i, f)| {
                let index = FieldIndex {
                    variant: v_i as _,
                    pos: f_i as _,
                };
                (index, f)
            })
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &'_ T> + Clone + '_ {
        self.fields.iter().flat_map(|v| v.iter())
    }
}

impl<T> Index<FieldIndex> for FieldMap<T> {
    type Output = T;

    fn index(&self, index: FieldIndex) -> &T {
        &self.fields[index.variant][index.pos]
    }
}

impl<T> IndexMut<FieldIndex> for FieldMap<T> {
    fn index_mut(&mut self, index: FieldIndex) -> &mut T {
        &mut self.fields[index.variant][index.pos]
    }
}

impl<'a, 'b, T> Index<&'a Field<'b>> for FieldMap<T> {
    type Output = T;

    fn index(&self, field: &'a Field<'b>) -> &T {
        let index = field.index;
        &self.fields[index.variant][index.pos]
    }
}

impl<'a, 'b, T> IndexMut<&'a Field<'b>> for FieldMap<T> {
    fn index_mut(&mut self, field: &'a Field<'b>) -> &mut T {
        let index = field.index;
        &mut self.fields[index.variant][index.pos]
    }
}
