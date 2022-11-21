use super::*;

/// A trait object used in method that access map entries without replacing them.
#[derive(StableAbi)]
#[repr(C)]
pub struct MapQuery<'a, K> {
    _marker: NotCopyNotClone,
    is_equal: extern "C" fn(&K, RRef<'_, ErasedObject>) -> bool,
    hash: extern "C" fn(RRef<'_, ErasedObject>, HasherObject<'_>),
    query: RRef<'a, ErasedObject>,
}

impl<'a, K> MapQuery<'a, K> {
    #[inline]
    pub(super) fn new<Q>(query: &'a &'a Q) -> Self
    where
        K: Borrow<Q>,
        Q: Hash + Eq + 'a + ?Sized,
    {
        MapQuery {
            _marker: NotCopyNotClone,
            is_equal: is_equal::<K, Q>,
            hash: hash::<Q>,
            query: unsafe { RRef::new(query).transmute() },
        }
    }

    #[inline]
    pub(super) unsafe fn as_static(&self) -> &MapQuery<'static, K> {
        unsafe { crate::utils::transmute_reference(self) }
    }
}

impl<'a, K> MapQuery<'a, K> {
    #[inline]
    pub(super) fn is_equal(&self, other: &K) -> bool {
        (self.is_equal)(other, self.query)
    }

    #[inline]
    pub(super) unsafe fn as_mapkey(&self) -> MapKey<K> {
        MapKey::Query(NonNull::from(unsafe { self.as_static() }))
    }
}

impl<'a, K> Hash for MapQuery<'a, K> {
    #[inline]
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        (self.hash)(self.query, HasherObject::new(hasher))
    }
}

extern "C" fn is_equal<K, Q>(key: &K, query: RRef<'_, ErasedObject>) -> bool
where
    K: Borrow<Q>,
    Q: Eq + ?Sized,
{
    extern_fn_panic_handling! {
        let query = unsafe{ query.transmute_into_ref::<&Q>() };
        key.borrow() == *query
    }
}

extern "C" fn hash<Q>(query: RRef<'_, ErasedObject>, mut hasher: HasherObject<'_>)
where
    Q: Hash + ?Sized,
{
    extern_fn_panic_handling! {
        let query = unsafe{ query.transmute_into_ref::<&Q>() };
        query.hash(&mut hasher);
    }
}
