use super::*;

use std::{hash::Hash, mem::ManuallyDrop, ptr};

use crate::{
    marker_type::UnsafeIgnoredType,
    prefix_type::{PrefixTypeTrait, WithMetadata},
    sabi_types::{RMut, RRef},
};

/// The enum stored alongside the unerased HashMap.
pub(super) enum BoxedRRawEntryMut<'a, K, V, S> {
    Occupied(UnerasedRawOccupiedEntryMut<'a, K, V, S>),
    Vacant(UnerasedRawVacantEntryMut<'a, K, V, S>),
}

/// A handle into an entry in a map, which is either vacant or occupied.
#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    bound(K: 'a),
    bound(V: 'a),
    bound(S: 'a),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
pub enum RRawEntryMut<'a, K, V, S> {
    Occupied(RRawOccupiedEntryMut<'a, K, V, S>),
    Vacant(RRawVacantEntryMut<'a, K, V, S>),
}

/////////////////////////////////////////////////////////////////////////////////////////////

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
struct ErasedRawOccupiedEntryMut<K, V, S>(PhantomData<(K, V)>, UnsafeIgnoredType<S>);

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
struct ErasedRawVacantEntryMut<K, V, S>(PhantomData<(K, V)>, UnsafeIgnoredType<S>);

type UnerasedRawOccupiedEntryMut<'a, K, V, S> =
    ManuallyDrop<RawOccupiedEntryMut<'a, MapKey<K>, V, S>>;

type UnerasedRawVacantEntryMut<'a, K, V, S> = ManuallyDrop<RawVacantEntryMut<'a, MapKey<K>, V, S>>;

impl<'a, K: 'a, V: 'a, S: 'a> ErasedType<'a> for ErasedRawOccupiedEntryMut<K, V, S> {
    type Unerased = UnerasedRawOccupiedEntryMut<'a, K, V, S>;
}

impl<'a, K: 'a, V: 'a, S: 'a> ErasedType<'a> for ErasedRawVacantEntryMut<K, V, S> {
    type Unerased = UnerasedRawVacantEntryMut<'a, K, V, S>;
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a, K, V, S> From<RawEntryMut<'a, MapKey<K>, V, S>> for BoxedRRawEntryMut<'a, K, V, S>
where
    K: Eq + Hash,
{
    fn from(entry: RawEntryMut<'a, MapKey<K>, V, S>) -> Self {
        match entry {
            RawEntryMut::Occupied(entry) => entry
                .piped(ManuallyDrop::new)
                .piped(BoxedRRawEntryMut::Occupied),
            RawEntryMut::Vacant(entry) => entry
                .piped(ManuallyDrop::new)
                .piped(BoxedRRawEntryMut::Vacant),
        }
    }
}

impl<'a, K, V, S: BuildHasher> RRawEntryMut<'a, K, V, S>
where
    K: Eq + Hash,
{
    pub(super) unsafe fn new(entry: &'a mut BoxedRRawEntryMut<'a, K, V, S>) -> Self {
        match entry {
            BoxedRRawEntryMut::Occupied(entry) => entry
                .piped(RRawOccupiedEntryMut::new)
                .piped(RRawEntryMut::Occupied),
            BoxedRRawEntryMut::Vacant(entry) => entry
                .piped(RRawVacantEntryMut::new)
                .piped(RRawEntryMut::Vacant),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a, K, V, S: BuildHasher> RRawEntryMut<'a, K, V, S> {
    /*
    /// Inserts `default` as the value in the entry if it wasn't occupied,
    /// returning a mutable reference to the value in the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// assert_eq!(map.entry(0).or_insert(100), &mut 100);
    ///
    /// assert_eq!(map.entry(0).or_insert(400), &mut 100);
    ///
    /// ```
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            RRawEntryMut::Occupied(entry) => entry.into_mut(),
            RRawEntryMut::Vacant(entry) => entry.insert(default),
        }
    }
    */

    /// Inserts `default()` as the value in the entry if it wasn't occupied,
    /// returning a mutable reference to the value in the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RString};
    ///
    /// let mut map = RHashMap::<u32, RString>::new();
    ///
    /// assert_eq!(
    ///     map.entry(0).or_insert_with(|| "foo".into()),
    ///     &mut RString::from("foo")
    /// );
    ///
    /// assert_eq!(
    ///     map.entry(0).or_insert_with(|| "bar".into()),
    ///     &mut RString::from("foo")
    /// );
    ///
    /// ```
    pub fn or_insert_with<F>(self, default: F) -> (&'a mut K, &'a mut V)
    where
        F: FnOnce() -> (K, V),
        K: Hash,
        S: BuildHasher,
    {
        match self {
            RRawEntryMut::Occupied(entry) => entry.into_key_value(),
            RRawEntryMut::Vacant(entry) => {
                let (key, value) = default();
                entry.insert(key, value)
            }
        }
    }

    /*
    /// Allows mutating an occupied entry before doing other operations.
    ///
    /// This is a no-op on a vacant entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap, RString};
    ///
    /// let mut map = RHashMap::<RString, RString>::new();
    /// map.insert("foo".into(), "bar".into());
    ///
    /// assert_eq!(
    ///     map.entry("foo".into())
    ///         .and_modify(|x| x.push_str("hoo"))
    ///         .get(),
    ///     Some(&RString::from("barhoo"))
    /// );
    /// ```
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            RRawEntryMut::Occupied(mut entry) => {
                f(entry.get_mut());
                RRawEntryMut::Occupied(entry)
            }
            RRawEntryMut::Vacant(entry) => RRawEntryMut::Vacant(entry),
        }
    }
    */
}

impl<K, V, S> Debug for RRawEntryMut<'_, K, V, S>
where
    K: Debug,
    V: Debug,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RRawEntryMut::Occupied(entry) => Debug::fmt(entry, f),
            RRawEntryMut::Vacant(entry) => Debug::fmt(entry, f),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

/// A handle into an occupied entry in a map.
#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    bound(K: 'a),
    bound(V: 'a),
    bound(S: 'a),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
pub struct RRawOccupiedEntryMut<'a, K, V, S> {
    entry: RMut<'a, ErasedRawOccupiedEntryMut<K, V, S>>,
    vtable: OccupiedVTable_Ref<K, V, S>,
    _marker: UnsafeIgnoredType<OccupiedEntry<'a, K, V, S>>,
}

/// A handle into a vacant entry in a map.
#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    bound(K: 'a),
    bound(V: 'a),
    bound(S: 'a),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
pub struct RRawVacantEntryMut<'a, K, V, S> {
    entry: RMut<'a, ErasedRawVacantEntryMut<K, V, S>>,
    vtable: VacantVTable_Ref<K, V, S>,
    _marker: UnsafeIgnoredType<VacantEntry<'a, K, V, S>>,
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a, K, V, S> RRawOccupiedEntryMut<'a, K, V, S>
where
    S: BuildHasher,
{
    fn vtable(&self) -> OccupiedVTable_Ref<K, V, S> {
        self.vtable
    }

    fn into_inner(self) -> RMut<'a, ErasedRawOccupiedEntryMut<K, V, S>> {
        let mut this = ManuallyDrop::new(self);
        unsafe { ((&mut this.entry) as *mut RMut<'a, ErasedRawOccupiedEntryMut<K, V, S>>).read() }
    }

    pub(super) fn new(entry: &'a mut UnerasedRawOccupiedEntryMut<'a, K, V, S>) -> Self {
        unsafe {
            Self {
                entry: ErasedRawOccupiedEntryMut::from_unerased(entry),
                vtable: OccupiedVTable::VTABLE_REF,
                _marker: UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    /*
    /// Gets a reference to the key of the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry, RHashMap};
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// map.insert(0, 100);
    ///
    /// match map.entry(0) {
    ///     REntry::Occupied(entry) => {
    ///         assert_eq!(entry.key(), &0);
    ///     }
    ///     REntry::Vacant(_) => unreachable!(),
    /// };
    ///
    /// ```
    pub fn key(&self) -> &K {
        let vtable = self.vtable();

        vtable.key()(self.entry.as_rref())
    }
    */

    /*
    /// Gets a reference to the value in the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry, RHashMap};
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// map.insert(6, 15);
    ///
    /// match map.entry(6) {
    ///     REntry::Occupied(entry) => {
    ///         assert_eq!(entry.get(), &15);
    ///     }
    ///     REntry::Vacant(_) => unreachable!(),
    /// };
    ///
    ///
    /// ```
    pub fn get(&self) -> &V {
        let vtable = self.vtable();

        vtable.get_elem()(self.entry.as_rref())
    }
    */

    /*
    /// Gets a mutable reference to the value in the entry.
    /// To borrow with the lifetime of the map, use `ROccupiedEntry::into_mut`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry, RHashMap};
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// map.insert(6, 15);
    ///
    /// match map.entry(6) {
    ///     REntry::Occupied(mut entry) => {
    ///         assert_eq!(entry.get_mut(), &mut 15);
    ///     }
    ///     REntry::Vacant(_) => unreachable!(),
    /// };
    ///
    ///
    /// ```
    pub fn get_mut(&mut self) -> &mut V {
        let vtable = self.vtable();

        vtable.get_mut_elem()(self.entry.reborrow())
    }
    */

    /// Gets a mutable reference to the value in the entry,
    /// that borrows with the lifetime of the map instead of
    /// borrowing from this `ROccupiedEntry`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry, RHashMap};
    ///
    /// let mut map = RHashMap::<String, u32>::new();
    ///
    /// map.insert("baz".into(), 0xDEAD);
    ///
    /// match map.entry("baz".into()) {
    ///     REntry::Occupied(entry) => {
    ///         assert_eq!(entry.into_mut(), &mut 0xDEAD);
    ///     }
    ///     REntry::Vacant(_) => unreachable!(),
    /// };
    ///
    ///
    /// ```
    pub fn into_mut(self) -> &'a mut V {
        let vtable = self.vtable();

        vtable.fn_into_mut_elem()(self)
    }

    /// TODO: docs
    pub fn into_key_value(self) -> (&'a mut K, &'a mut V) {
        let vtable = self.vtable();

        vtable.fn_into_key_value_elem()(self).into()
    }

    /// Replaces the current value of the entry with `value`, returning the previous value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry, RHashMap};
    ///
    /// let mut map = RHashMap::<String, u32>::new();
    ///
    /// map.insert("baz".into(), 0xD00D);
    ///
    /// match map.entry("baz".into()) {
    ///     REntry::Occupied(mut entry) => {
    ///         assert_eq!(entry.insert(0xDEAD), 0xD00D);
    ///     }
    ///     REntry::Vacant(_) => {
    ///         unreachable!();
    ///     }
    /// }
    ///
    /// assert_eq!(map.get("baz"), Some(&0xDEAD));
    ///
    /// ```
    pub fn insert(&mut self, value: V) -> V {
        let vtable = self.vtable();

        vtable.insert_elem()(self.entry.reborrow(), value).into()
    }

    /*
    /// Removes the entry from the map, returns the value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry, RHashMap};
    ///
    /// let mut map = RHashMap::<String, u32>::new();
    ///
    /// map.insert("baz".into(), 0xDEAD);
    ///
    /// match map.entry("baz".into()) {
    ///     REntry::Occupied(entry) => {
    ///         assert_eq!(entry.remove(), 0xDEAD);
    ///     }
    ///     REntry::Vacant(_) => {
    ///         unreachable!();
    ///     }
    /// }
    ///
    /// assert!(!map.contains_key("baz"));
    ///
    /// ```
    pub fn remove(self) -> V {
        let vtable = self.vtable();

        vtable.remove()(self)
    }
    */
}

impl<K, V, S> Debug for RRawOccupiedEntryMut<'_, K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ROccupiedEntry")
            // .field("key", self.key())
            // .field("value", self.get())
            .finish()
    }
}

impl<'a, K, V, S> Drop for RRawOccupiedEntryMut<'a, K, V, S> {
    fn drop(&mut self) {
        let vtable = self.vtable;

        unsafe {
            vtable.drop_entry()(self.entry.reborrow());
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a, K, V, S> RRawVacantEntryMut<'a, K, V, S> {
    fn vtable(&self) -> VacantVTable_Ref<K, V, S> {
        self.vtable
    }
}

impl<'a, K, V, S: BuildHasher> RRawVacantEntryMut<'a, K, V, S> {
    fn into_inner(self) -> RMut<'a, ErasedRawVacantEntryMut<K, V, S>> {
        let mut this = ManuallyDrop::new(self);
        unsafe { ((&mut this.entry) as *mut RMut<'a, ErasedRawVacantEntryMut<K, V, S>>).read() }
    }

    pub(super) fn new(entry: &'a mut UnerasedRawVacantEntryMut<'a, K, V, S>) -> Self
    where
        K: Eq + Hash,
    {
        unsafe {
            Self {
                entry: ErasedRawVacantEntryMut::from_unerased(entry),
                vtable: VacantVTable::VTABLE_REF,
                _marker: UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    /*
    /// Gets a reference to the key of the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry, RHashMap};
    ///
    /// let mut map = RHashMap::<u32, u32>::new();
    ///
    /// match map.entry(1337) {
    ///     REntry::Occupied(_) => {
    ///         unreachable!();
    ///     }
    ///     REntry::Vacant(entry) => {
    ///         assert_eq!(entry.key(), &1337);
    ///     }
    /// }
    ///
    /// assert_eq!(map.get(&1337), None);
    ///
    /// ```
    pub fn key(&self) -> &K {
        let vtable = self.vtable();

        vtable.key()(self.entry.as_rref())
    }
    */

    /*
    /// Gets back the key that was passed to `RHashMap::entry`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry, RHashMap};
    ///
    /// let mut map = RHashMap::<String, u32>::new();
    ///
    /// match map.entry("lol".into()) {
    ///     REntry::Occupied(_) => {
    ///         unreachable!();
    ///     }
    ///     REntry::Vacant(entry) => {
    ///         assert_eq!(entry.into_key(), "lol".to_string());
    ///     }
    /// }
    ///
    /// assert_eq!(map.get("lol"), None);
    ///
    /// ```
    pub fn into_key(self) -> K {
        let vtable = self.vtable();

        vtable.fn_into_key()(self)
    }
    */

    /*
    /// Sets the value of the entry, returning a mutable reference to it.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry, RHashMap};
    ///
    /// let mut map = RHashMap::<String, u32>::new();
    ///
    /// match map.entry("lol".into()) {
    ///     REntry::Occupied(_) => {
    ///         unreachable!();
    ///     }
    ///     REntry::Vacant(entry) => {
    ///         assert_eq!(entry.insert(67), &mut 67);
    ///     }
    /// }
    ///
    /// assert_eq!(map.get("lol"), Some(&67));
    ///
    /// ```
    pub fn insert(self, value: V) -> &'a mut V {
        let vtable = self.vtable();

        vtable.insert_elem()(self, value)
    }
    */

    pub fn insert(self, key: K, value: V) -> (&'a mut K, &'a mut V)
    where
        K: Hash,
        S: BuildHasher,
    {
        let vtable = self.vtable();

        vtable.insert_elem()(self, key, value).into()
    }

    /// TODO: docs
    ///
    /// https://docs.rs/halfbrown/latest/halfbrown/struct.RawVacantEntryMut.html#method.insert_hashed_nocheck
    pub fn insert_hashed_nocheck(self, hash: u64, key: K, value: V) -> (&'a mut K, &'a mut V)
    where
        K: Hash,
        S: BuildHasher,
    {
        let vtable = self.vtable();

        vtable.insert_hashed_nocheck_elem()(self, hash, key, value).into()
    }
}

impl<K, V, S> Debug for RRawVacantEntryMut<'_, K, V, S>
where
    K: Debug,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RVacantEntry")
            // .field("key", self.key())
            .finish()
    }
}

impl<'a, K, V, S> Drop for RRawVacantEntryMut<'a, K, V, S> {
    fn drop(&mut self) {
        let vtable = self.vtable();

        unsafe { vtable.drop_entry()(self.entry.reborrow()) }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    kind(Prefix),
    missing_field(panic),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
pub struct OccupiedVTable<K, V, S> {
    drop_entry: unsafe extern "C" fn(RMut<'_, ErasedRawOccupiedEntryMut<K, V, S>>),
    // key: extern "C" fn(RRef<'_, ErasedRawOccupiedEntryMut<K, V, S>>) -> &K,
    // get_elem: extern "C" fn(RRef<'_, ErasedRawOccupiedEntryMut<K, V, S>>) -> &V,
    // get_mut_elem: extern "C" fn(RMut<'_, ErasedRawOccupiedEntryMut<K, V, S>>) -> &mut V,
    fn_into_mut_elem: extern "C" fn(RRawOccupiedEntryMut<'_, K, V, S>) -> &'_ mut V,
    fn_into_key_value_elem:
        for<'a> extern "C" fn(RRawOccupiedEntryMut<'a, K, V, S>) -> Tuple2<&'a mut K, &'a mut V>,
    insert_elem: extern "C" fn(RMut<'_, ErasedRawOccupiedEntryMut<K, V, S>>, V) -> V,
    // remove: extern "C" fn(RRawOccupiedEntryMut<'_, K, V, S>) -> V,
}

impl<K, V, S> OccupiedVTable<K, V, S>
where
    S: BuildHasher,
{
    const VTABLE_REF: OccupiedVTable_Ref<K, V, S> = OccupiedVTable_Ref(Self::WM_VTABLE.as_prefix());

    staticref! {
        const WM_VTABLE: WithMetadata<OccupiedVTable<K, V, S>> =
            WithMetadata::new(PrefixTypeTrait::METADATA, Self::VTABLE)
    }

    const VTABLE: OccupiedVTable<K, V, S> = OccupiedVTable {
        drop_entry: ErasedRawOccupiedEntryMut::drop_entry,
        // key: ErasedRawOccupiedEntryMut::key,
        // get_elem: ErasedRawOccupiedEntryMut::get_elem,
        // get_mut_elem: ErasedRawOccupiedEntryMut::get_mut_elem,
        fn_into_mut_elem: ErasedRawOccupiedEntryMut::fn_into_mut_elem,
        fn_into_key_value_elem: ErasedRawOccupiedEntryMut::fn_into_key_value_elem,
        insert_elem: ErasedRawOccupiedEntryMut::insert_elem,
        // remove: ErasedRawOccupiedEntryMut::remove,
    };
}

impl<K, V, S> ErasedRawOccupiedEntryMut<K, V, S>
where
    S: BuildHasher,
{
    unsafe extern "C" fn drop_entry(this: RMut<'_, Self>) {
        extern_fn_panic_handling! {
            Self::run_downcast_as_mut(this, |this|{
                ManuallyDrop::drop(this);
            })
        }
    }
    /*
    extern "C" fn key(this: RRef<'_, Self>) -> &K {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as(
                    this,
                    |this| this.key().as_ref()
                )
            }
        }
    }
    */
    /*
    extern "C" fn get_elem(this: RRef<'_, Self>) -> &V {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as(
                    this,
                    |this| this.get()
                )
            }
        }
    }
    */
    /*
    extern "C" fn get_mut_elem(this: RMut<'_, Self>) -> &mut V {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as_mut(
                    this,
                    |this| this.get_mut()
                )
            }
        }
    }
    */
    extern "C" fn fn_into_mut_elem(this: RRawOccupiedEntryMut<'_, K, V, S>) -> &'_ mut V {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as_mut(
                    this.into_inner(),
                    |this| take_manuallydrop(this).into_mut()
                )
            }
        }
    }
    extern "C" fn fn_into_key_value_elem<'a>(
        this: RRawOccupiedEntryMut<'a, K, V, S>,
    ) -> Tuple2<&'a mut K, &'a mut V> {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as_mut(
                    this.into_inner(),
                    |this| {
                        let (key, value) = take_manuallydrop(this).into_key_value();
                        Tuple2(key.as_mut(), value)
                    }
                )
            }
        }
    }
    extern "C" fn insert_elem(this: RMut<'_, Self>, elem: V) -> V {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as_mut(
                    this,
                    |this| this.insert(elem)
                )
            }
        }
    }
    /*
    extern "C" fn remove(this: RRawOccupiedEntryMut<'_, K, V, S>) -> V {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as_mut(
                    this.into_inner(),
                    |this| take_manuallydrop(this).remove()
                )
            }
        }
    }
    */
}

/////////////////////////////////////////////////////////////////////////////////////////////

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    kind(Prefix),
    missing_field(panic),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
pub struct VacantVTable<K, V, S> {
    drop_entry: unsafe extern "C" fn(RMut<'_, ErasedRawVacantEntryMut<K, V, S>>),
    insert_elem: for<'a> extern "C" fn(
        RRawVacantEntryMut<'a, K, V, S>,
        K,
        V,
    ) -> Tuple2<&'a mut K, &'a mut V>,
    insert_hashed_nocheck_elem: for<'a> extern "C" fn(
        RRawVacantEntryMut<'a, K, V, S>,
        u64,
        K,
        V,
    ) -> Tuple2<&'a mut K, &'a mut V>,
}

impl<K, V, S> VacantVTable<K, V, S>
where
    K: Hash,
    S: BuildHasher,
{
    const VTABLE_REF: VacantVTable_Ref<K, V, S> = VacantVTable_Ref(Self::WM_VTABLE.as_prefix());

    staticref! {
        const WM_VTABLE: WithMetadata<VacantVTable<K, V, S>> =
            WithMetadata::new(PrefixTypeTrait::METADATA, Self::VTABLE)
    }

    const VTABLE: VacantVTable<K, V, S> = VacantVTable {
        drop_entry: ErasedRawVacantEntryMut::drop_entry,
        insert_elem: ErasedRawVacantEntryMut::insert_elem,
        insert_hashed_nocheck_elem: ErasedRawVacantEntryMut::insert_hashed_nocheck_elem,
    };
}

impl<K, V, S> ErasedRawVacantEntryMut<K, V, S>
where
    K: Hash,
    S: BuildHasher,
{
    unsafe extern "C" fn drop_entry(this: RMut<'_, Self>) {
        extern_fn_panic_handling! {
            Self::run_downcast_as_mut(this, |this|{
                ManuallyDrop::drop(this);
            })
        }
    }
    extern "C" fn insert_elem<'a>(
        this: RRawVacantEntryMut<'a, K, V, S>,
        key: K,
        elem: V,
    ) -> Tuple2<&'a mut K, &'a mut V> {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as_mut(
                    this.into_inner(),
                    |this| {
                        let (key, value) = take_manuallydrop(this)
                            .insert(MapKey::Value(key), elem);

                        Tuple2(key.as_mut(), value)
                    }
                )
            }
        }
    }
    extern "C" fn insert_hashed_nocheck_elem<'a>(
        this: RRawVacantEntryMut<'a, K, V, S>,
        hash: u64,
        key: K,
        elem: V,
    ) -> Tuple2<&'a mut K, &'a mut V> {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as_mut(
                    this.into_inner(),
                    |this| {
                        let (key, value) = take_manuallydrop(this)
                            .insert_hashed_nocheck(hash, MapKey::Value(key), elem);

                        Tuple2(key.as_mut(), value)
                    }
                )
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

/// Copy paste of the unstable `ManuallyDrop::take`
unsafe fn take_manuallydrop<T>(slot: &mut ManuallyDrop<T>) -> T {
    ManuallyDrop::into_inner(ptr::read(slot))
}
