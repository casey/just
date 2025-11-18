use super::*;

/// A struct containing the parsed representation of positional command-line
/// arguments, i.e. arguments that are not flags, options, or the subcommand.
///
/// The DSL of positional arguments is fairly complex and mostly accidental.
/// There are three possible components: overrides, a search directory, and the
/// rest:
///
/// - Overrides are of the form `NAME=.*`
///
/// - After overrides comes a single optional search directory argument. This is
///   either '.', '..', or an argument that contains a `/`.
///
///   If the argument contains a `/`, everything before and including the slash
///   is the search directory, and everything after is added to the rest.
///
/// - Everything else is an argument.
///
/// Overrides set the values of top-level variables in the justfile being
/// invoked and are a convenient way to override settings.
///
/// For modes that do not take other arguments, the search directory argument
/// determines where to begin searching for the justfile.  This allows command
/// lines like `just -l ..` and `just ../build` to find the same justfile.
///
/// For modes that do take other arguments, the search argument is simply
/// prepended to rest.
#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Positional {
  /// Everything else
  pub arguments: Vec<String>,
  /// Overrides from values of the form `[a-zA-Z_][a-zA-Z0-9_-]*=.*`
  pub overrides: Vec<(String, String)>,
  /// An argument equal to '.', '..', or ending with `/`
  pub search_directory: Option<String>,
}

impl Positional {
  pub fn from_values<'values>(values: Option<impl IntoIterator<Item = &'values str>>) -> Self {
    let mut overrides = Vec::new();
    let mut search_directory = None;
    let mut arguments = Vec::new();

    if let Some(values) = values {
      for value in values {
        if search_directory.is_none() && arguments.is_empty() {
          if let Some(o) = Self::override_from_value(value) {
            overrides.push(o);
          } else if value == "." || value == ".." {
            search_directory = Some(value.to_owned());
          } else if let Some(i) = value.rfind('/') {
            let (dir, tail) = value.split_at(i + 1);

            search_directory = Some(dir.to_owned());

            if !tail.is_empty() {
              arguments.push(tail.to_owned());
            }
          } else {
            arguments.push(value.to_owned());
          }
        } else {
          arguments.push(value.to_owned());
        }
      }
    }

    Self {
      arguments,
      overrides,
      search_directory,
    }
  }

  /// Parse an override from a value of the form `NAME=.*`.
  fn override_from_value(value: &str) -> Option<(String, String)> {
    let equals = value.find('=')?;

    let (identifier, equals_value) = value.split_at(equals);

    // exclude `=` from value
    let value = &equals_value[1..];

    if Lexer::is_identifier(identifier) {
      Some((identifier.to_owned(), value.to_owned()))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use pretty_assertions::assert_eq;

  #[test]
  fn no_values() {
    assert_eq!(
      Positional::from_values(Some([].iter().copied())),
      Positional {
        overrides: vec![],
        search_directory: None,
        arguments: vec![],
      },
    );
  }

  #[test]
  fn arguments_only() {
    assert_eq!(
      Positional::from_values(Some(["foo", "bar"].iter().copied())),
      Positional {
        overrides: vec![],
        search_directory: None,
        arguments: vec!["foo".to_owned(), "bar".to_owned()],
      },
    );
  }

  #[test]
  fn all_overrides() {
    assert_eq!(
      Positional::from_values(Some(["foo=bar", "bar=foo"].iter().copied())),
      Positional {
        overrides: vec![
          ("foo".to_owned(), "bar".to_owned()),
          ("bar".to_owned(), "foo".to_owned())
        ],
        search_directory: None,
        arguments: vec![],
      },
    );
  }

  #[test]
  fn override_not_name() {
    assert_eq!(
      Positional::from_values(Some(["foo=bar", "bar.=foo"].iter().copied())),
      Positional {
        overrides: vec![("foo".to_owned(), "bar".to_owned())],
        search_directory: None,
        arguments: vec!["bar.=foo".to_owned()],
      },
    );
  }

  #[test]
  fn no_overrides() {
    assert_eq!(
      Positional::from_values(Some(["the-dir/", "baz", "bzzd"].iter().copied())),
      Positional {
        overrides: vec![],
        search_directory: Some("the-dir/".to_owned()),
        arguments: vec!["baz".to_owned(), "bzzd".to_owned()],
      },
    );
  }

  #[test]
  fn no_search_directory() {
    assert_eq!(
      Positional::from_values(Some(["foo=bar", "bar=foo", "baz", "bzzd"].iter().copied())),
      Positional {
        overrides: vec![
          ("foo".to_owned(), "bar".to_owned()),
          ("bar".to_owned(), "foo".to_owned())
        ],
        search_directory: None,
        arguments: vec!["baz".to_owned(), "bzzd".to_owned()],
      },
    );
  }

  #[test]
  fn no_arguments() {
    assert_eq!(
      Positional::from_values(Some(["foo=bar", "bar=foo", "the-dir/"].iter().copied())),
      Positional {
        overrides: vec![
          ("foo".to_owned(), "bar".to_owned()),
          ("bar".to_owned(), "foo".to_owned())
        ],
        search_directory: Some("the-dir/".to_owned()),
        arguments: vec![],
      },
    );
  }

  #[test]
  fn all_dot() {
    assert_eq!(
      Positional::from_values(Some(["foo=bar", "bar=foo", ".", "garnor"].iter().copied())),
      Positional {
        overrides: vec![
          ("foo".to_owned(), "bar".to_owned()),
          ("bar".to_owned(), "foo".to_owned())
        ],
        search_directory: Some(".".to_owned()),
        arguments: vec!["garnor".to_owned()],
      },
    );
  }

  #[test]
  fn all_dot_dot() {
    assert_eq!(
      Positional::from_values(Some(["foo=bar", "bar=foo", "..", "garnor"].iter().copied())),
      Positional {
        overrides: vec![
          ("foo".to_owned(), "bar".to_owned()),
          ("bar".to_owned(), "foo".to_owned())
        ],
        search_directory: Some("..".to_owned()),
        arguments: vec!["garnor".to_owned()],
      },
    );
  }

  #[test]
  fn all_slash() {
    assert_eq!(
      Positional::from_values(Some(["foo=bar", "bar=foo", "/", "garnor"].iter().copied())),
      Positional {
        overrides: vec![
          ("foo".to_owned(), "bar".to_owned()),
          ("bar".to_owned(), "foo".to_owned())
        ],
        search_directory: Some("/".to_owned()),
        arguments: vec!["garnor".to_owned()],
      },
    );
  }

  #[test]
  fn search_directory_after_argument() {
    assert_eq!(
      Positional::from_values(Some(
        ["foo=bar", "bar=foo", "baz", "bzzd", "bar/"]
          .iter()
          .copied()
      )),
      Positional {
        overrides: vec![
          ("foo".to_owned(), "bar".to_owned()),
          ("bar".to_owned(), "foo".to_owned())
        ],
        search_directory: None,
        arguments: vec!["baz".to_owned(), "bzzd".to_owned(), "bar/".to_owned()],
      },
    );
  }

  #[test]
  fn override_after_search_directory() {
    assert_eq!(
      Positional::from_values(Some(["..", "a=b"].iter().copied())),
      Positional {
        overrides: vec![],
        search_directory: Some("..".to_owned()),
        arguments: vec!["a=b".to_owned()],
      },
    );
  }

  #[test]
  fn override_after_argument() {
    assert_eq!(
      Positional::from_values(Some(["a", "a=b"].iter().copied())),
      Positional {
        overrides: vec![],
        search_directory: None,
        arguments: vec!["a".to_owned(), "a=b".to_owned()],
      },
    );
  }
}
