use super::*;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
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

  pub(crate) fn push(&mut self, element: &str) {
    self.elements.push(element.into());
  }

  pub(crate) fn join(&self) -> String {
    self.elements.join(" ")
  }

  pub(crate) fn into_elements(self) -> Vec<String> {
    self.elements
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.elements.is_empty()
  }

  pub(crate) fn is_truthy(&self) -> bool {
    !self.elements.is_empty()
  }
}

impl ColorDisplay for Value {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    if self.elements.len() == 1 {
      write!(f, "{}", Element(&self.elements[0]).color_display(color))
    } else {
      write!(f, "[")?;

      for (i, element) in self.elements.iter().enumerate() {
        if i > 0 {
          write!(f, ", ")?;
        }

        write!(f, "{}", Element(element).color_display(color))?;
      }

      write!(f, "]")
    }
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

impl From<Vec<String>> for Value {
  fn from(elements: Vec<String>) -> Self {
    Self { elements }
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
      assert_eq!(value.clone().join(), expected);
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
  fn display() {
    #[track_caller]
    fn case(elements: &[&str], expected: &str) {
      let value = elements.iter().map(ToString::to_string).collect::<Value>();
      assert_eq!(value.color_display(Color::never()).to_string(), expected);
    }

    case(&[], "[]");
    case(&["foo"], r#""foo""#);
    case(&["foo", "bar"], r#"["foo", "bar"]"#);
    case(&["a\tb\"c"], r#""a\tb\"c""#);
    case(&["\\", "\n"], r#"["\\", "\n"]"#);
  }

  #[test]
  fn display_color() {
    assert_eq!(
      Value::from("a\tb")
        .color_display(Color::always())
        .to_string(),
      "\u{1b}[32m\"a\u{1b}[36m\\t\u{1b}[32mb\"\u{1b}[0m",
    );
  }

  #[test]
  fn is_truthy() {
    #[track_caller]
    fn case(elements: &[&str], expected: bool) {
      let value = elements.iter().map(ToString::to_string).collect::<Value>();
      assert_eq!(value.is_truthy(), expected);
    }

    case(&[], false);
    case(&[""], true);
    case(&["foo"], true);
    case(&["", ""], true);
    case(&["foo", "bar"], true);
  }

  #[test]
  fn from_str() {
    assert_eq!(Value::from("foo bar").elements(), ["foo bar"]);
    assert_eq!(Value::from(String::from("foo")).elements(), ["foo"]);
    assert_eq!(Value::new().elements(), [] as [&str; 0]);
  }
}
