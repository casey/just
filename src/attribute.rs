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

  pub(crate) fn expect_args(&self) -> Range<usize> {
    use Attribute::*;

    match self {
      Confirm(_) => 1..1,
      _ => 0..0,
    }
  }

  pub(crate) fn with_arguments(self, arguments: Vec<StringLiteral<'_>>) -> Result<Attribute, CompileErrorKind<'_>> {
    use Attribute::*;

    match self {
      Confirm(_) if arguments.len() == 1 => Ok(Attribute::Confirm(arguments.first().map(|s| s.cooked.clone()))),
      _ => Err(CompileErrorKind::AttributeArgumentCountMismatch { attribute: self.to_str(), found: arguments.len(), expected: self.expect_args() }),
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
