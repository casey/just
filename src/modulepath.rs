use super::*;

#[derive(Debug, Default, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub(crate) struct Modulepath {
  pub(crate) components: Vec<String>,
  pub(crate) spaced: bool,
}

impl Modulepath {
  pub(crate) fn is_empty(&self) -> bool {
    self.components.is_empty()
  }

  pub(crate) fn starts_with(&self, other: &Modulepath) -> bool {
    self.components.starts_with(&other.components)
  }
}

impl Serialize for Modulepath {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl From<&Namepath<'_>> for Modulepath {
  fn from(namepath: &Namepath) -> Self {
    Self {
      components: namepath.iter().map(|name| name.lexeme().into()).collect(),
      spaced: false,
    }
  }
}

impl TryFrom<&[&str]> for Modulepath {
  type Error = ();

  fn try_from(path: &[&str]) -> Result<Self, Self::Error> {
    let spaced = path.len() > 1;

    let components = if path.len() == 1 {
      let first = path[0];

      if first.starts_with(':') || first.ends_with(':') || first.contains(":::") {
        return Err(());
      }

      first
        .split("::")
        .map(str::to_string)
        .collect::<Vec<String>>()
    } else {
      path.iter().map(|s| (*s).to_string()).collect()
    };

    for name in &components {
      if name.is_empty() {
        return Err(());
      }

      for (i, c) in name.chars().enumerate() {
        if i == 0 {
          if !Lexer::is_identifier_start(c) {
            return Err(());
          }
        } else if !Lexer::is_identifier_continue(c) {
          return Err(());
        }
      }
    }

    Ok(Self { components, spaced })
  }
}

impl Display for Modulepath {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for (i, name) in self.components.iter().enumerate() {
      if i > 0 {
        if self.spaced {
          write!(f, " ")?;
        } else {
          write!(f, "::")?;
        }
      }
      write!(f, "{name}")?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn try_from_ok() {
    #[track_caller]
    fn case(path: &[&str], expected: &[&str], display: &str) {
      let actual = Modulepath::try_from(path).unwrap();
      assert_eq!(actual.path, expected);
      assert_eq!(actual.to_string(), display);
    }

    case(&[], &[], "");
    case(&["foo"], &["foo"], "foo");
    case(&["foo0"], &["foo0"], "foo0");
    case(&["foo", "bar"], &["foo", "bar"], "foo bar");
    case(&["foo::bar"], &["foo", "bar"], "foo::bar");
  }

  #[test]
  fn try_from_err() {
    #[track_caller]
    fn case(path: &[&str]) {
      assert!(Modulepath::try_from(path).is_err());
    }

    case(&[":foo"]);
    case(&["foo:"]);
    case(&["foo:::bar"]);
    case(&["0foo"]);
    case(&["f$oo"]);
    case(&[""]);
  }
}
