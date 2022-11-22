#![allow(clippy::missing_const_for_fn)]

use std::{
    borrow::Borrow,
    cmp::{Eq, PartialEq},
    collections::hash_map::{Entry, HashMap},
    fmt::{self, Debug},
    hash::Hash,
    mem,
    ops::{Index, IndexMut},
    ptr,
};

use generational_arena::{Arena, Index as ArenaIndex};

use core_extensions::SelfOps;

/// A Map that maps multiple keys to the same value.
///
/// Every key maps to a value,which is stored at an index.
/// Indices can be used as a proxy for the value.
#[derive(Clone)]
pub struct MultiKeyMap<K, T> {
    map: HashMap<K, MapIndex>,
    arena: Arena<MapValue<K, T>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MapValue<K, T> {
    keys: Vec<K>,
    value: T,
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MapIndex {
    index: ArenaIndex,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IndexValue<T> {
    pub index: MapIndex,
    pub value: T,
}

/// When the element was inserted,now or before the method call.
#[must_use = "call `.into_inner()` to unwrap into the inner value."]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InsertionTime<T> {
    Now(T),
    Before(T),
}

impl<K, T> MultiKeyMap<K, T>
where
    K: Hash + Eq,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
            arena: Arena::new(),
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let &i = self.map.get(key)?;
        self.get_with_index(i)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut T>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let &i = self.map.get(key)?;
        self.get_mut_with_index(i)
    }

    #[allow(dead_code)]
    pub fn get2_mut<Q>(&mut self, key0: &Q, key1: &Q) -> (Option<&mut T>, Option<&mut T>)
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let i0 = self.map.get(key0).cloned();
        let i1 = self.map.get(key1).cloned();

        match (i0, i1) {
            (None, None) => (None, None),
            (Some(l), None) => (self.get_mut_with_index(l), None),
            (None, Some(r)) => (None, self.get_mut_with_index(r)),
            (Some(l), Some(r)) => self.get2_mut_with_index(l, r),
        }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<MapIndex>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.get(key).cloned()
    }

    pub fn get_with_index(&self, i: MapIndex) -> Option<&T> {
        self.arena.get(i.index).map(|x| &x.value)
    }

    pub fn get_mut_with_index(&mut self, i: MapIndex) -> Option<&mut T> {
        self.arena.get_mut(i.index).map(|x| &mut x.value)
    }

    pub fn get2_mut_with_index(
        &mut self,
        i0: MapIndex,
        i1: MapIndex,
    ) -> (Option<&mut T>, Option<&mut T>) {
        let (l, r) = self.arena.get2_mut(i0.index, i1.index);
        fn mapper<K, T>(x: &mut MapValue<K, T>) -> &mut T {
            &mut x.value
        }
        (l.map(mapper), r.map(mapper))
    }

    #[allow(dead_code)]
    pub fn replace_index(&mut self, replace: MapIndex, with: T) -> Option<T> {
        self.get_mut_with_index(replace)
            .map(|x| mem::replace(x, with))
    }

    #[allow(dead_code)]
    /// The amount of keys associated with values.
    pub fn key_len(&self) -> usize {
        self.map.len()
    }

    #[allow(dead_code)]
    /// The amount of values.
    pub fn value_len(&self) -> usize {
        self.arena.len()
    }

    /// Replaces the element at the `replace` index with the one at the `with` index,
    /// mapping all keys to the `with` index to the `replace` index.
    ///
    /// # Return value
    ///
    /// This method returns the previous value at `replace` if all these conditions are satisfied:
    ///
    /// - The `replace` index is not the same as the `with` index.
    ///
    /// - Both `replace` and `with` are valid indices
    ///
    /// If the conditions are not satisfied this method will return None without
    /// modifying the collection.
    ///
    pub fn replace_with_index(&mut self, replace: MapIndex, with: MapIndex) -> Option<T> {
        if replace == with
            || !self.arena.contains(replace.index)
            || !self.arena.contains(with.index)
        {
            return None;
        }
        let with_ = self.arena.remove(with.index)?;
        let replaced = self.arena.get_mut(replace.index)?;
        for key in &with_.keys {
            *self.map.get_mut(key).unwrap() = replace;
        }
        replaced.keys.extend(with_.keys);
        Some(mem::replace(&mut replaced.value, with_.value))
    }

    pub fn get_or_insert(&mut self, key: K, value: T) -> InsertionTime<IndexValue<&mut T>>
    where
        K: Clone,
    {
        match self.map.entry(key.clone()) {
            Entry::Occupied(entry) => {
                let index = *entry.get();
                InsertionTime::Before(IndexValue {
                    index,
                    value: &mut self.arena[index.index].value,
                })
            }
            Entry::Vacant(entry) => {
                let inserted = MapValue {
                    keys: vec![key],
                    value,
                };
                let index = MapIndex::new(self.arena.insert(inserted));
                entry.insert(index);
                // Just inserted the value at the index
                InsertionTime::Now(IndexValue {
                    index,
                    value: &mut self.arena.get_mut(index.index).unwrap().value,
                })
            }
        }
    }

    /// Associates `key` with the value at the `index`.
    /// If the `key` was already associated with a value,this will not do anything.
    ///
    /// # Panic
    ///
    /// Panics if the index is invalid.
    pub fn associate_key(&mut self, key: K, index: MapIndex)
    where
        K: Clone,
    {
        let value = match self.arena.get_mut(index.index) {
            Some(x) => x,
            None => panic!("Invalid index:{:?}", index),
        };
        match self.map.entry(key.clone()) {
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                entry.insert(index);
                value.keys.push(key);
            }
        }
    }

    /// Associates `key` with the value at the `index`.
    /// If `key` was already associated with a value,
    /// it will be disassociated from that value,
    /// returning that value if it has no keys associated with it.
    ///
    /// # Panic
    ///
    /// Panics if the index is invalid.
    ///
    /// # Index validity
    ///
    /// If `key` was associated with a value,and it was the only key for that value,
    /// the index for the value will be invalidated.
    #[allow(dead_code)]
    pub fn associate_key_forced(&mut self, key: K, index: MapIndex) -> Option<T>
    where
        K: Clone + ::std::fmt::Debug,
    {
        assert!(
            self.arena.contains(index.index),
            "Invalid index:{:?}",
            index,
        );
        let ret = match self.map.entry(key.clone()) {
            Entry::Occupied(mut entry) => {
                let index_before = *entry.get();
                entry.insert(index);
                let slot = &mut self.arena[index_before.index];
                let key_ind = slot.keys.iter().position(|x| *x == key).unwrap();
                slot.keys.swap_remove(key_ind);
                if slot.keys.is_empty() {
                    self.arena
                        .remove(index_before.index)
                        .unwrap()
                        .value
                        .piped(Some)
                } else {
                    None
                }
            }
            Entry::Vacant(_) => None,
        };
        let value = &mut self.arena[index.index];
        self.map.entry(key.clone()).or_insert(index);
        value.keys.push(key);
        ret
    }
}

impl<'a, K, Q: ?Sized, T> Index<&'a Q> for MultiKeyMap<K, T>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash,
{
    type Output = T;

    fn index(&self, index: &'a Q) -> &T {
        self.get(index).expect("no entry found for key")
    }
}

impl<'a, K, Q: ?Sized, T> IndexMut<&'a Q> for MultiKeyMap<K, T>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash,
{
    fn index_mut(&mut self, index: &'a Q) -> &mut T {
        self.get_mut(index).expect("no entry found for key")
    }
}

impl<K, T> Debug for MultiKeyMap<K, T>
where
    K: Eq + Hash + Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MultiKeyMap")
            .field("map", &self.map)
            .field("arena", &self.arena)
            .finish()
    }
}

impl<K, T> Eq for MultiKeyMap<K, T>
where
    K: Eq + Hash,
    T: Eq,
{
}

impl<K, T> PartialEq for MultiKeyMap<K, T>
where
    K: Eq + Hash,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.arena.len() != other.arena.len() || self.map.len() != other.map.len() {
            return false;
        }
        for (_, l_val) in self.arena.iter() {
            let mut keys = l_val.keys.iter();

            let r_val_index = match other.get_index(keys.next().unwrap()) {
                Some(x) => x,
                None => return false,
            };

            let r_val = &other.arena[r_val_index.index];

            if l_val.value != r_val.value {
                return false;
            }

            let all_map_to_r_val = keys.all(|key| match other.get_index(key) {
                Some(r_ind) => ptr::eq(r_val, &other.arena[r_ind.index]),
                None => false,
            });

            if !all_map_to_r_val {
                return false;
            }
        }
        true
    }
}

impl MapIndex {
    #[inline]
    fn new(index: ArenaIndex) -> Self {
        Self { index }
    }
}

impl<T> InsertionTime<T> {
    pub fn into_inner(self) -> T {
        match self {
            InsertionTime::Before(v) | InsertionTime::Now(v) => v,
        }
    }
    #[allow(dead_code)]
    pub fn split(self) -> (T, InsertionTime<()>) {
        let discr = self.discriminant();
        (self.into_inner(), discr)
    }
    #[allow(dead_code)]
    pub fn map<F, U>(self, f: F) -> InsertionTime<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            InsertionTime::Before(v) => InsertionTime::Before(f(v)),
            InsertionTime::Now(v) => InsertionTime::Now(f(v)),
        }
    }
    #[allow(dead_code)]
    pub fn discriminant(&self) -> InsertionTime<()> {
        match self {
            InsertionTime::Before { .. } => InsertionTime::Before(()),
            InsertionTime::Now { .. } => InsertionTime::Now(()),
        }
    }
}

#[cfg(all(test, not(feature = "only_new_tests")))]
mod tests {
    use super::*;

    use crate::test_utils::must_panic;

    #[test]
    fn equality() {
        fn insert(map: &mut MultiKeyMap<u32, u32>, key: u32, value: u32) {
            let index = map.get_or_insert(key, value).into_inner().index;
            map.associate_key(key + 1, index);
            map.associate_key(key + 2, index);
        }

        ///////////////////////////////////////////////////
        ////                EQUAL
        ///////////////////////////////////////////////////
        {
            let map_a = MultiKeyMap::<u32, u32>::new();

            let map_b = MultiKeyMap::<u32, u32>::new();

            assert_eq!(map_a, map_b);
        }
        {
            let mut map_a = MultiKeyMap::<u32, u32>::new();
            insert(&mut map_a, 1000, 200);

            let mut map_b = MultiKeyMap::<u32, u32>::new();
            insert(&mut map_b, 1000, 200);

            assert_eq!(map_a, map_b);
        }
        {
            let mut map_a = MultiKeyMap::<u32, u32>::new();
            insert(&mut map_a, 1000, 200);
            insert(&mut map_a, 2000, 400);

            let mut map_b = MultiKeyMap::<u32, u32>::new();
            insert(&mut map_b, 1000, 200);
            insert(&mut map_b, 2000, 400);

            assert_eq!(map_a, map_b);
        }

        ///////////////////////////////////////////////////
        ////             NOT EQUAL
        ///////////////////////////////////////////////////
        {
            let map_a = MultiKeyMap::<u32, u32>::new();

            let mut map_b = MultiKeyMap::<u32, u32>::new();
            insert(&mut map_b, 1000, 200);

            assert_ne!(map_a, map_b);
        }
        {
            let mut map_a = MultiKeyMap::<u32, u32>::new();
            insert(&mut map_a, 1000, 200);

            let map_b = MultiKeyMap::<u32, u32>::new();

            assert_ne!(map_a, map_b);
        }
        {
            let mut map_a = MultiKeyMap::<u32, u32>::new();
            insert(&mut map_a, 1000, 200);
            insert(&mut map_a, 2000, 401);

            let mut map_b = MultiKeyMap::<u32, u32>::new();
            insert(&mut map_b, 1000, 200);
            insert(&mut map_b, 2000, 400);

            assert_ne!(map_a, map_b);
        }
    }

    #[test]
    fn get_or_insert() {
        let mut map = MultiKeyMap::<u32, u32>::new();

        let (ret, ret_discr) = map.get_or_insert(10, 1).split();
        *ret.value = 1234;
        assert_matches!(ret_discr, InsertionTime::Now { .. });

        assert_matches!(
            map.get_or_insert(10, 2).map(|x| x.value).split(),
            (&mut 1234, InsertionTime::Before { .. })
        );
        assert_matches!(
            map.get_or_insert(10, 3).map(|x| x.value).split(),
            (&mut 1234, InsertionTime::Before { .. })
        );
    }

    #[test]
    fn associate_key() {
        let mut map = MultiKeyMap::<u32, u32>::new();

        let (ret, ret_discr) = map.get_or_insert(100, 1).split();
        let index0 = ret.index;
        *ret.value = 1234;
        assert_matches!(ret_discr, InsertionTime::Now { .. });

        let index1 = map.get_or_insert(200, 200).into_inner().index;
        let index2 = map.get_or_insert(300, 300).into_inner().index;

        map.associate_key(20, index0);
        map.associate_key(20, index1);
        map.associate_key(20, index2);
        assert_eq!(map[&20], 1234);

        map.associate_key(30, index0);
        map.associate_key(30, index1);
        map.associate_key(30, index2);
        assert_eq!(map[&30], 1234);

        map.associate_key(50, index2);
        map.associate_key(50, index0);
        map.associate_key(50, index1);
        assert_eq!(map[&50], 300);

        map[&100] = 456;
        assert_eq!(map[&20], 456);
        assert_eq!(map[&30], 456);
    }

    #[test]
    fn associate_key_forced() {
        let mut map = MultiKeyMap::<u32, u32>::new();

        let index0 = map.get_or_insert(100, 1000).into_inner().index;
        let index1 = map.get_or_insert(200, 2000).into_inner().index;
        let index2 = map.get_or_insert(300, 3000).into_inner().index;

        assert_eq!(map.associate_key_forced(20, index2), None);
        assert_eq!(map.associate_key_forced(20, index1), None);
        assert_eq!(map.associate_key_forced(20, index0), None);
        assert_eq!(map[&20], 1000);

        assert_eq!(map.associate_key_forced(30, index2), None);
        assert_eq!(map.associate_key_forced(30, index0), None);
        assert_eq!(map.associate_key_forced(30, index1), None);
        assert_eq!(map[&30], 2000);

        assert_eq!(map.associate_key_forced(50, index1), None);
        assert_eq!(map.associate_key_forced(50, index0), None);
        assert_eq!(map.associate_key_forced(50, index2), None);
        assert_eq!(map[&50], 3000);

        assert_eq!(map.associate_key_forced(100, index2), None);
        assert_eq!(map.associate_key_forced(20, index2), Some(1000));

        assert_eq!(map.associate_key_forced(200, index2), None);
        assert_eq!(map.associate_key_forced(30, index2), Some(2000));

        must_panic(|| map.associate_key_forced(100, index0)).unwrap();
        must_panic(|| map.associate_key_forced(200, index0)).unwrap();
        must_panic(|| map.associate_key_forced(20, index0)).unwrap();
        must_panic(|| map.associate_key_forced(30, index0)).unwrap();
    }

    #[test]
    fn replace_index() {
        let mut map = MultiKeyMap::<u32, u32>::new();

        let index0 = map.get_or_insert(1000, 200).into_inner().index;
        map.associate_key(1001, index0);
        map.associate_key(1002, index0);

        let index1 = map.get_or_insert(2000, 300).into_inner().index;
        map.associate_key(2001, index1);
        map.associate_key(2002, index1);

        let index2 = map.get_or_insert(3000, 400).into_inner().index;
        map.associate_key(3001, index2);
        map.associate_key(3002, index2);

        assert_eq!(map[&1000], 200);
        assert_eq!(map[&1001], 200);
        assert_eq!(map[&1001], 200);

        map.replace_index(index0, 205);
        assert_eq!(map[&1000], 205);
        assert_eq!(map[&1001], 205);
        assert_eq!(map[&1001], 205);

        map.replace_index(index1, 305);
        assert_eq!(map[&2000], 305);
        assert_eq!(map[&2001], 305);
        assert_eq!(map[&2001], 305);

        map.replace_index(index2, 405);
        assert_eq!(map[&3000], 405);
        assert_eq!(map[&3001], 405);
        assert_eq!(map[&3001], 405);
    }

    #[test]
    fn replace_with_index() {
        let mut map = MultiKeyMap::<u32, u32>::new();

        let index0 = map.get_or_insert(1000, 200).into_inner().index;
        map.associate_key(1001, index0);
        map.associate_key(1002, index0);

        let index1 = map.get_or_insert(2000, 300).into_inner().index;
        map.associate_key(2001, index1);
        map.associate_key(2002, index1);

        let index2 = map.get_or_insert(3000, 400).into_inner().index;
        map.associate_key(3001, index2);
        map.associate_key(3002, index2);

        map.replace_with_index(index0, index2);
        assert_eq!(map[&1000], 400);
        assert_eq!(map[&1001], 400);
        assert_eq!(map[&1002], 400);
        assert_eq!(map[&2000], 300);
        assert_eq!(map[&2001], 300);
        assert_eq!(map[&2002], 300);
        assert_eq!(map[&3000], 400);
        assert_eq!(map[&3001], 400);
        assert_eq!(map[&3002], 400);
        map[&1000] = 600;
        assert_eq!(map[&1000], 600);
        assert_eq!(map[&1001], 600);
        assert_eq!(map[&1002], 600);
        assert_eq!(map[&2000], 300);
        assert_eq!(map[&2001], 300);
        assert_eq!(map[&2002], 300);
        assert_eq!(map[&3000], 600);
        assert_eq!(map[&3001], 600);
        assert_eq!(map[&3002], 600);

        map.replace_with_index(index1, index0);
        map[&1000] = 800;
        assert_eq!(map[&1000], 800);
        assert_eq!(map[&1001], 800);
        assert_eq!(map[&1002], 800);
        assert_eq!(map[&2000], 800);
        assert_eq!(map[&2001], 800);
        assert_eq!(map[&2002], 800);
        assert_eq!(map[&3000], 800);
        assert_eq!(map[&3001], 800);
        assert_eq!(map[&3002], 800);
    }

    #[test]
    fn indexing() {
        let mut map = MultiKeyMap::<u32, u32>::new();

        let (index0, it0) = map.get_or_insert(1000, 200).map(|x| x.index).split();
        let (index1, it1) = map.get_or_insert(2000, 300).map(|x| x.index).split();
        let (index2, it2) = map.get_or_insert(3000, 400).map(|x| x.index).split();

        assert_eq!(it0, InsertionTime::Now(()));
        assert_eq!(it1, InsertionTime::Now(()));
        assert_eq!(it2, InsertionTime::Now(()));

        let expected = vec![
            (1000, index0, 200),
            (2000, index1, 300),
            (3000, index2, 400),
        ];
        #[allow(clippy::deref_addrof)]
        for (key, index, val) in expected {
            assert_eq!(*map.get_with_index(index).unwrap(), val);
            assert_eq!(*map.get(&key).unwrap(), val);
            assert_eq!(*(&map[&key]), val);

            assert_eq!(*map.get_mut(&key).unwrap(), val);
            assert_eq!(*map.get_mut_with_index(index).unwrap(), val);
            assert_eq!(*(&mut map[&key]), val);
        }
    }
}
