use super::*;

#[derive(Clone)]
pub enum MapKey<K> {
    Value(K),
    /// This is a horrible hack.
    Query(NonNull<MapQuery<'static, K>>),
}

impl<K> MapKey<K> {
    #[inline]
    pub fn into_inner(self) -> K {
        match self {
            MapKey::Value(v) => v,
            _ => unreachable!("This is a BUG!!!!!!!!!!!!!!!!!!!!"),
        }
    }

    #[inline]
    pub fn as_ref(&self) -> &K {
        match self {
            MapKey::Value(v) => v,
            _ => unreachable!("This is a BUG!!!!!!!!!!!!!!!!!!!!"),
        }
    }
}

impl<K> From<K> for MapKey<K> {
    #[inline]
    fn from(value: K) -> Self {
        MapKey::Value(value)
    }
}

impl<K> Debug for MapKey<K>
where
    K: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapKey::Value(v) => Debug::fmt(v, f),
            _ => unreachable!("This is a BUG!!!!!!!!!!!!!!!!!!!!"),
        }
    }
}

impl<K> Eq for MapKey<K> where K: Eq {}

impl<K> PartialEq for MapKey<K>
where
    K: PartialEq,
{
    fn eq<'a>(&'a self, other: &'a Self) -> bool {
        match (self, other) {
            (MapKey::Value(lhs), MapKey::Query(rhs)) | (MapKey::Query(rhs), MapKey::Value(lhs)) => unsafe {
                rhs.as_ref().is_equal(lhs)
            },
            (MapKey::Value(lhs), MapKey::Value(rhs)) => lhs == rhs,
            _ => {
                unreachable!("This is a BUG!!!!!!!!!!!!!!!!!!!!!!!!");
            }
        }
    }
}

impl<K> Hash for MapKey<K>
where
    K: Hash,
{
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        match self {
            MapKey::Value(this) => {
                this.hash(hasher);
            }
            MapKey::Query(this) => unsafe {
                this.as_ref().hash(hasher);
            },
        }
    }
}

impl<K> Borrow<K> for MapKey<K> {
    fn borrow(&self) -> &K {
        self.as_ref()
    }
}
