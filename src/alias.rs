use super::*;

/// An alias, e.g. `alias name := target`
#[derive(Debug, PartialEq, Clone, Serialize)]
pub(crate) struct Alias<'src, T = Namepath<'src>> {
  pub(crate) attributes: AttributeSet<'src>,
  pub(crate) name: Name<'src>,
  #[serde(
    bound(serialize = "T: Keyed<'src>"),
    serialize_with = "keyed::serialize"
  )]
  pub(crate) target: T,
}

impl<'src> Alias<'src> {
  pub(crate) fn resolve(self, target: Arc<Recipe<'src>>) -> RecipeAlias<'src> {
    assert!(self.target.last().lexeme() == target.name());

    Alias {
      attributes: self.attributes,
      name: self.name,
      target,
    }
  }
}

impl<'src> RecipeAlias<'src> {
  pub(crate) fn is_public(&self) -> bool {
    !self.name.lexeme().starts_with('_') && !self.attributes.contains(AttributeKind::Private)
  }
}

impl<'src, T> Keyed<'src> for Alias<'src, T> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}

impl<'src> Display for Alias<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "alias {} := {}", self.name.lexeme(), self.target)
  }
}

impl<'src> Display for RecipeAlias<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "alias {} := {}",
      self.name.lexeme(),
      self.target.name.lexeme()
    )
  }
}
