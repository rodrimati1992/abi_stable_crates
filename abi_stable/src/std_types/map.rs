use std::{
    borrow::Borrow,
    collections::{HashMap,hash_map::RandomState},
    cmp::{Eq,PartialEq},
    fmt::{self,Debug},
    hash::{Hash,Hasher,BuildHasher},
    ops::{Index,IndexMut},
    iter::FromIterator,
    ptr::NonNull,
    marker::PhantomData,
    mem,
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    DynTrait,
    StableAbi,
    marker_type::{ErasedObject,NotCopyNotClone,UnsafeIgnoredType},
    erased_types::trait_objects::HasherObject,
    prefix_type::{PrefixTypeTrait,WithMetadata},
    std_types::*,
    traits::{IntoReprRust,ErasedType},
    utils::{transmute_reference,transmute_mut_reference},
};


mod entry;
mod extern_fns;
mod iterator_stuff;
mod map_query;
mod map_key;

#[cfg(test)]
mod test;

use self::{
    map_query::MapQuery,
    map_key::MapKey,
    entry::{BoxedREntry},
};

pub use self::{
    iterator_stuff::{
        RefIterInterface,MutIterInterface,ValIterInterface,
        IntoIter,
    },
    entry::{REntry,ROccupiedEntry,RVacantEntry},
};


/**

An ffi-safe hashmap,which wraps `std::collections::HashMap<K,V,S>`,
only requiring the `K:Eq+Hash` bounds when constructing it.

Most of the API in `HashMap` is implemented here,including the Entry API.

*/
#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    inside_abi_stable_crate,
    // The hasher doesn't matter
    unconstrained(S),
)]
pub struct RHashMap<K,V,S=RandomState>{
    map:RBox<ErasedMap<K,V,S>>,
    vtable:*const VTable<K,V,S>,
}


///////////////////////////////////////////////////////////////////////////////


struct BoxedHashMap<'a,K,V,S>{
    map:HashMap<MapKey<K>,V,S>,
    entry:Option<BoxedREntry<'a,K,V>>,
}

/// An RHashMap iterator,
/// implementing `Iterator<Item= Tuple2< &K, &V > >+!Send+!Sync+Clone`
pub type Iter<'a,K,V>=
    DynTrait<'a,RBox<()>,RefIterInterface<K,V>>;

/// An RHashMap iterator,
/// implementing `Iterator<Item= Tuple2< &K, &mut V > >+!Send+!Sync`
pub type IterMut<'a,K,V>=
    DynTrait<'a,RBox<()>,MutIterInterface<K,V>>;

/// An RHashMap iterator,
/// implementing `Iterator<Item= Tuple2< K, V > >+!Send+!Sync`
pub type Drain<'a,K,V>=
    DynTrait<'a,RBox<()>,ValIterInterface<K,V>>;


/// Used as the erased type of the RHashMap type.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    inside_abi_stable_crate,
    // The hasher doesn't matter
    unconstrained(S),
)]
struct ErasedMap<K,V,S>(
    PhantomData<Tuple2<K,V>>,
    UnsafeIgnoredType<S>
);

unsafe impl<'a,K:'a,V:'a,S:'a> ErasedType<'a> for ErasedMap<K,V,S> {
    type Unerased=BoxedHashMap<'a,K,V,S>;
}


///////////////////////////////////////////////////////////////////////////////


impl<K,V> RHashMap<K,V,RandomState>{
    /// Constructs an empty RHashMap.
    #[inline]
    pub fn new()->RHashMap<K,V>
    where 
        Self:Default
    {
        Self::default()
    }

    /// Constructs an empty RHashMap with the passed capacity.
    #[inline]
    pub fn with_capacity(capacity:usize)->RHashMap<K,V>
    where 
        Self:Default
    {
        let mut this=Self::default();
        this.reserve(capacity);
        this
    }
}


impl<K,V,S> RHashMap<K,V,S>{
    /// Constructs an empty RHashMap with the passed `hash_builder` to hash the keys.
    #[inline]
    pub fn with_hasher(hash_builder: S) -> RHashMap<K, V, S> 
    where
        K:Eq+Hash,
        S:BuildHasher+Default,
    {
        Self::with_capacity_and_hasher(0,hash_builder)
    }
    /// Constructs an empty RHashMap with the passed capacity,
    /// and the passed `hash_builder` to hash the keys.
    pub fn with_capacity_and_hasher(
        capacity: usize,
        hash_builder: S
    ) -> RHashMap<K, V, S> 
    where
        K:Eq+Hash,
        S:BuildHasher+Default,
    {
        let mut map=VTable::<K,V,S>::erased_map(hash_builder);
        map.reserve(capacity);
        RHashMap{
            map,
            vtable:unsafe{
                (*VTable::VTABLE_REF).as_prefix_raw()
            },
        }
    }
}


impl<K,V,S> RHashMap<K,V,S>{

    fn vtable<'a>(&self)->&'a VTable<K,V,S>{
        unsafe{ 
            &*self.vtable
        }
    }

}


impl<K,V,S> RHashMap<K,V,S>{
    /// Returns whether the map associates a value with the key.
    pub fn contains_key<Q>(&self,query:&Q)->bool
    where
        K:Borrow<Q>,
        Q:Hash+Eq+?Sized
    {
        self.get(query).is_some()
    }

    /// Returns a reference to the value associated with the key.
    pub fn get<Q>(&self,query:&Q)->Option<&V>
    where
        K:Borrow<Q>,
        Q:Hash+Eq+?Sized
    {
        let vtable=self.vtable();
        unsafe{
            vtable.get_elem()(&*self.map,MapQuery::new(&query))
        }
    }

    /// Returns a mutable reference to the value associated with the key.
    pub fn get_mut<Q>(&mut self,query:&Q)->Option<&mut V>
    where
        K:Borrow<Q>,
        Q:Hash+Eq+?Sized
    {
        let vtable=self.vtable();
        unsafe{
            vtable.get_mut_elem()(&mut *self.map,MapQuery::new(&query))
        }
    }

    /// Removes the value associated with the key.
    pub fn remove<Q>(&mut self,query:&Q)->ROption<V>
    where
        K:Borrow<Q>,
        Q:Hash+Eq+?Sized
    {
        self.remove_entry(query).map(|x| x.1 )
    }

    /// Removes the entry for the key.
    pub fn remove_entry<Q>(&mut self,query:&Q)->ROption<Tuple2<K,V>>
    where
        K:Borrow<Q>,
        Q:Hash+Eq+?Sized
    {
        let vtable=self.vtable();
        vtable.remove_entry()(&mut *self.map,MapQuery::new(&query))
    }
}


impl<K,V,S> RHashMap<K,V,S>{
    /// Returns whether the map associates a value with the key.
    pub fn contains_key_p(&self,key:&K)->bool{
        self.get_p(key).is_some()
    }

    /// Returns a reference to the value associated with the key.
    pub fn get_p(&self,key:&K)->Option<&V>{
        let vtable=self.vtable();
        unsafe{
            vtable.get_elem_p()(&*self.map,&key)
        }
    }

    /// Returns a mutable reference to the value associated with the key.
    pub fn get_mut_p(&mut self,key:&K)->Option<&mut V>{
        let vtable=self.vtable();
        unsafe{
            vtable.get_mut_elem_p()(&mut *self.map,&key)
        }
    }

    /// Removes the entry for the key.
    pub fn remove_entry_p(&mut self,key:&K)->ROption<Tuple2<K,V>>{
        let vtable=self.vtable();
        vtable.remove_entry_p()(&mut *self.map,&key)
    }

    /// Removes the value associated with the key.
    pub fn remove_p(&mut self,key:&K)->ROption<V>{
        self.remove_entry_p(key).map(|x| x.1 )
    }

    /// Returns a reference to the value associated with the key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not associated with a value.
    pub fn index_p(&self,key:&K)->&V{
        self.get_p(key).expect("no entry in RHashMap<_,_> found for key")
    }

    /// Returns a mutable reference to the value associated with the key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not associated with a value.
    pub fn index_mut_p(&mut self,key:&K)->&V{
        self.get_mut_p(key).expect("no entry in RHashMap<_,_> found for key")
    }

    //////////////////////////////////

    /// Inserts a value into the map,associating it with a key,returning the previous value.
    pub fn insert(&mut self,key:K,value:V)->ROption<V>{
        let vtable=self.vtable();
        unsafe{
            vtable.insert_elem()(&mut *self.map,key,value)
        }
    }

    /// Reserves enough space to insert `reserved` extra elements.
    pub fn reserve(&mut self,reserved:usize){
        let vtable=self.vtable();

        vtable.reserve()(&mut *self.map,reserved);
    }

    /// Removes all the entries in the map.
    pub fn clear(&mut self){
        let vtable=self.vtable();
        vtable.clear_map()(&mut *self.map);
    }

    /// Returns the ammount of entries in the map.
    pub fn len(&self)->usize{
        let vtable=self.vtable();
        vtable.len()(&*self.map)
    }

    /// Returns the capacity of the map,the ammount of elements it can store without reallocating.
    ///
    /// Note that this is a lower bound,since hash maps don't necessarily have an exact capacity.
    pub fn capacity(&self)->usize{
        let vtable=self.vtable();
        vtable.capacity()(&*self.map)
    }

    /// Returns whether the map contains any entries.
    pub fn is_empty(&self)->bool{
        self.len()==0
    }

    /// Iterates over the entries in the map,with references to the values in the map.
    ///
    /// This returns an `Iterator<Item= Tuple2< &K, &V > >+!Send+!Sync+Clone`
    pub fn iter    (&self)->Iter<'_,K,V>{
        let vtable=self.vtable();

        vtable.iter()(&*self.map)
    }
    
    /// Iterates over the entries in the map,with mutable references to the values in the map.
    ///
    /// This returns an `Iterator<Item= Tuple2< &K, &mutV > >+!Send+!Sync`
    pub fn iter_mut(&mut self)->IterMut<'_,K,V>{
        let vtable=self.vtable();

        vtable.iter_mut()(&mut *self.map)
    }

    /// Clears the map,returning an iterator over all the entries that were removed.
    /// 
    /// This returns an `Iterator<Item= Tuple2< K, V > >+!Send+!Sync`
    pub fn drain   (&mut self)->Drain<'_,K,V>{
        let vtable=self.vtable();

        vtable.drain()(&mut *self.map)
    }

    /// Gets a handle into the entry in the map for the key,
    /// that allows operating directly on the entry.
    pub fn entry(&mut self,key:K)->REntry<'_,K,V>{
        let vtable=self.vtable();

        vtable.entry()(&mut *self.map,key)
    }
}


/// This returns an `Iterator<Item= Tuple2< K, V > >+!Send+!Sync`
impl<K,V,S> IntoIterator for RHashMap<K,V,S>{
    type Item=Tuple2<K,V>;
    type IntoIter=IntoIter<K,V>;
    
    fn into_iter(self)->IntoIter<K,V>{
        let vtable=self.vtable();

        vtable.iter_val()(self.map)
    }
}


/// This returns an `Iterator<Item= Tuple2< &K, &V > >+!Send+!Sync+Clone`
impl<'a,K,V,S> IntoIterator for &'a RHashMap<K,V,S>{
    type Item=Tuple2<&'a K,&'a V>;
    type IntoIter=Iter<'a,K,V>;
    
    fn into_iter(self)->Self::IntoIter{
        self.iter()
    }
}


/// This returns an `Iterator<Item= Tuple2< &K, &mutV > >+!Send+!Sync`
impl<'a,K,V,S> IntoIterator for &'a mut RHashMap<K,V,S>{
    type Item=Tuple2<&'a K,&'a mut V>;
    type IntoIter=IterMut<'a,K,V>;
    
    fn into_iter(self)->Self::IntoIter{
        self.iter_mut()
    }
}


impl<K,V,S> From<HashMap<K,V,S>> for RHashMap<K,V,S>
where
    Self:Default
{
    fn from(map:HashMap<K,V,S>)->Self{
        map.into_iter().collect()
    }
}

impl<K,V,S> Into<HashMap<K,V,S>> for RHashMap<K,V,S>
where
    K:Eq+Hash,
    S:BuildHasher+Default,
{
    fn into(self)->HashMap<K,V,S>{
        self.into_iter().map(IntoReprRust::into_rust).collect()
    }
}


impl<K,V,S> FromIterator<(K,V)> for RHashMap<K,V,S>
where
    Self:Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K,V)>
    {
        let mut map=Self::default();
        map.extend(iter);
        map
    }
}


impl<K,V,S> FromIterator<Tuple2<K,V>> for RHashMap<K,V,S>
where
    Self:Default
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Tuple2<K,V>>
    {
        let mut map=Self::default();
        map.extend(iter);
        map
    }
}


impl<K,V,S> Extend<(K,V)> for RHashMap<K,V,S>{
    fn extend<I>(&mut self,iter: I)
    where
        I: IntoIterator<Item = (K,V)>
    {
        let iter=iter.into_iter();
        self.reserve(iter.size_hint().0);
        for (k,v) in iter {
            self.insert(k,v);
        }
    }
}


impl<K,V,S> Extend<Tuple2<K,V>> for RHashMap<K,V,S>{
    #[inline]
    fn extend<I>(&mut self,iter: I)
    where
        I: IntoIterator<Item = Tuple2<K,V>>
    {
        self.extend( iter.into_iter().map(Tuple2::into_rust) );
    }
}

impl<K,V,S> Default for RHashMap<K,V,S>
where
    K:Eq+Hash,
    S:BuildHasher+Default,
{
    fn default()->Self{
        Self::with_hasher(S::default())
    }
}


impl<K,V,S> Clone for RHashMap<K,V,S>
where 
    K:Clone,
    V:Clone,
    Self:Default
{
    fn clone(&self)->Self{
        self.iter().map(|Tuple2(k,v)| (k.clone(),v.clone()) ).collect()
    }
}


impl<K,V,S> Debug for RHashMap<K,V,S>
where 
    K:Debug,
    V:Debug,
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_map()
         .entries(self.iter().map(Tuple2::into_rust))
         .finish()
    }
}


impl<K,V,S> Eq for RHashMap<K,V,S>
where 
    K:Eq,
    V:Eq,
{}


impl<K,V,S> PartialEq for RHashMap<K,V,S>
where 
    K:PartialEq,
    V:PartialEq,
{
    fn eq(&self,other:&Self)->bool{
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|Tuple2(k, vl)|{
                other.get_p(k)
                     .map_or(false, |vr| *vr == *vl)
            })
    }
}


unsafe impl<K, V, S> Send for RHashMap<K, V, S> 
where
    HashMap<K, V, S>: Send,
{}

unsafe impl<K, V, S> Sync for RHashMap<K, V, S> 
where
    HashMap<K, V, S>: Sync,
{}


impl<K,Q,V,S> Index<&Q> for RHashMap<K,V,S>
where
    K:Borrow<Q>,
    Q:Eq+Hash
{
    type Output=V;

    fn index(&self,query:&Q)->&V{
        self.get(query).expect("no entry in RHashMap<_,_> found for key")
    }
}

impl<K,Q,V,S> IndexMut<&Q> for RHashMap<K,V,S>
where
    K:Borrow<Q>,
    Q:Eq+Hash
{
    fn index_mut(&mut self,query:&Q)->&mut V{
        self.get_mut(query).expect("no entry in RHashMap<_,_> found for key")
    }
}


mod serde{
    use super::*;

    use ::serde::{
        de::{Visitor, MapAccess},
        ser::SerializeMap,
        Deserialize,Serialize,Deserializer,Serializer,
    };


    struct RHashMapVisitor<K,V,S> {
        marker: PhantomData<fn() -> RHashMap<K,V,S>>
    }

    impl<K,V,S> RHashMapVisitor<K,V,S> {
        fn new() -> Self {
            RHashMapVisitor {
                marker: PhantomData
            }
        }
    }

    impl<'de,K,V,S> Visitor<'de> for RHashMapVisitor<K,V,S>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
        RHashMap<K,V,S>:Default,
    {
        type Value = RHashMap<K,V,S>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an RHashMap")
        }

        fn visit_map<M>(self, mut map_access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let capacity=map_access.size_hint().unwrap_or(0);
            let mut map = RHashMap::default();
            map.reserve(capacity);

            while let Some((k, v)) = map_access.next_entry()? {
                map.insert(k, v);
            }

            Ok(map)
        }
    }

    impl<'de,K,V,S> Deserialize<'de> for RHashMap<K,V,S>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
        Self:Default,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(RHashMapVisitor::new())
        }
    }

    

    impl<K,V,S> Serialize for RHashMap<K,V,S>
    where
        K:Serialize,
        V:Serialize,
    {
        fn serialize<Z>(&self, serializer: Z) -> Result<Z::Ok, Z::Error>
        where
            Z: Serializer
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
    inside_abi_stable_crate,
    kind(Prefix(prefix_struct="VTable")),
    missing_field(panic),
    // The hasher doesn't matter
    unconstrained(S),
)]
pub struct VTableVal<K,V,S>{
    insert_elem:extern fn(&mut ErasedMap<K,V,S>,K,V)->ROption<V>,
    
    get_elem:for<'a> extern fn(&'a ErasedMap<K,V,S>,MapQuery<'_,K>)->Option<&'a V>,
    get_mut_elem:for<'a> extern fn(&'a mut ErasedMap<K,V,S>,MapQuery<'_,K>)->Option<&'a mut V>,
    remove_entry:extern fn(&mut ErasedMap<K,V,S>,MapQuery<'_,K>)->ROption<Tuple2<K,V>>,
    
    get_elem_p:for<'a> extern fn(&'a ErasedMap<K,V,S>,&K)->Option<&'a V>,
    get_mut_elem_p:for<'a> extern fn(&'a mut ErasedMap<K,V,S>,&K)->Option<&'a mut V>,
    remove_entry_p:extern fn(&mut ErasedMap<K,V,S>,&K)->ROption<Tuple2<K,V>>,
    
    reserve:extern fn(&mut ErasedMap<K,V,S>,usize),
    clear_map:extern fn(&mut ErasedMap<K,V,S>),
    len:extern fn(&ErasedMap<K,V,S>)->usize,
    capacity:extern fn(&ErasedMap<K,V,S>)->usize,
    iter    :extern fn(&ErasedMap<K,V,S>     )->Iter<'_,K,V>,
    iter_mut:extern fn(&mut ErasedMap<K,V,S> )->IterMut<'_,K,V>,
    drain   :extern fn(&mut ErasedMap<K,V,S> )->Drain<'_,K,V>,
    iter_val:extern fn(RBox<ErasedMap<K,V,S>>)->IntoIter<K,V>,
    #[sabi(last_prefix_field)]
    entry:extern fn(&mut ErasedMap<K,V,S>,K)->REntry<'_,K,V>,
}



impl<K,V,S> VTable<K,V,S>
where
    K:Eq+Hash,
    S:BuildHasher,
{
    const VTABLE_REF: *const WithMetadata<VTableVal<K,V,S>> = {
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VTABLE)
    };

    fn erased_map(hash_builder:S)->RBox<ErasedMap<K,V,S>>{
        unsafe{
            let map=HashMap::<MapKey<K>,V,S>::with_hasher(hash_builder);
            let boxed=BoxedHashMap{
                map,
                entry:None,
            };
            let boxed=RBox::new(boxed);
            let boxed=mem::transmute::<RBox<_>,RBox<ErasedMap<K,V,S>>>(boxed);
            boxed
        }
    }


    const VTABLE:VTableVal<K,V,S>=VTableVal{
        insert_elem :ErasedMap::insert_elem,

        get_elem    :ErasedMap::get_elem,
        get_mut_elem:ErasedMap::get_mut_elem,
        remove_entry:ErasedMap::remove_entry,

        get_elem_p    :ErasedMap::get_elem_p,
        get_mut_elem_p:ErasedMap::get_mut_elem_p,
        remove_entry_p:ErasedMap::remove_entry_p,

        reserve     :ErasedMap::reserve,
        clear_map   :ErasedMap::clear_map,
        len         :ErasedMap::len,
        capacity    :ErasedMap::capacity,
        iter        :ErasedMap::iter,
        iter_mut    :ErasedMap::iter_mut,
        drain       :ErasedMap::drain,
        iter_val    :ErasedMap::iter_val,
        entry       :ErasedMap::entry,
    };

}



///////////////////////////////////////////////////////////////////////////////
