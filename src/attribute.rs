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

  /// Returns a range from the min to max expected arguments of a given attribute
  pub(crate) fn expect_args(&self) -> Range<usize> {
    use Attribute::*;

    match self {
      Confirm(_) => 1..2,
      _ => 0..0,
    }
  }

  pub(crate) fn with_arguments(
    self,
    arguments: Vec<StringLiteral<'_>>,
  ) -> Result<Attribute, CompileErrorKind<'_>> {
    use Attribute::*;

    if !self.expect_args().range_contains(&arguments.len()) {
      return Err(CompileErrorKind::AttributeArgumentCountMismatch {
        attribute: self.to_str(),
        found: arguments.len(),
        expected: self.expect_args(),
      });
    }

    match self {
      Confirm(_) => Ok(Attribute::Confirm(
        arguments.first().map(|s| s.cooked.clone()),
      )),
      _ => unreachable!("Missing implementation for attribute that accepts arguments"),
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
