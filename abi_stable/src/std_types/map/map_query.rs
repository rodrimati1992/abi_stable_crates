use super::*;

/// A trait object used in method that access map entries without replacing them.
#[derive(StableAbi)]
#[repr(C)]
#[sabi(inside_abi_stable_crate)]
pub struct MapQuery<'a,K>{
    _marker:NotCopyNotClone,
    is_equal:extern fn(&K,&ErasedObject)->bool,
    hash    :extern "C" fn(&ErasedObject,HasherObject<'_>),
    query:&'a ErasedObject,
}

impl<'a,K> MapQuery<'a,K>{
    #[inline]
    pub(super) fn new<Q>(query:&'a &'a Q)->Self
    where 
        K:Borrow<Q>,
        Q:Hash + Eq + 'a+?Sized,
    {
        MapQuery{
            _marker:NotCopyNotClone,
            is_equal:is_equal::<K,Q>,
            hash    :hash::<Q>,
            query:unsafe{ transmute_reference(query) },
        }
    }

    #[inline]
    pub(super) unsafe fn to_static(self)->MapQuery<'static,K>{
        mem::transmute::<MapQuery<'a,K>,MapQuery<'static,K>>(self)
    }
    
    #[inline]
    pub(super) unsafe fn as_static(&self)->&MapQuery<'static,K>{
        transmute_reference(self)
    }
}

impl<'a,K> MapQuery<'a,K>{
    #[inline]
    pub(super) fn is_equal(&self,other:&K)->bool{
        (self.is_equal)(other,self.query)
    }

    #[inline]
    pub(super) unsafe fn as_mapkey(&self)->MapKey<K>{
        MapKey::Query(NonNull::from(self.as_static()))
    }
}


impl<'a,K> Hash for MapQuery<'a,K>{
    #[inline]
    fn hash<H>(&self,hasher:&mut H)
    where
        H: Hasher,
    {
        (self.hash)(&self.query,HasherObject::new(hasher))
    }
}


extern fn is_equal<K,Q>(key:&K,query:&ErasedObject)->bool
where
    K:Borrow<Q>,
    Q:Eq+?Sized,
{
    extern_fn_panic_handling!{
        let query =unsafe{ transmute_reference::<ErasedObject,&Q>(query) };
        key.borrow()==*query
    }
}


extern fn hash<Q>(query:&ErasedObject,mut hasher:HasherObject<'_>)
where
    Q:Hash+?Sized,
{
    extern_fn_panic_handling!{
        let query =unsafe{ transmute_reference::<ErasedObject,&Q>(query) };
        query.hash(&mut hasher);
    }
}
