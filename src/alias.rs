use super::*;

/// An alias, e.g. `name := target`
#[derive(Debug, PartialEq, Clone, Serialize)]
pub(crate) struct Alias<'src, T = Rc<Recipe<'src>>> {
  pub(crate) attributes: AttributeSet<'src>,
  pub(crate) name: Name<'src>,
  #[serde(
    bound(serialize = "T: Keyed<'src>"),
    serialize_with = "keyed::serialize"
  )]
  pub(crate) target: T,
}

impl<'src> Alias<'src, Namepath<'src>> {
  pub(crate) fn resolve(self, target: Rc<Recipe<'src>>) -> Alias<'src> {
    assert!(self.target.last().lexeme() == target.namepath.last().lexeme());

    Alias {
      attributes: self.attributes,
      name: self.name,
      target,
    }
  }
}

impl Alias<'_> {
  pub(crate) fn is_private(&self) -> bool {
    self.name.lexeme().starts_with('_') || self.attributes.contains(AttributeDiscriminant::Private)
  }
}

impl<'src, T> Keyed<'src> for Alias<'src, T> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}

impl<'src> Display for Alias<'src, Namepath<'src>> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "alias {} := {}", self.name.lexeme(), self.target)
  }
}

impl Display for Alias<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "alias {} := {}",
      self.name.lexeme(),
      self.target.name.lexeme()
    )
  }
}
