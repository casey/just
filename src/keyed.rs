use crate::common::*;

pub(crate) trait Keyed<'key> {
  fn key(&self) -> &'key str;
}

impl<'key, T: Keyed<'key>> Keyed<'key> for Rc<T> {
  fn key(&self) -> &'key str {
    self.as_ref().key()
  }
}
