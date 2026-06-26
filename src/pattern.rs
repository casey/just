use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Pattern {
  pub(crate) regex: Regex,
}

impl Pattern {
  pub(crate) fn is_match(&self, haystack: &str) -> bool {
    self.regex.is_match(haystack)
  }

  pub(crate) fn new<'src>(value: &str, key: Name<'src>) -> Result<Self, CompileError<'src>> {
    value
      .parse::<Regex>()
      .map_err(|source| key.error(CompileErrorKind::ArgumentPatternRegex { source }))?;

    Ok(Self {
      regex: format!("^(?:{value})$")
        .parse()
        .map_err(|source| key.error(CompileErrorKind::ArgumentPatternRegex { source }))?,
    })
  }

  pub(crate) fn original(&self) -> &str {
    self
      .regex
      .as_str()
      .strip_prefix("^(?:")
      .unwrap()
      .strip_suffix(")$")
      .unwrap()
  }
}

impl Eq for Pattern {}

impl Ord for Pattern {
  fn cmp(&self, other: &pattern::Pattern) -> Ordering {
    self.regex.as_str().cmp(other.regex.as_str())
  }
}

impl PartialEq for Pattern {
  fn eq(&self, other: &pattern::Pattern) -> bool {
    self.regex.as_str() == other.regex.as_str()
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
    serializer.serialize_str(self.original())
  }
}
