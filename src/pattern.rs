use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Pattern<'src> {
  pub(crate) regex: Regex,
  pub(crate) token: Token<'src>,
}

impl<'src> Pattern<'src> {
  pub(crate) fn is_match(&self, haystack: &str) -> bool {
    self.regex.is_match(haystack)
  }

  pub(crate) fn new(literal: &StringLiteral<'src>) -> Result<Self, CompileError<'src>> {
    literal.cooked.parse::<Regex>().map_err(|source| {
      literal
        .token
        .error(CompileErrorKind::ArgumentPatternRegex { source })
    })?;

    Ok(Self {
      regex: format!("^(?:{})$", literal.cooked)
        .parse::<Regex>()
        .map_err(|source| {
          literal
            .token
            .error(CompileErrorKind::ArgumentPatternRegex { source })
        })?,
      token: literal.token,
    })
  }

  pub(crate) fn original(&self) -> &str {
    &self.regex.as_str()[4..self.regex.as_str().len() - 2]
  }
}

impl Eq for Pattern<'_> {}

impl Ord for Pattern<'_> {
  fn cmp(&self, other: &pattern::Pattern) -> Ordering {
    self.regex.as_str().cmp(other.regex.as_str())
  }
}

impl PartialEq for Pattern<'_> {
  fn eq(&self, other: &pattern::Pattern) -> bool {
    self.regex.as_str() == other.regex.as_str()
  }
}

impl PartialOrd for Pattern<'_> {
  fn partial_cmp(&self, other: &pattern::Pattern) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Serialize for Pattern<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.original())
  }
}
