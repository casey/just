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

  pub(crate) fn apply(&self, other: &Self, operator: ListOperator) -> Option<Self> {
    let separator = operator.separator();
    let (a, b) = (&self.elements, &other.elements);
    match (a.len(), b.len()) {
      (m, n) if m == n => Some(
        a.iter()
          .zip(b)
          .map(|(a, b)| format!("{a}{separator}{b}"))
          .collect(),
      ),
      (1, n) if n >= 2 => Some(
        b.iter()
          .map(|b| format!("{}{separator}{b}", a[0]))
          .collect(),
      ),
      (m, 1) if m >= 2 => Some(
        a.iter()
          .map(|a| format!("{a}{separator}{}", b[0]))
          .collect(),
      ),
      _ => None,
    }
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

impl From<bool> for Value {
  fn from(condition: bool) -> Self {
    if condition {
      "true".into()
    } else {
      Self::new()
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
  fn color_display() {
    assert_eq!(
      Value::from("abc\t\r\nxyz")
        .color_display(Color::always())
        .to_string(),
      "\u{1b}[32m\"abc\u{1b}[36m\\t\\r\\n\u{1b}[32mxyz\"\u{1b}[0m",
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

  #[test]
  fn concatenate() {
    use ListOperator::{Concatenate, Join};

    #[track_caller]
    fn case(a: &[&str], b: &[&str], operator: ListOperator, expected: Option<&[&str]>) {
      let a = a.iter().map(ToString::to_string).collect::<Value>();
      let b = b.iter().map(ToString::to_string).collect::<Value>();
      let expected = expected.map(|expected| expected.iter().map(ToString::to_string).collect());
      assert_eq!(a.apply(&b, operator), expected);
    }

    case(&[], &[], Concatenate, Some(&[]));
    case(&["a"], &["b"], Concatenate, Some(&["ab"]));
    case(&["a"], &["b"], Join, Some(&["a/b"]));
    case(&["a", "b"], &["c", "d"], Concatenate, Some(&["ac", "bd"]));
    case(&["a", "b"], &["c", "d"], Join, Some(&["a/c", "b/d"]));
    case(&["a"], &["b", "c"], Concatenate, Some(&["ab", "ac"]));
    case(
      &["a"],
      &["b", "c", "d"],
      Concatenate,
      Some(&["ab", "ac", "ad"]),
    );
    case(&["a", "b"], &["c"], Join, Some(&["a/c", "b/c"]));
    case(
      &["a", "b", "c"],
      &["d"],
      Concatenate,
      Some(&["ad", "bd", "cd"]),
    );

    case(&[], &["a"], Concatenate, None);
    case(&["a"], &[], Concatenate, None);
    case(&[], &["a", "b"], Concatenate, None);
    case(&["a", "b"], &[], Concatenate, None);
    case(&["a", "b"], &["c", "d", "e"], Concatenate, None);
  }
}
