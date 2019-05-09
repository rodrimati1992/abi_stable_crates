use std::{
    borrow::Borrow,
    collections::HashMap,
    cmp::{Eq,PartialEq},
    fmt::{self,Debug},
    hash::{Hash,Hasher},
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
    marker_type::{ErasedObject,NotCopyNotClone},
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



#[derive(StableAbi)]
#[repr(C)]
#[sabi(inside_abi_stable_crate)]
pub struct RHashMap<K,V>{
    map:RBox<ErasedMap<K,V>>,
    vtable:*const VTable<K,V>,
}


struct BoxedHashMap<'a,K,V>{
    map:HashMap<MapKey<K>,V>,
    entry:Option<BoxedREntry<'a,K,V>>,
}





impl<K,V> RHashMap<K,V>{
    pub fn new()->RHashMap<K,V>
    where 
        K:Hash+Eq
    {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity:usize)->RHashMap<K,V>
    where 
        K:Hash+Eq
    {
        RHashMap{
            map:VTable::<K,V>::erased_map_with_capacity(capacity),
            vtable:unsafe{
                (*VTable::VTABLE_REF).as_prefix_raw()
            },
        }
    }

    fn vtable<'a>(&self)->&'a VTable<K,V>{
        unsafe{ 
            &*self.vtable
        }
    }

    pub fn contains_key<Q>(&self,query:&Q)->bool
    where
        K:Borrow<Q>,
        Q:Hash+Eq+?Sized
    {
        self.get(query).is_some()
    }

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

    pub fn insert(&mut self,key:K,value:V)->ROption<V>
    where
        K:Hash+Eq,
    {
        let vtable=self.vtable();
        unsafe{
            vtable.insert_elem()(&mut *self.map,key,value)
        }
    }

    pub fn remove_entry<Q>(&mut self,query:&Q)->ROption<Tuple2<K,V>>
    where
        K:Borrow<Q>,
        Q:Hash+Eq+?Sized
    {
        let vtable=self.vtable();
        vtable.remove_entry()(&mut *self.map,MapQuery::new(&query))
    }

    pub fn remove<Q>(&mut self,query:&Q)->ROption<V>
    where
        K:Borrow<Q>,
        Q:Hash+Eq+?Sized
    {
        self.remove_entry(query).map(|x| x.1 )
    }

    pub fn clear(&mut self){
        let vtable=self.vtable();
        vtable.clear_map()(&mut *self.map);
    }

    pub fn len(&self)->usize{
        let vtable=self.vtable();
        vtable.len()(&*self.map)
    }

    pub fn is_empty(&self)->bool{
        self.len()==0
    }

    pub fn iter    (&self)->Iter<'_,K,V>{
        let vtable=self.vtable();

        vtable.iter()(&*self.map)
    }
    
    pub fn iter_mut(&mut self)->IterMut<'_,K,V>{
        let vtable=self.vtable();

        vtable.iter_mut()(&mut *self.map)
    }

    pub fn drain   (&mut self)->Drain<'_,K,V>{
        let vtable=self.vtable();

        vtable.drain()(&mut *self.map)
    }

    pub fn entry(&mut self,key:K)->REntry<'_,K,V>{
        let vtable=self.vtable();

        vtable.entry()(&mut *self.map,key)
    }
}


impl<K,V> IntoIterator for RHashMap<K,V>{
    type Item=Tuple2<K,V>;
    type IntoIter=IntoIter<K,V>;
    
    fn into_iter(self)->IntoIter<K,V>{
        let vtable=self.vtable();

        vtable.iter_val()(self.map)
    }
}


impl<K,V> From<HashMap<K,V>> for RHashMap<K,V>
where
    K:Eq+Hash,
{
    fn from(map:HashMap<K,V>)->Self{
        map.into_iter().collect()
    }
}

impl<K,V> Into<HashMap<K,V>> for RHashMap<K,V>
where
    K:Eq+Hash,
{
    fn into(self)->HashMap<K,V>{
        self.into_iter().map(IntoReprRust::into_rust).collect()
    }
}


impl<K,V> FromIterator<(K,V)> for RHashMap<K,V>
where
    K:Eq+Hash,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K,V)>
    {
        let iter=iter.into_iter();
        let mut map=Self::with_capacity(iter.size_hint().0);
        for (k,v) in iter {
            map.insert(k,v);
        }
        map
    }
}


pub type Iter<'a,K,V>=
    DynTrait<'a,RBox<()>,RefIterInterface<K,V>>;

pub type IterMut<'a,K,V>=
    DynTrait<'a,RBox<()>,MutIterInterface<K,V>>;

pub type Drain<'a,K,V>=
    DynTrait<'a,RBox<()>,ValIterInterface<K,V>>;


/// Used as the erased type of the RHashMap type.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
struct ErasedMap<K,V>(PhantomData<Tuple2<K,V>>);

unsafe impl<'a,K:'a,V:'a> ErasedType<'a> for ErasedMap<K,V> {
    type Unerased=BoxedHashMap<'a,K,V>;
}


///////////////////////////////////////////////////////////////////////////////


#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    inside_abi_stable_crate,
    kind(Prefix(prefix_struct="VTable")),
    missing_field(panic),
)]
pub struct VTableVal<K,V>{
    get_elem:for<'a> extern fn(&'a ErasedMap<K,V>,MapQuery<'_,K>)->Option<&'a V>,
    get_mut_elem:for<'a> extern fn(&'a mut ErasedMap<K,V>,MapQuery<'_,K>)->Option<&'a mut V>,
    insert_elem:extern fn(&mut ErasedMap<K,V>,K,V)->ROption<V>,
    remove_entry:extern fn(&mut ErasedMap<K,V>,MapQuery<'_,K>)->ROption<Tuple2<K,V>>,
    clear_map:extern fn(&mut ErasedMap<K,V>),
    len:extern fn(&ErasedMap<K,V>)->usize,
    iter    :extern fn(&ErasedMap<K,V>     )->Iter<'_,K,V>,
    iter_mut:extern fn(&mut ErasedMap<K,V> )->IterMut<'_,K,V>,
    drain   :extern fn(&mut ErasedMap<K,V> )->Drain<'_,K,V>,
    iter_val:extern fn(RBox<ErasedMap<K,V>>)->IntoIter<K,V>,
    entry:extern fn(&mut ErasedMap<K,V>,K)->REntry<'_,K,V>,
    _type_params:PhantomData<extern fn(K,V)>,
}



impl<K,V> VTable<K,V>
where
    K:Eq+Hash,
{
    const VTABLE_REF: *const WithMetadata<VTableVal<K,V>> = {
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VTABLE)
    };

    fn erased_map_with_capacity(capacity:usize)->RBox<ErasedMap<K,V>>{
        unsafe{
            let map=HashMap::<MapKey<K>,V>::with_capacity(capacity);
            let boxed=BoxedHashMap{
                map,
                entry:None,
            };
            let boxed=RBox::new(boxed);
            let boxed=mem::transmute::<RBox<_>,RBox<ErasedMap<K,V>>>(boxed);
            boxed
        }
    }


    const VTABLE:VTableVal<K,V>=VTableVal{
        get_elem    :ErasedMap::get_elem,
        get_mut_elem:ErasedMap::get_mut_elem,
        insert_elem :ErasedMap::insert_elem,
        remove_entry:ErasedMap::remove_entry,
        clear_map   :ErasedMap::clear_map,
        len         :ErasedMap::len,
        iter        :ErasedMap::iter,
        iter_mut    :ErasedMap::iter_mut,
        drain       :ErasedMap::drain,
        iter_val    :ErasedMap::iter_val,
        entry       :ErasedMap::entry,
        _type_params:PhantomData
    };

}



///////////////////////////////////////////////////////////////////////////////
