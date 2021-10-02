use super::*;

use crate::{
    pointer_trait::TransmuteElement,
    sabi_types::{RMut, RRef},
    traits::IntoReprC,
};

impl<K, V, S> ErasedMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    fn run<'a, F, R>(this: RRef<'a, Self>, f: F) -> R
    where
        F: FnOnce(&'a BoxedHashMap<'a, K, V, S>) -> R,
    {
        extern_fn_panic_handling! {
            let map = unsafe{ this.transmute_into_ref::<BoxedHashMap<'a,K,V,S>>() };
            f(map)
        }
    }

    fn run_mut<'a, F, R>(this: RMut<'a, Self>, f: F) -> R
    where
        F: FnOnce(&'a mut BoxedHashMap<'a, K, V, S>) -> R,
    {
        extern_fn_panic_handling! {
            let map = unsafe{ this.transmute_into_mut::<BoxedHashMap<'a,K,V,S>>() };
            f(map)
        }
    }

    fn run_val<'a, F, R>(this: RBox<Self>, f: F) -> R
    where
        F: FnOnce(RBox<BoxedHashMap<'a, K, V, S>>) -> R,
        K: 'a,
        V: 'a,
    {
        extern_fn_panic_handling! {
            let map = unsafe{ this.transmute_element::<BoxedHashMap<'a,K,V,S>>() };
            f( map )
        }
    }

    pub(super) extern "C" fn insert_elem(this: RMut<'_, Self>, key: K, value: V) -> ROption<V> {
        Self::run_mut(this, |this| {
            this.map.insert(MapKey::Value(key), value).into_c()
        })
    }

    pub(super) extern "C" fn get_elem<'a>(
        this: RRef<'a, Self>,
        key: MapQuery<'_, K>,
    ) -> Option<&'a V> {
        Self::run(this, |this| unsafe { this.map.get(&key.as_mapkey()) })
    }

    pub(super) extern "C" fn get_mut_elem<'a>(
        this: RMut<'a, Self>,
        key: MapQuery<'_, K>,
    ) -> Option<&'a mut V> {
        Self::run_mut(this, |this| unsafe { this.map.get_mut(&key.as_mapkey()) })
    }

    pub(super) extern "C" fn remove_entry(
        this: RMut<'_, Self>,
        key: MapQuery<'_, K>,
    ) -> ROption<Tuple2<K, V>> {
        Self::run_mut(this, |this| {
            match this.map.remove_entry(unsafe { &key.as_mapkey() }) {
                Some(x) => RSome(Tuple2(x.0.into_inner(), x.1)),
                None => RNone,
            }
        })
    }

    pub(super) extern "C" fn get_elem_p<'a>(this: RRef<'a, Self>, key: &K) -> Option<&'a V> {
        Self::run(this, |this| this.map.get(key))
    }

    pub(super) extern "C" fn get_mut_elem_p<'a>(
        this: RMut<'a, Self>,
        key: &K,
    ) -> Option<&'a mut V> {
        Self::run_mut(this, |this| this.map.get_mut(key))
    }

    pub(super) extern "C" fn remove_entry_p(
        this: RMut<'_, Self>,
        key: &K,
    ) -> ROption<Tuple2<K, V>> {
        Self::run_mut(this, |this| match this.map.remove_entry(key) {
            Some(x) => RSome(Tuple2(x.0.into_inner(), x.1)),
            None => RNone,
        })
    }

    pub(super) extern "C" fn reserve(this: RMut<'_, Self>, reserved: usize) {
        Self::run_mut(this, |this| this.map.reserve(reserved))
    }

    pub(super) extern "C" fn clear_map(this: RMut<'_, Self>) {
        Self::run_mut(this, |this| this.map.clear())
    }

    pub(super) extern "C" fn len(this: RRef<'_, Self>) -> usize {
        Self::run(this, |this| this.map.len())
    }

    pub(super) extern "C" fn capacity(this: RRef<'_, Self>) -> usize {
        Self::run(this, |this| this.map.capacity())
    }

    pub(super) extern "C" fn iter(this: RRef<'_, Self>) -> Iter<'_, K, V> {
        Self::run(this, |this| {
            let iter = this.map.iter().map(map_iter_ref);
            DynTrait::from_borrowing_value(iter, RefIterInterface::NEW)
        })
    }

    pub(super) extern "C" fn iter_mut(this: RMut<'_, Self>) -> IterMut<'_, K, V> {
        Self::run_mut(this, |this| {
            let iter = this.map.iter_mut().map(map_iter_ref);
            DynTrait::from_borrowing_value(iter, MutIterInterface::NEW)
        })
    }

    pub(super) extern "C" fn drain(this: RMut<'_, Self>) -> Drain<'_, K, V> {
        Self::run_mut(this, |this| {
            let iter = this.map.drain().map(map_iter_val);
            DynTrait::from_borrowing_value(iter, ValIterInterface::NEW)
        })
    }

    pub(super) extern "C" fn iter_val(this: RBox<ErasedMap<K, V, S>>) -> IntoIter<K, V> {
        Self::run_val(this, |this| {
            let iter = this
                .piped(RBox::into_inner)
                .map
                .into_iter()
                .map(map_iter_val);
            let iter = DynTrait::from_borrowing_value(iter, ValIterInterface::NEW);
            unsafe { IntoIter::new(iter) }
        })
    }

    pub(super) extern "C" fn entry(this: RMut<'_, Self>, key: K) -> REntry<'_, K, V> {
        Self::run_mut(this, |this| {
            this.entry = None;
            let map = &mut this.map;
            let entry_mut = this
                .entry
                .get_or_insert_with(|| { map }.entry(MapKey::Value(key)).piped(BoxedREntry::from));

            unsafe { REntry::new(entry_mut) }
        })
    }
}

fn map_iter_ref<'a, K, V: 'a>((key, val): (&'a MapKey<K>, V)) -> Tuple2<&'a K, V> {
    Tuple2(key.as_ref(), val)
}

fn map_iter_val<K, V>((key, val): (MapKey<K>, V)) -> Tuple2<K, V> {
    Tuple2(key.into_inner(), val)
}

///////////////////////////////////////////////////////////////////////////////
