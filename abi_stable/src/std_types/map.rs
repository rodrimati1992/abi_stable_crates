//! Contains the ffi-safe equivalent of `std::collections::HashMap`, and related items.
#![allow(clippy::missing_const_for_fn)]

use std::{
    borrow::Borrow,
    cmp::{Eq, PartialEq},
    collections::{hash_map::RandomState, HashMap},
    fmt::{self, Debug},
    hash::{BuildHasher, Hash, Hasher},
    iter::FromIterator,
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::{
    erased_types::trait_objects::HasherObject,
    marker_type::{
        ErasedObject, ErasedPrefix, NonOwningPhantom, NotCopyNotClone, UnsafeIgnoredType,
    },
    pointer_trait::{AsMutPtr, AsPtr},
    prefix_type::{PrefixRef, WithMetadata},
    sabi_types::{RMut, RRef},
    std_types::*,
    traits::{ErasedType, IntoReprRust},
    DynTrait, StableAbi,
};

mod entry;
mod extern_fns;
mod iterator_stuff;
mod map_key;
mod map_query;

#[cfg(all(test, not(feature = "only_new_tests")))]
mod test;

use self::{entry::BoxedREntry, map_key::MapKey, map_query::MapQuery};

pub use self::{
    entry::{REntry, ROccupiedEntry, RVacantEntry},
    iterator_stuff::{IntoIter, MutIterInterface, RefIterInterface, ValIterInterface},
};

/// An ffi-safe hashmap, which wraps `std::collections::HashMap<K, V, S>`,
/// only requiring the `K: Eq + Hash` bounds when constructing it.
///
/// Most of the API in `HashMap` is implemented here, including the Entry API.
///
///
/// # Example
///
/// This example demonstrates how one can use the RHashMap as a dictionary.
///
/// ```
/// use abi_stable::std_types::{RHashMap, RSome, RString, Tuple2};
///
/// let mut map = RHashMap::new();
///
/// map.insert(
///     "dictionary",
///     "A book/document containing definitions of words",
/// );
/// map.insert("bibliophile", "Someone who loves books.");
/// map.insert("pictograph", "A picture representating of a word.");
///
/// assert_eq!(
///     map["dictionary"],
///     "A book/document containing definitions of words",
/// );
///
/// assert_eq!(map.remove("bibliophile"), RSome("Someone who loves books."),);
///
/// assert_eq!(
///     map.get("pictograph"),
///     Some(&"A picture representating of a word."),
/// );
///
/// for Tuple2(k, v) in map {
///     assert!(k == "dictionary" || k == "pictograph");
///
///     assert!(
///         v == "A book/document containing definitions of words" ||
///         v == "A picture representating of a word.",
///         "{} => {}",
///         k,
///         v,
///     );
/// }
///
///
/// ```
///
#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
pub struct RHashMap<K, V, S = RandomState> {
    map: RBox<ErasedMap<K, V, S>>,
    #[sabi(unsafe_change_type = VTable_Ref<K, V, S>)]
    vtable: PrefixRef<ErasedPrefix>,
}

///////////////////////////////////////////////////////////////////////////////

struct BoxedHashMap<'a, K, V, S> {
    map: HashMap<MapKey<K>, V, S>,
    entry: Option<BoxedREntry<'a, K, V>>,
}

/// An RHashMap iterator,
/// implementing `Iterator<Item= Tuple2< &K, &V > > + !Send + !Sync + Clone`
pub type Iter<'a, K, V> = DynTrait<'a, RBox<()>, RefIterInterface<K, V>>;

/// An RHashMap iterator,
/// implementing `Iterator<Item= Tuple2< &K, &mut V > > + !Send + !Sync`
pub type IterMut<'a, K, V> = DynTrait<'a, RBox<()>, MutIterInterface<K, V>>;

/// An RHashMap iterator,
/// implementing `Iterator<Item= Tuple2< K, V > > + !Send + !Sync`
pub type Drain<'a, K, V> = DynTrait<'a, RBox<()>, ValIterInterface<K, V>>;

/// Used as the erased type of the RHashMap type.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
struct ErasedMap<K, V, S>(PhantomData<(K, V)>, UnsafeIgnoredType<S>);

impl<'a, K: 'a, V: 'a, S: 'a> ErasedType<'a> for ErasedMap<K, V, S> {
    type Unerased = BoxedHashMap<'a, K, V, S>;
}

///////////////////////////////////////////////////////////////////////////////

impl<K, V> RHashMap<K, V, RandomState> {
    /// Constructs an empty RHashMap.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RString};
    ///
    /// let mut map = RHashMap::<RString, u32>::new();
    /// assert!(map.is_empty());
    /// map.insert("Hello".into(), 10);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// ```
    #[inline]
    pub fn new() -> RHashMap<K, V>
    where
        Self: Default,
    {
        Self::default()
    }

    /// Constructs an empty RHashMap with at least the passed capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RString};
    ///
    /// let mut map = RHashMap::<RString, u32>::with_capacity(10);
    /// assert!(map.capacity() >= 10);
    ///
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> RHashMap<K, V>
    where
        Self: Default,
    {
        let mut this = Self::default();
        this.reserve(capacity);
        this
    }
}

impl<K, V, S> RHashMap<K, V, S> {
    /// Constructs an empty RHashMap with the passed `hash_builder` to hash the keys.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RString};
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let mut map = RHashMap::<RString, u32, _>::with_hasher(s);
    /// assert!(map.is_empty());
    /// map.insert("Hello".into(), 10);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// ```
    #[inline]
    pub fn with_hasher(hash_builder: S) -> RHashMap<K, V, S>
    where
        K: Eq + Hash,
        S: BuildHasher + Default,
    {
        Self::with_capacity_and_hasher(0, hash_builder)
    }
    /// Constructs an empty RHashMap with at least the passed capacity,
    /// and the passed `hash_builder` to hash the keys.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RString};
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let mut map = RHashMap::<RString, u32, _>::with_capacity_and_hasher(10, s);
    /// assert!(map.capacity() >= 10);
    ///
    /// ```
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> RHashMap<K, V, S>
    where
        K: Eq + Hash,
        S: BuildHasher + Default,
    {
        let mut map = VTable::<K, V, S>::erased_map(hash_builder);
        unsafe {
            ErasedMap::reserve(map.as_rmut(), capacity);

            RHashMap {
                map,
                vtable: VTable::<K, V, S>::VTABLE_REF.0.cast(),
            }
        }
    }
}

impl<K, V, S> RHashMap<K, V, S> {
    fn vtable(&self) -> VTable_Ref<K, V, S> {
        unsafe { VTable_Ref::<K, V, S>(self.vtable.cast()) }
    }
}

impl<K, V, S> RHashMap<K, V, S> {
    /// Returns whether the map associates a value with the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RString};
    ///
    /// let mut map = RHashMap::<RString, u32>::new();
    /// assert_eq!(map.contains_key("boo"), false);
    /// map.insert("boo".into(), 0);
    /// assert_eq!(map.contains_key("boo"), true);
    ///
    /// ```
    pub fn contains_key<Q>(&self, query: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(query).is_some()
    }

    /// Returns a reference to the value associated with the key.
    ///
    /// Returns a `None` if there is no entry for the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RString};
    ///
    /// let mut map = RHashMap::<RString, u32>::new();
    /// assert_eq!(map.get("boo"), None);
    /// map.insert("boo".into(), 0);
    /// assert_eq!(map.get("boo"), Some(&0));
    ///
    /// ```
    pub fn get<Q>(&self, query: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let vtable = self.vtable();
        unsafe { vtable.get_elem()(self.map.as_rref(), MapQuery::new(&query)) }
    }

    /// Returns a mutable reference to the value associated with the key.
    ///
    /// Returns a `None` if there is no entry for the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RString};
    ///
    /// let mut map = RHashMap::<RString, u32>::new();
    /// assert_eq!(map.get_mut("boo"), None);
    /// map.insert("boo".into(), 0);
    /// assert_eq!(map.get_mut("boo"), Some(&mut 0));
    ///
    /// ```
    pub fn get_mut<Q>(&mut self, query: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let vtable = self.vtable();
        unsafe { vtable.get_mut_elem()(self.map.as_rmut(), MapQuery::new(&query)) }
    }

    /// Removes the value associated with the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RSome, RNone};
    ///
    /// let mut map = vec![(0, 1), (3, 4)].into_iter().collect::<RHashMap<u32, u32>>();
    ///
    /// assert_eq!(map.remove(&0), RSome(1));
    /// assert_eq!(map.remove(&0), RNone);
    ///
    /// assert_eq!(map.remove(&3), RSome(4));
    /// assert_eq!(map.remove(&3), RNone);
    ///
    /// ```
    pub fn remove<Q>(&mut self, query: &Q) -> ROption<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.remove_entry(query).map(|x| x.1)
    }

    /// Removes the entry for the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RSome, RNone, Tuple2};
    ///
    /// let mut map = vec![(0, 1), (3, 4)].into_iter().collect::<RHashMap<u32, u32>>();
    ///
    /// assert_eq!(map.remove_entry(&0), RSome(Tuple2(0, 1)));
    /// assert_eq!(map.remove_entry(&0), RNone);
    ///
    /// assert_eq!(map.remove_entry(&3), RSome(Tuple2(3, 4)));
    /// assert_eq!(map.remove_entry(&3), RNone);
    ///
    /// ```
    pub fn remove_entry<Q>(&mut self, query: &Q) -> ROption<Tuple2<K, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let vtable = self.vtable();
        unsafe { vtable.remove_entry()(self.map.as_rmut(), MapQuery::new(&query)) }
    }
}

impl<K, V, S> RHashMap<K, V, S> {
    /// Returns whether the map associates a value with the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    /// assert_eq!(map.contains_key(&11), false);
    /// map.insert(11, 0);
    /// assert_eq!(map.contains_key(&11), true);
    ///
    /// ```
    pub fn contains_key_p(&self, key: &K) -> bool {
        self.get_p(key).is_some()
    }

    /// Returns a reference to the value associated with the key.
    ///
    /// Returns a `None` if there is no entry for the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    /// assert_eq!(map.get(&12), None);
    /// map.insert(12, 0);
    /// assert_eq!(map.get(&12), Some(&0));
    ///
    /// ```
    pub fn get_p(&self, key: &K) -> Option<&V> {
        let vtable = self.vtable();
        unsafe { vtable.get_elem_p()(self.map.as_rref(), key) }
    }

    /// Returns a mutable reference to the value associated with the key.
    ///
    /// Returns a `None` if there is no entry for the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    /// assert_eq!(map.get_mut(&12), None);
    /// map.insert(12, 0);
    /// assert_eq!(map.get_mut(&12), Some(&mut 0));
    ///
    /// ```
    pub fn get_mut_p(&mut self, key: &K) -> Option<&mut V> {
        let vtable = self.vtable();
        unsafe { vtable.get_mut_elem_p()(self.map.as_rmut(), key) }
    }

    /// Removes the value associated with the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RSome, RNone};
    ///
    /// let mut map = vec![(0, 1), (3, 4)].into_iter().collect::<RHashMap<u32, u32>>();
    ///
    /// assert_eq!(map.remove_p(&0), RSome(1));
    /// assert_eq!(map.remove_p(&0), RNone);
    ///
    /// assert_eq!(map.remove_p(&3), RSome(4));
    /// assert_eq!(map.remove_p(&3), RNone);
    ///
    /// ```
    pub fn remove_p(&mut self, key: &K) -> ROption<V> {
        self.remove_entry_p(key).map(|x| x.1)
    }

    /// Removes the entry for the key.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RSome, RNone, Tuple2};
    ///
    /// let mut map = vec![(0, 1), (3, 4)].into_iter().collect::<RHashMap<u32, u32>>();
    ///
    /// assert_eq!(map.remove_entry_p(&0), RSome(Tuple2(0, 1)));
    /// assert_eq!(map.remove_entry_p(&0), RNone);
    ///
    /// assert_eq!(map.remove_entry_p(&3), RSome(Tuple2(3, 4)));
    /// assert_eq!(map.remove_entry_p(&3), RNone);
    ///
    /// ```
    pub fn remove_entry_p(&mut self, key: &K) -> ROption<Tuple2<K, V>> {
        let vtable = self.vtable();
        unsafe { vtable.remove_entry_p()(self.map.as_rmut(), key) }
    }

    /// Returns a reference to the value associated with the key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not associated with a value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = vec![(0, 1), (3, 4)].into_iter().collect::<RHashMap<u32, u32>>();
    ///
    /// assert_eq!(map.index_p(&0), &1);
    /// assert_eq!(map.index_p(&3), &4);
    ///
    /// ```
    ///
    /// ```should_panic
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// assert_eq!(map.index_p(&0), &1);
    ///
    /// ```
    pub fn index_p(&self, key: &K) -> &V {
        self.get_p(key)
            .expect("no entry in RHashMap<_, _> found for key")
    }

    /// Returns a mutable reference to the value associated with the key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not associated with a value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = vec![(0, 1), (3, 4)].into_iter().collect::<RHashMap<u32, u32>>();
    ///
    /// assert_eq!(map.index_mut_p(&0), &mut 1);
    /// assert_eq!(map.index_mut_p(&3), &mut 4);
    ///
    /// ```
    ///
    /// ```should_panic
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// assert_eq!(map.index_mut_p(&0), &mut 1);
    ///
    /// ```
    pub fn index_mut_p(&mut self, key: &K) -> &mut V {
        self.get_mut_p(key)
            .expect("no entry in RHashMap<_, _> found for key")
    }

    //////////////////////////////////

    /// Inserts a value into the map, associating it with a key, returning the previous value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// map.insert(0, 1);
    /// map.insert(2, 3);
    ///
    /// assert_eq!(map[&0], 1);
    /// assert_eq!(map[&2], 3);
    ///
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> ROption<V> {
        let vtable = self.vtable();
        unsafe { vtable.insert_elem()(self.map.as_rmut(), key, value) }
    }

    /// Reserves enough space to insert `reserved` extra elements without reallocating.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    /// map.reserve(10);
    ///
    /// ```
    pub fn reserve(&mut self, reserved: usize) {
        let vtable = self.vtable();

        unsafe {
            vtable.reserve()(self.map.as_rmut(), reserved);
        }
    }

    /// Removes all the entries in the map.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = vec![(0, 1), (3, 4)].into_iter().collect::<RHashMap<u32, u32>>();
    ///
    /// assert_eq!(map.contains_key(&0), true);
    /// assert_eq!(map.contains_key(&3), true);
    ///
    /// map.clear();
    ///
    /// assert_eq!(map.contains_key(&0), false);
    /// assert_eq!(map.contains_key(&3), false);
    ///
    /// ```
    pub fn clear(&mut self) {
        let vtable = self.vtable();
        unsafe {
            vtable.clear_map()(self.map.as_rmut());
        }
    }

    /// Returns the amount of entries in the map.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// assert_eq!(map.len(), 0);
    /// map.insert(0, 1);
    /// assert_eq!(map.len(), 1);
    /// map.insert(2, 3);
    /// assert_eq!(map.len(), 2);
    ///
    /// ```
    pub fn len(&self) -> usize {
        let vtable = self.vtable();
        unsafe { vtable.len()(self.map.as_rref()) }
    }

    /// Returns the capacity of the map, the amount of elements it can store without reallocating.
    ///
    /// Note that this is a lower bound, since hash maps don't necessarily have an exact capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::with_capacity(4);
    ///
    /// assert!(map.capacity() >= 4);
    ///
    /// ```
    pub fn capacity(&self) -> usize {
        let vtable = self.vtable();
        unsafe { vtable.capacity()(self.map.as_rref()) }
    }

    /// Returns whether the map contains any entries.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// assert_eq!(map.is_empty(), true);
    /// map.insert(0, 1);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterates over the entries in the map, with references to the values in the map.
    ///
    /// This returns a type that implements
    /// `Iterator<Item= Tuple2< &K, &V > > + !Send + !Sync + Clone`
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, Tuple2};
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// map.insert(0, 1);
    /// map.insert(3, 4);
    ///
    /// let mut list = map.iter().collect::<Vec<_>>();
    /// list.sort();
    /// assert_eq!( list, vec![Tuple2(&0, &1), Tuple2(&3, &4)] );
    ///
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V> {
        let vtable = self.vtable();

        unsafe { vtable.iter()(self.map.as_rref()) }
    }

    /// Iterates over the entries in the map, with mutable references to the values in the map.
    ///
    /// This returns a type that implements
    /// `Iterator<Item= Tuple2< &K, &mut V > > + !Send + !Sync`
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, Tuple2};
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// map.insert(0, 1);
    /// map.insert(3, 4);
    ///
    /// let mut list = map.iter_mut().collect::<Vec<_>>();
    /// list.sort();
    /// assert_eq!( list, vec![Tuple2(&0, &mut 1), Tuple2(&3, &mut  4)] );
    ///
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        let vtable = self.vtable();

        unsafe { vtable.iter_mut()(self.map.as_rmut()) }
    }

    /// Clears the map, returning an iterator over all the entries that were removed.
    ///
    /// This returns a type that implements `Iterator<Item= Tuple2< K, V > > + !Send + !Sync`
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, Tuple2};
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// map.insert(0, 1);
    /// map.insert(3, 4);
    ///
    /// let mut list = map.drain().collect::<Vec<_>>();
    /// list.sort();
    /// assert_eq!( list, vec![Tuple2(0, 1), Tuple2(3, 4)] );
    ///
    /// assert!(map.is_empty());
    ///
    /// ```
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        let vtable = self.vtable();

        unsafe { vtable.drain()(self.map.as_rmut()) }
    }

    /// Gets a handle into the entry in the map for the key,
    /// that allows operating directly on the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// // Inserting an entry that wasn't there before.
    /// {
    ///     let mut entry = map.entry(0);
    ///     assert_eq!(entry.get(), None);
    ///     assert_eq!(entry.or_insert(3), &mut 3);
    ///     assert_eq!(map.get(&0), Some(&3));
    /// }
    ///
    ///
    /// ```
    ///
    pub fn entry(&mut self, key: K) -> REntry<'_, K, V> {
        let vtable = self.vtable();

        unsafe { vtable.entry()(self.map.as_rmut(), key) }
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for key in map.keys() {
    ///     println!("{}", key);
    /// }
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { inner: self.iter() }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for val in map.values() {
    ///     println!("{}", val);
    /// }
    /// ```
    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }
}

/// An iterator over the keys of a `RHashMap`.
///
/// This `struct` is created by the [`keys`] method on [`RHashMap`]. See its
/// documentation for more.
///
/// [`keys`]: RHashMap::keys
///
/// # Example
///
/// ```
/// use abi_stable::std_types::RHashMap;
///
/// let mut map = RHashMap::new();
/// map.insert("a", 1);
/// let iter_keys = map.keys();
/// ```
#[repr(C)]
#[derive(StableAbi)]
pub struct Keys<'a, K: 'a, V: 'a> {
    inner: Iter<'a, K, V>,
}

// FIXME(#26925) Remove in favor of `#[derive(Clone)]`
impl<K, V> Clone for Keys<'_, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Keys {
            inner: self.inner.clone(),
        }
    }
}

impl<K: Debug, V> fmt::Debug for Keys<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<&'a K> {
        self.inner.next().map(|tuple| tuple.0)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// An iterator over the values of a `HashMap`.
///
/// This `struct` is created by the [`values`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`values`]: HashMap::values
///
/// # Example
///
/// ```
/// use abi_stable::std_types::RHashMap;
///
/// let mut map = RHashMap::new();
/// map.insert("a", 1);
/// let iter_values = map.values();
/// ```
#[repr(C)]
#[derive(StableAbi)]
pub struct Values<'a, K: 'a, V: 'a> {
    inner: Iter<'a, K, V>,
}

// FIXME(#26925) Remove in favor of `#[derive(Clone)]`
impl<K, V> Clone for Values<'_, K, V> {
    #[inline]
    fn clone(&self) -> Self {
        Values {
            inner: self.inner.clone(),
        }
    }
}

impl<K, V: Debug> fmt::Debug for Values<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<&'a V> {
        self.inner.next().map(|tuple| tuple.1)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// This returns an `Iterator<Item= Tuple2< K, V > >+!Send+!Sync`
impl<K, V, S> IntoIterator for RHashMap<K, V, S> {
    type Item = Tuple2<K, V>;
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> IntoIter<K, V> {
        let vtable = self.vtable();

        unsafe { vtable.iter_val()(self.map) }
    }
}

/// This returns an `Iterator<Item= Tuple2< &K, &V > > + !Send + !Sync + Clone`
impl<'a, K, V, S> IntoIterator for &'a RHashMap<K, V, S> {
    type Item = Tuple2<&'a K, &'a V>;
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// This returns a type that implements
/// `Iterator<Item= Tuple2< &K, &mut V > > + !Send + !Sync`
impl<'a, K, V, S> IntoIterator for &'a mut RHashMap<K, V, S> {
    type Item = Tuple2<&'a K, &'a mut V>;
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, S> From<HashMap<K, V, S>> for RHashMap<K, V, S>
where
    Self: Default,
{
    fn from(map: HashMap<K, V, S>) -> Self {
        map.into_iter().collect()
    }
}

impl<K, V, S> From<RHashMap<K, V, S>> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn from(this: RHashMap<K, V, S>) -> HashMap<K, V, S> {
        this.into_iter().map(|x| x.into_tuple()).collect()
    }
}

impl<K, V, S> FromIterator<(K, V)> for RHashMap<K, V, S>
where
    Self: Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}

impl<K, V, S> FromIterator<Tuple2<K, V>> for RHashMap<K, V, S>
where
    Self: Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Tuple2<K, V>>,
    {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}

impl<K, V, S> Extend<(K, V)> for RHashMap<K, V, S> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K, V, S> Extend<Tuple2<K, V>> for RHashMap<K, V, S> {
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Tuple2<K, V>>,
    {
        self.extend(iter.into_iter().map(Tuple2::into_rust));
    }
}

impl<K, V, S> Default for RHashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<K, V, S> Clone for RHashMap<K, V, S>
where
    K: Clone,
    V: Clone,
    Self: Default,
{
    fn clone(&self) -> Self {
        self.iter()
            .map(|Tuple2(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl<K, V, S> Debug for RHashMap<K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.iter().map(Tuple2::into_rust))
            .finish()
    }
}

impl<K, V, S> Eq for RHashMap<K, V, S>
where
    K: Eq,
    V: Eq,
{
}

impl<K, V, S> PartialEq for RHashMap<K, V, S>
where
    K: PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|Tuple2(k, vl)| other.get_p(k).map_or(false, |vr| *vr == *vl))
    }
}

unsafe impl<K, V, S> Send for RHashMap<K, V, S> where HashMap<K, V, S>: Send {}

unsafe impl<K, V, S> Sync for RHashMap<K, V, S> where HashMap<K, V, S>: Sync {}

impl<K, Q, V, S> Index<&Q> for RHashMap<K, V, S>
where
    K: Borrow<Q>,
    Q: Eq + Hash + ?Sized,
{
    type Output = V;

    fn index(&self, query: &Q) -> &V {
        self.get(query)
            .expect("no entry in RHashMap<_, _> found for key")
    }
}

impl<K, Q, V, S> IndexMut<&Q> for RHashMap<K, V, S>
where
    K: Borrow<Q>,
    Q: Eq + Hash + ?Sized,
{
    fn index_mut(&mut self, query: &Q) -> &mut V {
        self.get_mut(query)
            .expect("no entry in RHashMap<_, _> found for key")
    }
}

mod serde {
    use super::*;

    use ::serde::{
        de::{MapAccess, Visitor},
        ser::SerializeMap,
        Deserialize, Deserializer, Serialize, Serializer,
    };

    struct RHashMapVisitor<K, V, S> {
        _marker: NonOwningPhantom<RHashMap<K, V, S>>,
    }

    impl<K, V, S> RHashMapVisitor<K, V, S> {
        fn new() -> Self {
            RHashMapVisitor {
                _marker: NonOwningPhantom::NEW,
            }
        }
    }

    impl<'de, K, V, S> Visitor<'de> for RHashMapVisitor<K, V, S>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
        RHashMap<K, V, S>: Default,
    {
        type Value = RHashMap<K, V, S>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an RHashMap")
        }

        fn visit_map<M>(self, mut map_access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let capacity = map_access.size_hint().unwrap_or(0);
            let mut map = RHashMap::default();
            map.reserve(capacity);

            while let Some((k, v)) = map_access.next_entry()? {
                map.insert(k, v);
            }

            Ok(map)
        }
    }

    impl<'de, K, V, S> Deserialize<'de> for RHashMap<K, V, S>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
        Self: Default,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(RHashMapVisitor::new())
        }
    }

    impl<K, V, S> Serialize for RHashMap<K, V, S>
    where
        K: Serialize,
        V: Serialize,
    {
        fn serialize<Z>(&self, serializer: Z) -> Result<Z::Ok, Z::Error>
        where
            Z: Serializer,
        {
            let mut map = serializer.serialize_map(Some(self.len()))?;
            for Tuple2(k, v) in self.iter() {
                map.serialize_entry(k, v)?;
            }
            map.end()
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    kind(Prefix),
    missing_field(panic),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
    //debug_print,
)]
struct VTable<K, V, S> {
    ///
    insert_elem: unsafe extern "C" fn(RMut<'_, ErasedMap<K, V, S>>, K, V) -> ROption<V>,

    get_elem: for<'a> unsafe extern "C" fn(
        RRef<'a, ErasedMap<K, V, S>>,
        MapQuery<'_, K>,
    ) -> Option<&'a V>,
    get_mut_elem: for<'a> unsafe extern "C" fn(
        RMut<'a, ErasedMap<K, V, S>>,
        MapQuery<'_, K>,
    ) -> Option<&'a mut V>,
    remove_entry: unsafe extern "C" fn(
        RMut<'_, ErasedMap<K, V, S>>,
        MapQuery<'_, K>,
    ) -> ROption<Tuple2<K, V>>,

    get_elem_p: for<'a> unsafe extern "C" fn(RRef<'a, ErasedMap<K, V, S>>, &K) -> Option<&'a V>,
    get_mut_elem_p:
        for<'a> unsafe extern "C" fn(RMut<'a, ErasedMap<K, V, S>>, &K) -> Option<&'a mut V>,
    remove_entry_p: unsafe extern "C" fn(RMut<'_, ErasedMap<K, V, S>>, &K) -> ROption<Tuple2<K, V>>,

    reserve: unsafe extern "C" fn(RMut<'_, ErasedMap<K, V, S>>, usize),
    clear_map: unsafe extern "C" fn(RMut<'_, ErasedMap<K, V, S>>),
    len: unsafe extern "C" fn(RRef<'_, ErasedMap<K, V, S>>) -> usize,
    capacity: unsafe extern "C" fn(RRef<'_, ErasedMap<K, V, S>>) -> usize,
    iter: unsafe extern "C" fn(RRef<'_, ErasedMap<K, V, S>>) -> Iter<'_, K, V>,
    iter_mut: unsafe extern "C" fn(RMut<'_, ErasedMap<K, V, S>>) -> IterMut<'_, K, V>,
    drain: unsafe extern "C" fn(RMut<'_, ErasedMap<K, V, S>>) -> Drain<'_, K, V>,
    iter_val: unsafe extern "C" fn(RBox<ErasedMap<K, V, S>>) -> IntoIter<K, V>,
    #[sabi(last_prefix_field)]
    entry: unsafe extern "C" fn(RMut<'_, ErasedMap<K, V, S>>, K) -> REntry<'_, K, V>,
}

impl<K, V, S> VTable<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    const VTABLE_VAL: WithMetadata<VTable<K, V, S>> = WithMetadata::new(Self::VTABLE);

    const VTABLE_REF: VTable_Ref<K, V, S> = unsafe { VTable_Ref(Self::VTABLE_VAL.as_prefix()) };

    fn erased_map(hash_builder: S) -> RBox<ErasedMap<K, V, S>> {
        unsafe {
            let map = HashMap::<MapKey<K>, V, S>::with_hasher(hash_builder);
            let boxed = BoxedHashMap { map, entry: None };
            let boxed = RBox::new(boxed);
            mem::transmute::<RBox<_>, RBox<ErasedMap<K, V, S>>>(boxed)
        }
    }

    const VTABLE: VTable<K, V, S> = VTable {
        insert_elem: ErasedMap::insert_elem,

        get_elem: ErasedMap::get_elem,
        get_mut_elem: ErasedMap::get_mut_elem,
        remove_entry: ErasedMap::remove_entry,

        get_elem_p: ErasedMap::get_elem_p,
        get_mut_elem_p: ErasedMap::get_mut_elem_p,
        remove_entry_p: ErasedMap::remove_entry_p,

        reserve: ErasedMap::reserve,
        clear_map: ErasedMap::clear_map,
        len: ErasedMap::len,
        capacity: ErasedMap::capacity,
        iter: ErasedMap::iter,
        iter_mut: ErasedMap::iter_mut,
        drain: ErasedMap::drain,
        iter_val: ErasedMap::iter_val,
        entry: ErasedMap::entry,
    };
}

///////////////////////////////////////////////////////////////////////////////
