use super::*;

use crate::{
    pointer_trait::TransmuteElement,
    traits::IntoReprC,
};


impl<K,V,S> ErasedMap<K,V,S>
where 
    K:Hash+Eq,
    S:BuildHasher,
{
    pub(super) unsafe fn as_hashmap(&self)->&BoxedHashMap<'_,K,V,S>{
        transmute_reference(self)
    }
    pub(super) unsafe fn as_mut_hashmap(&mut self)->&mut BoxedHashMap<'_,K,V,S>{
        transmute_mut_reference(self)
    }

    fn run<'a,F,R>(&'a self,f:F)->R
    where F:FnOnce(&'a BoxedHashMap<'a,K,V,S>)->R
    {
        extern_fn_panic_handling!{
            let map=unsafe{ self.as_hashmap() };
            f( map )
        }
    }
    
    fn run_mut<'a,F,R>(&'a mut self,f:F)->R
    where F:FnOnce(&'a mut BoxedHashMap<'a,K,V,S>)->R
    {
        extern_fn_panic_handling!{
            let map=unsafe{ self.as_mut_hashmap() };
            f( map )
        }
    }

    fn run_val<'a,F,R>(this:RBox<Self>,f:F)->R
    where 
        F:FnOnce(RBox<BoxedHashMap<'a,K,V,S>>)->R,
        K:'a,
        V:'a,
    {
        extern_fn_panic_handling!{
            let map=unsafe{ this.transmute_element::<BoxedHashMap<'a,K,V,S>>() };
            f( map )
        }
    }

    pub(super)extern "C" fn insert_elem(&mut self,key:K,value:V)->ROption<V>{
        self.run_mut(|this|{
            this.map.insert(MapKey::Value(key),value)
                .into_c()
        })
    }

    pub(super)extern "C" fn get_elem(&self,key:MapQuery<'_,K>)->Option<&V>{
        self.run(|this|unsafe{ 
            this.map.get(&key.as_mapkey()) 
        })
    }    

    pub(super)extern "C" fn get_mut_elem(&mut self,key:MapQuery<'_,K>)->Option<&mut V>{
        self.run_mut(|this|unsafe{ 
            this.map.get_mut(&key.as_mapkey()) 
        })
    }

    pub(super)extern "C" fn remove_entry(&mut self,key:MapQuery<'_,K>)->ROption<Tuple2<K,V>>{
        self.run_mut(|this|{
            match this.map.remove_entry(unsafe{ &key.as_mapkey() }) {
                Some(x)=>RSome(Tuple2(x.0.into_inner(),x.1)),
                None=>RNone,
            }
        })
    }

    pub(super)extern "C" fn get_elem_p(&self,key:&K)->Option<&V>{
        self.run(|this| this.map.get(key) )
    }    

    pub(super)extern "C" fn get_mut_elem_p(&mut self,key:&K)->Option<&mut V>{
        self.run_mut(|this| this.map.get_mut(key) )
    }

    pub(super)extern "C" fn remove_entry_p(&mut self,key:&K)->ROption<Tuple2<K,V>>{
        self.run_mut(|this|{
            match this.map.remove_entry( key ) {
                Some(x)=>RSome(Tuple2(x.0.into_inner(),x.1)),
                None=>RNone,
            }
        })
    }


    pub(super)extern "C" fn reserve(&mut self,reserved:usize){
        self.run_mut(|this| this.map.reserve(reserved) )
    }

    pub(super)extern "C" fn clear_map(&mut self){
        self.run_mut(|this| this.map.clear() )
    }

    pub(super)extern "C" fn len(&self)->usize{
        self.run(|this| this.map.len() )
    }

    pub(super)extern "C" fn capacity(&self)->usize{
        self.run(|this| this.map.capacity() )
    }

    pub(super)extern "C" fn iter     (&self)->Iter<'_,K,V>{
        self.run(|this|{
            let iter=this.map.iter().map(map_iter_ref);
            DynTrait::from_borrowing_value(iter,RefIterInterface::NEW)
        })
    }

    pub(super)extern "C" fn iter_mut (&mut self)->IterMut<'_,K,V>{
        self.run_mut(|this|{
            let iter=this.map.iter_mut().map(map_iter_ref);
            DynTrait::from_borrowing_value(iter,MutIterInterface::NEW)
        })
    }

    pub(super)extern "C" fn drain    (&mut self)->Drain<'_,K,V>{
        self.run_mut(|this|{
            let iter=this.map.drain().map(map_iter_val);
            DynTrait::from_borrowing_value(iter,ValIterInterface::NEW)
        })
    }

    pub(super)extern "C" fn iter_val<'a>(this:RBox<ErasedMap<K,V,S>>)->IntoIter<K,V>{
        Self::run_val(this,|this|{
            let iter=this.piped(RBox::into_inner).map.into_iter().map(map_iter_val);
            let iter=DynTrait::from_borrowing_value(iter,ValIterInterface::NEW);
            unsafe{ IntoIter::new(iter) }
        })
    }

    pub(super)extern "C" fn entry(&mut self,key:K)->REntry<'_,K,V>{
        self.run_mut(|this|{
            this.entry=None;
            let map=&mut this.map;
            let entry_mut=this.entry
                .get_or_insert_with(||{ 
                    {map}.entry(MapKey::Value(key))
                       .piped(BoxedREntry::from) 
                });
                
            unsafe{
                REntry::new(entry_mut)
            }
        })
    }
}


fn map_iter_ref<'a,K,V:'a>((key,val):(&'a MapKey<K>,V))->Tuple2<&'a K,V>{
    Tuple2( key.as_ref(),val )
}

fn map_iter_val<K,V>((key,val):(MapKey<K>,V))->Tuple2<K,V>{
    Tuple2( key.into_inner(),val )
}


///////////////////////////////////////////////////////////////////////////////
