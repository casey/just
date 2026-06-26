use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Pattern {
  pub(crate) regexes: Vec<Regex>,
}

impl Pattern {
  pub(crate) fn is_match(&self, haystack: &str) -> bool {
    self.regexes.iter().any(|regex| regex.is_match(haystack))
  }

  pub(crate) fn new<'src>(value: &Value, key: Name<'src>) -> Result<Self, CompileError<'src>> {
    let regexes = value
      .elements()
      .iter()
      .map(|element| {
        element
          .parse::<Regex>()
          .map_err(|source| key.error(CompileErrorKind::ArgumentPatternRegex { source }))?;

        format!("^(?:{element})$")
          .parse()
          .map_err(|source| key.error(CompileErrorKind::ArgumentPatternRegex { source }))
      })
      .collect::<Result<Vec<Regex>, CompileError>>()?;

    Ok(Self { regexes })
  }

  pub(crate) fn originals(&self) -> impl Iterator<Item = &str> + Clone {
    self.regexes.iter().map(|regex| {
      regex
        .as_str()
        .strip_prefix("^(?:")
        .unwrap()
        .strip_suffix(")$")
        .unwrap()
    })
  }
}

impl Eq for Pattern {}

impl Ord for Pattern {
  fn cmp(&self, other: &pattern::Pattern) -> Ordering {
    self
      .regexes
      .iter()
      .map(Regex::as_str)
      .cmp(other.regexes.iter().map(Regex::as_str))
  }
}

impl PartialEq for Pattern {
  fn eq(&self, other: &pattern::Pattern) -> bool {
    self
      .regexes
      .iter()
      .map(Regex::as_str)
      .eq(other.regexes.iter().map(Regex::as_str))
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
    serializer.collect_seq(self.originals())
  }
}
