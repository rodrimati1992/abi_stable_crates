use std::{
    fmt,
    marker::PhantomData,
};

use serde::de::{Deserialize, Deserializer, Visitor, MapAccess};


/// Used to deserialize a json object to a list of key-value pairs
#[derive(Clone,Debug)]
pub struct VecFromMap<K,V>{
    pub vec:Vec<(K,V)>,
}


struct VecFromMapVisitor<K, V> {
    marker: PhantomData<(K, V)>
}

impl<K, V> VecFromMapVisitor<K, V> {
    const NEW:Self=Self{marker:PhantomData};
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
        let cap=access.size_hint().unwrap_or(0);
        let mut vec = Vec::<(K,V)>::with_capacity(cap);

        while let Some(pair) = access.next_entry()? {
            vec.push(pair);
        }

        Ok(VecFromMap{vec})
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
