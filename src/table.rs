use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Table<'key, V: Keyed<'key>> {
  map: BTreeMap<&'key str, V>,
}

impl<'key, V: Keyed<'key>> Table<'key, V> {
  pub(crate) fn insert(&mut self, value: V) {
    self.map.insert(value.key(), value);
  }
}

impl<'key, V: Keyed<'key>> FromIterator<V> for Table<'key, V> {
  fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
    Table {
      map: iter.into_iter().map(|value| (value.key(), value)).collect(),
    }
  }
}

impl<'key, V: Keyed<'key>> Deref for Table<'key, V> {
  type Target = BTreeMap<&'key str, V>;

  fn deref(&self) -> &Self::Target {
    &self.map
  }
}

impl<'key, V: Keyed<'key>> IntoIterator for Table<'key, V> {
  type Item = (&'key str, V);
  type IntoIter = std::collections::btree_map::IntoIter<&'key str, V>;

  fn into_iter(self) -> std::collections::btree_map::IntoIter<&'key str, V> {
    self.map.into_iter()
  }
}

impl<'table, V: Keyed<'table> + 'table> IntoIterator for &'table Table<'table, V> {
  type Item = (&'table &'table str, &'table V);
  type IntoIter = std::collections::btree_map::Iter<'table, &'table str, V>;

  fn into_iter(self) -> std::collections::btree_map::Iter<'table, &'table str, V> {
    self.map.iter()
  }
}
