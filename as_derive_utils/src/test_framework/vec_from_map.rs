use std::{fmt, marker::PhantomData};

use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};

/// Used to deserialize a list of key-value pairs from a json object.
#[derive(Clone, Debug)]
pub(crate) struct VecFromMap<K, V> {
    pub(crate) vec: Vec<(K, V)>,
}

struct VecFromMapVisitor<K, V> {
    marker: PhantomData<(K, V)>,
}

impl<K, V> VecFromMapVisitor<K, V> {
    const NEW: Self = Self {
        marker: PhantomData,
    };
}

impl<'de, K, V> Visitor<'de> for VecFromMapVisitor<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = VecFromMap<K, V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let cap = access.size_hint().unwrap_or(0);
        let mut vec = Vec::<(K, V)>::with_capacity(cap);

        while let Some(pair) = access.next_entry()? {
            vec.push(pair);
        }

        Ok(VecFromMap { vec })
    }
}

impl<'de, K, V> Deserialize<'de> for VecFromMap<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(VecFromMapVisitor::NEW)
    }
}

pub(crate) fn deserialize_vec_pairs<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    VecFromMap<K, V>: Deserialize<'de>,
{
    VecFromMap::<K, V>::deserialize(deserializer).map(|x| x.vec)
}
