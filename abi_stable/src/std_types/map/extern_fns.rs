use super::*;


impl<K,V> ErasedMap<K,V>
where 
    K:Hash+Eq
{
    pub unsafe fn as_hashmap(&self)->&HashMap<MapKey<K>,V>{
        transmute_reference(self)
    }
    pub unsafe fn as_mut_hashmap(&mut self)->&mut HashMap<MapKey<K>,V>{
        transmute_mut_reference(self)
    }

    fn run<'a,F,R>(&'a self,f:F)->R
    where F:FnOnce(&'a HashMap<MapKey<K>,V>)->R
    {
        extern_fn_panic_handling!{
            let map=unsafe{ self.as_hashmap() };
            f( map )
        }
    }
    
    fn run_mut<'a,F,R>(&'a mut self,f:F)->R
    where F:FnOnce(&'a mut HashMap<MapKey<K>,V>)->R
    {
        extern_fn_panic_handling!{
            let map=unsafe{ self.as_mut_hashmap() };
            f( map )
        }
    }

    pub(super)extern fn get_elem(&self,key:MapQuery<'_,K>)->Option<&V>{
        self.run(|this|unsafe{ 
            this.get(&key.as_mapkey()) 
        })
    }    

    pub(super)extern fn get_mut_elem(&mut self,key:MapQuery<'_,K>)->Option<&mut V>{
        self.run_mut(|this|unsafe{ 
            this.get_mut(&key.as_mapkey()) 
        })
    }

    pub(super)extern fn insert_elem(&mut self,key:K,value:V)->ROption<V>{
        self.run_mut(|this|{
            this.insert(MapKey::Value(key),value)
                .into_c()
        })
    }

    pub(super)extern fn remove_entry(&mut self,key:MapQuery<'_,K>)->ROption<Tuple2<K,V>>{
        self.run_mut(|this|{
            match this.remove_entry(unsafe{ &key.as_mapkey() }) {
                Some(x)=>RSome(Tuple2(x.0.into_inner(),x.1)),
                None=>RNone,
            }
        })
    }

    pub(super)extern fn clear_map(&mut self){
        self.run_mut(|this| this.clear() )
    }

    pub(super)extern fn len(&self)->usize{
        self.run(|this| this.len() )
    }

}



