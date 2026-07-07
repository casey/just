use super::*;

#[derive(Clone)]
pub(crate) struct ListEntry<'a> {
  pub(crate) aliases: &'a [&'a str],
  pub(crate) doc: Option<Cow<'a, str>>,
  pub(crate) name: &'a str,
  pub(crate) recipe: &'a Recipe<'a>,
}
