use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Value {
  parts: Vec<String>,
}

impl Value {
  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn parts(&self) -> &[String] {
    &self.parts
  }

  pub(crate) fn joined(&self) -> Cow<'_, str> {
    match self.parts.as_slice() {
      [part] => Cow::Borrowed(part),
      parts => Cow::Owned(parts.join(" ")),
    }
  }

  pub(crate) fn into_joined(self) -> String {
    if self.parts.len() == 1 {
      self.parts.into_iter().next().unwrap()
    } else {
      self.parts.join(" ")
    }
  }

  pub(crate) fn is_empty(&self) -> bool {
    match self.parts.as_slice() {
      [] => true,
      [part] => part.is_empty(),
      _ => false,
    }
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for (i, part) in self.parts.iter().enumerate() {
      if i > 0 {
        write!(f, " ")?;
      }
      write!(f, "{part}")?;
    }
    Ok(())
  }
}

impl Serialize for Value {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.joined())
  }
}

impl From<&String> for Value {
  fn from(part: &String) -> Self {
    Self {
      parts: vec![part.clone()],
    }
  }
}

impl From<&str> for Value {
  fn from(part: &str) -> Self {
    part.to_string().into()
  }
}

impl From<String> for Value {
  fn from(part: String) -> Self {
    Self { parts: vec![part] }
  }
}

impl FromIterator<String> for Value {
  fn from_iter<I: IntoIterator<Item = String>>(parts: I) -> Self {
    Self {
      parts: parts.into_iter().collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn joined() {
    #[track_caller]
    fn case(parts: &[&str], expected: &str) {
      let value = parts.iter().map(ToString::to_string).collect::<Value>();
      assert_eq!(value.joined(), expected);
      assert_eq!(value.to_string(), expected);
      assert_eq!(value.clone().into_joined(), expected);
      assert_eq!(
        serde_json::to_string(&value).unwrap(),
        format!("{expected:?}")
      );
    }

    case(&[], "");
    case(&[""], "");
    case(&["foo"], "foo");
    case(&["foo", "bar"], "foo bar");
    case(&["foo bar", "baz"], "foo bar baz");
    case(&["", ""], " ");
  }

  #[test]
  fn is_empty() {
    #[track_caller]
    fn case(parts: &[&str], expected: bool) {
      let value = parts.iter().map(ToString::to_string).collect::<Value>();
      assert_eq!(value.is_empty(), expected);
    }

    case(&[], true);
    case(&[""], true);
    case(&["foo"], false);
    case(&["", ""], false);
    case(&["foo", "bar"], false);
  }

  #[test]
  fn from_str() {
    assert_eq!(Value::from("foo bar").parts(), ["foo bar"]);
    assert_eq!(Value::from(String::from("foo")).parts(), ["foo"]);
    assert_eq!(Value::new().parts(), [] as [&str; 0]);
  }
}
