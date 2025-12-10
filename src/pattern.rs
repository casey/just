use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Pattern(pub(crate) Regex);

impl Pattern {
  pub(crate) fn is_match(&self, haystack: &str) -> bool {
    self.0.is_match(haystack)
  }

  pub(crate) fn new<'src>(
    token: Token<'src>,
    literal: &StringLiteral,
  ) -> Result<Self, CompileError<'src>> {
    literal
      .cooked
      .parse::<Regex>()
      .map_err(|source| token.error(CompileErrorKind::ArgumentPatternRegex { source }))?;

    Ok(Self(
      format!("^({})$", literal.cooked)
        .parse::<Regex>()
        .map_err(|source| token.error(CompileErrorKind::ArgumentPatternRegex { source }))?,
    ))
  }
}

impl Display for Pattern {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", &self.0.as_str()[2..self.0.as_str().len() - 2])
  }
}

impl Eq for Pattern {}

impl Ord for Pattern {
  fn cmp(&self, other: &pattern::Pattern) -> Ordering {
    self.0.as_str().cmp(other.0.as_str())
  }
}

impl PartialEq for Pattern {
  fn eq(&self, other: &pattern::Pattern) -> bool {
    self.0.as_str() == other.0.as_str()
  }
}

impl PartialOrd for Pattern {
  fn partial_cmp(&self, other: &pattern::Pattern) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Serialize for Pattern {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.0.as_str())
  }
}
