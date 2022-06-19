use super::*;

pub(crate) trait Keyed<'key> {
  fn key(&self) -> &'key str;
}

impl<'key, T: Keyed<'key>> Keyed<'key> for Rc<T> {
  fn key(&self) -> &'key str {
    self.as_ref().key()
  }
}

pub(crate) fn serialize<'src, S, K>(keyed: &K, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
  K: Keyed<'src>,
{
  serializer.serialize_str(keyed.key())
}

pub(crate) fn serialize_option<'src, S, K>(
  recipe: &Option<K>,
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: Serializer,
  K: Keyed<'src>,
{
  match recipe {
    None => serializer.serialize_none(),
    Some(keyed) => serialize(keyed, serializer),
  }
}
