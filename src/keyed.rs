pub(crate) trait Keyed<'key> {
  fn key(&self) -> &'key str;
}
