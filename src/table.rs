use crate::common::*;

use std::collections::btree_map;

#[derive(Debug, PartialEq)]
pub(crate) struct Table<'key, V: Keyed<'key>> {
  map: BTreeMap<&'key str, V>,
}

impl<'key, V: Keyed<'key>> Table<'key, V> {
  pub(crate) fn insert(&mut self, value: V) {
    self.map.insert(value.key(), value);
  }

  pub(crate) fn len(&self) -> usize {
    self.map.len()
  }

  pub(crate) fn get(&self, key: &str) -> Option<&V> {
    self.map.get(key)
  }

  pub(crate) fn values(&self) -> btree_map::Values<&'key str, V> {
    self.map.values()
  }

  pub(crate) fn contains_key(&self, key: &str) -> bool {
    self.map.contains_key(key)
  }

  pub(crate) fn keys(&self) -> btree_map::Keys<&'key str, V> {
    self.map.keys()
  }

  pub(crate) fn iter(&self) -> btree_map::Iter<&'key str, V> {
    self.map.iter()
  }
}

impl<'key, V: Keyed<'key>> FromIterator<V> for Table<'key, V> {
  fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
    Table {
      map: iter.into_iter().map(|value| (value.key(), value)).collect(),
    }
  }
}

impl<'key, V: Keyed<'key>> Index<&'key str> for Table<'key, V> {
  type Output = V;

  #[inline]
  fn index(&self, key: &str) -> &V {
    self.map.get(key).expect("no entry found for key")
  }
}

impl<'key, V: Keyed<'key>> IntoIterator for Table<'key, V> {
  type Item = (&'key str, V);
  type IntoIter = btree_map::IntoIter<&'key str, V>;

  fn into_iter(self) -> btree_map::IntoIter<&'key str, V> {
    self.map.into_iter()
  }
}

impl<'table, V: Keyed<'table> + 'table> IntoIterator for &'table Table<'table, V> {
  type Item = (&'table &'table str, &'table V);
  type IntoIter = btree_map::Iter<'table, &'table str, V>;

  fn into_iter(self) -> btree_map::Iter<'table, &'table str, V> {
    self.map.iter()
  }
}
