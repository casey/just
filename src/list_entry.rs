use super::*;

#[derive(Debug, Clone)]
pub(crate) struct ListEntry<'src, 'outer> {
  pub(crate) prefix: String,
  pub(crate) recipe: &'outer Recipe<'src>,
  pub(crate) aliases: Vec<&'src str>,
}

impl<'src, 'outer> ListEntry<'src, 'outer> {
  pub(crate) fn from_recipe(
    recipe: &'outer Recipe<'src>,
    prefix: String,
    aliases: Vec<&'src str>,
  ) -> Self {
    Self {
      prefix,
      recipe,
      aliases,
    }
  }
}
