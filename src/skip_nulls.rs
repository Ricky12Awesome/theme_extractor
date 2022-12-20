use std::marker::PhantomData;

use itertools::Itertools;
use serde::{
  de::{MapAccess, Visitor},
  Deserialize, Deserializer,
};

struct SkipNullsMap<K, V, T> {
  _marker: PhantomData<(K, V, T)>,
}

impl<K, V, T> SkipNullsMap<K, V, T> {
  fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

struct MapAccessEntryIter<'de, M, K, V> {
  access: M,
  _marker: PhantomData<(K, V, &'de ())>,
}

impl<'de, M, K, V> MapAccessEntryIter<'de, M, K, V> {
  fn from(access: M) -> Self {
    Self {
      access,
      _marker: PhantomData,
    }
  }
}

impl<'de, M, K, V> Iterator for MapAccessEntryIter<'de, M, K, V>
where
  M: MapAccess<'de>,
  K: Deserialize<'de>,
  V: Deserialize<'de>,
{
  type Item = Result<(K, V), M::Error>;

  fn next(&mut self) -> Option<Self::Item> {
    self.access.next_entry().transpose()
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (0, self.access.size_hint())
  }
}

impl<'de, K, V, T> Visitor<'de> for SkipNullsMap<K, V, T>
where
  K: Deserialize<'de>,
  V: Deserialize<'de>,
  T: FromIterator<(K, V)>,
{
  type Value = T;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    // don't know a good message to put here
    formatter.write_str("a key-value tuple")
  }

  fn visit_map<M>(self, access: M) -> Result<Self::Value, M::Error>
  where
    M: MapAccess<'de>,
  {
    MapAccessEntryIter::<'de, M, Option<K>, Option<V>>::from(access)
      .filter_map_ok(|(k, v)| Some((k?, v?)))
      .collect()
  }
}

pub fn skip_nulls<'de, D, K, V, T>(deserializer: D) -> Result<T, D::Error>
where
  D: Deserializer<'de>,
  K: Deserialize<'de>,
  V: Deserialize<'de>,
  T: FromIterator<(K, V)>,
{
  deserializer.deserialize_map(SkipNullsMap::new())
}
