use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Value {
  elements: Vec<String>,
}

impl Value {
  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn elements(&self) -> &[String] {
    &self.elements
  }

  pub(crate) fn join(&self) -> Cow<'_, str> {
    match self.elements.as_slice() {
      [element] => Cow::Borrowed(element),
      elements => Cow::Owned(elements.join(" ")),
    }
  }

  pub(crate) fn into_string(self) -> String {
    if self.elements.len() == 1 {
      self.elements.into_iter().next().unwrap()
    } else {
      self.elements.join(" ")
    }
  }

  pub(crate) fn is_empty(&self) -> bool {
    match self.elements.as_slice() {
      [] => true,
      [element] => element.is_empty(),
      _ => false,
    }
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for (i, element) in self.elements.iter().enumerate() {
      if i > 0 {
        write!(f, " ")?;
      }
      write!(f, "{element}")?;
    }
    Ok(())
  }
}

impl Serialize for Value {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.join())
  }
}

impl From<&String> for Value {
  fn from(element: &String) -> Self {
    Self {
      elements: vec![element.clone()],
    }
  }
}

impl From<&str> for Value {
  fn from(element: &str) -> Self {
    element.to_string().into()
  }
}

impl From<String> for Value {
  fn from(element: String) -> Self {
    Self {
      elements: vec![element],
    }
  }
}

impl FromIterator<String> for Value {
  fn from_iter<I: IntoIterator<Item = String>>(elements: I) -> Self {
    Self {
      elements: elements.into_iter().collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn join() {
    #[track_caller]
    fn case(elements: &[&str], expected: &str) {
      let value = elements.iter().map(ToString::to_string).collect::<Value>();
      assert_eq!(value.join(), expected);
      assert_eq!(value.to_string(), expected);
      assert_eq!(value.clone().into_string(), expected);
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
    fn case(elements: &[&str], expected: bool) {
      let value = elements.iter().map(ToString::to_string).collect::<Value>();
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
    assert_eq!(Value::from("foo bar").elements(), ["foo bar"]);
    assert_eq!(Value::from(String::from("foo")).elements(), ["foo"]);
    assert_eq!(Value::new().elements(), [] as [&str; 0]);
  }
}
