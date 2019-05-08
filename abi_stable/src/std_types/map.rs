use std::{
    borrow::Borrow,
    collections::{
        HashMap,
        hash_map::Entry,
    },
    cmp::{Eq,PartialEq},
    fmt::{self,Debug},
    hash::{Hash,Hasher},
    ptr::NonNull,
    marker::PhantomData,
    mem,
};

use crate::{
    StableAbi,
    marker_type::{ErasedObject,NotCopyNotClone},
    erased_types::trait_objects::HasherObject,
    prefix_type::{PrefixTypeTrait,WithMetadata},
    std_types::*,
    traits::IntoReprC,
    utils::{transmute_reference,transmute_mut_reference},
};


mod map_query;
mod map_key;
mod extern_fns;

#[cfg(test)]
mod test;

use self::{
    map_query::MapQuery,
    map_key::MapKey,
    extern_fns::*,
};



#[derive(StableAbi)]
#[repr(C)]
#[sabi(inside_abi_stable_crate)]
pub struct RHashMap<K,V>{
    map:RBox<ErasedMap<K,V>>,
    vtable:*const VTable<K,V>,
}


impl<K,V> RHashMap<K,V>{
    pub fn new<'k>()->RHashMap<K,V>
    where 
        ():MapConstruction<'k,K,V>,
    {
        RHashMap{
            map:<() as MapConstruction<'k,K,V>>::new_erased_map(),
            vtable:unsafe{
                (*<() as MapConstruction<'k,K,V>>::VTABLE_REF).as_prefix_raw()
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
}


/// Used as the erased type of the RHashMap type.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct ErasedMap<K,V,M=()>(PhantomData<Tuple2<K,V,M>>);

/// Used as the erased type of the Key type.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct ErasedKey<K>(PhantomData<K>);

/// Used as the erased type of the Proxy type
/// (the type the key gets converted to do comparisons).
#[derive(StableAbi)]
#[repr(C)]
#[sabi(inside_abi_stable_crate)]
pub struct ErasedProxy<K>(PhantomData<K>);


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
    _type_params:PhantomData<extern fn(K,V)>,
}

//////////////////////////////////////////////////////////////////////////////


pub trait MapConstruction<'lt,K,V>{
    const VTABLE:VTableVal<K,V>;

    fn new_erased_map()->RBox<ErasedMap<K,V>>;

    const VTABLE_REF: *const WithMetadata<VTableVal<K,V>> = {
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VTABLE)
    };
}

impl<'lt,K:'lt,V> MapConstruction<'lt,K,V> for ()
where
    K:Eq+Hash,
{
    fn new_erased_map()->RBox<ErasedMap<K,V>>{
        unsafe{
            let x=HashMap::<MapKey<K>,V>::new();
            let x=RBox::new(x);
            let x=mem::transmute::<RBox<_>,RBox<ErasedMap<K,V>>>(x);
            x
        }
    }

    const VTABLE:VTableVal<K,V>=VTableVal{
        get_elem    :ErasedMap::get_elem,
        get_mut_elem:ErasedMap::get_mut_elem,
        insert_elem :ErasedMap::insert_elem,
        remove_entry:ErasedMap::remove_entry,
        clear_map   :ErasedMap::clear_map,
        len         :ErasedMap::len,
        _type_params:PhantomData
    };

}



///////////////////////////////////////////////////////////////////////////////
