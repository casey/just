use super::*;

#[derive(EnumString, PartialEq, Debug, Clone, Serialize, Ord, PartialOrd, Eq, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Attribute {
  Confirm(Option<String>),
  Linux,
  Macos,
  NoCd,
  NoExitMessage,
  Private,
  NoQuiet,
  Unix,
  Windows,
}

impl Attribute {
  pub(crate) fn from_name(name: Name) -> Option<Attribute> {
    name.lexeme().parse().ok()
  }

  pub(crate) fn to_str(&self) -> &'static str {
    self.into()
  }

  pub(crate) fn with_argument(
    self,
    arguments: StringLiteral<'_>,
  ) -> Result<Attribute, CompileErrorKind<'_>> {
    use Attribute::*;

    match self {
      Confirm(_) => Ok(Attribute::Confirm(Some(arguments.cooked.clone()))),
      // Return error for all attributes that don't accept arguments
      _ => Err(CompileErrorKind::AttributeArgumentCountMismatch {
        attribute: self.to_str(),
        found: 1,
        expected: 0..=0,
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn to_str() {
    assert_eq!(Attribute::NoExitMessage.to_str(), "no-exit-message");
  }
}
