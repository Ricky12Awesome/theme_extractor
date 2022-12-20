use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use serde::{
  de::{MapAccess, Visitor},
  Deserialize, Deserializer,
};

struct SkipNullsMap<K, V> {
  marker: PhantomData<(K, V)>,
}

impl<K, V> SkipNullsMap<K, V> {
  fn new() -> Self {
    Self {
      marker: PhantomData,
    }
  }
}

impl<'de, K, V> Visitor<'de> for SkipNullsMap<K, V>
where
  K: Deserialize<'de> + Eq + Hash,
  V: Deserialize<'de>,
{
  type Value = HashMap<K, V>;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    // don't know a good message to put here
    formatter.write_str("a map")
  }

  fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

    while let Some((key, Some(value))) = access.next_entry::<K, Option<V>>()? {
      map.insert(key, value);
    }

    Ok(map)
  }
}

pub fn skip_nulls_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
  D: Deserializer<'de>,
  K: Deserialize<'de> + Eq + Hash,
  V: Deserialize<'de>,
{
  deserializer.deserialize_map(SkipNullsMap::new())
}
