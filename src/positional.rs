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
  /// Overrides from values of the form `[a-zA-Z_][a-zA-Z0-9_-]*=.*`
  pub overrides: Vec<(String, String)>,
  /// An argument equal to '.', '..', or ending with `/`
  pub search_directory: Option<String>,
  /// Everything else
  pub arguments: Vec<String>,
}

impl Positional {
  pub fn from_values<'values>(values: Option<impl IntoIterator<Item = String>>) -> Self {
    let mut overrides = Vec::new();
    let mut search_directory = None;
    let mut arguments = Vec::new();

    if let Some(values) = values {
      for value in values {
        if search_directory.is_none() && arguments.is_empty() {
          if let Some(o) = Self::override_from_value(&value) {
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
      overrides,
      search_directory,
      arguments,
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

  macro_rules! test {
    {
      name: $name:ident,
      values: $vals:expr,
      overrides: $overrides:expr,
      search_directory: $search_directory:expr,
      arguments: $arguments:expr,
    } => {
      #[test]
      fn $name() {
        assert_eq! (
          Positional::from_values(Some($vals.iter().cloned())),
          Positional {
            overrides: $overrides
              .iter()
              .cloned()
              .map(|(key, value): (&str, &str)| (key.to_owned(), value.to_owned()))
              .collect(),
            search_directory: $search_directory.map(str::to_owned),
            arguments: $arguments.iter().cloned().map(str::to_owned).collect(),
          },
        )
      }
    }
  }

  test! {
    name: no_values,
    values: [],
    overrides: [],
    search_directory: None,
    arguments: [],
  }

  test! {
    name: arguments_only,
    values: ["foo", "bar"],
    overrides: [],
    search_directory: None,
    arguments: ["foo", "bar"],
  }

  test! {
    name: all_overrides,
    values: ["foo=bar", "bar=foo"],
    overrides: [("foo", "bar"), ("bar", "foo")],
    search_directory: None,
    arguments: [],
  }

  test! {
    name: override_not_name,
    values: ["foo=bar", "bar.=foo"],
    overrides: [("foo", "bar")],
    search_directory: None,
    arguments: ["bar.=foo"],
  }

  test! {
    name: no_overrides,
    values: ["the-dir/", "baz", "bzzd"],
    overrides: [],
    search_directory: Some("the-dir/"),
    arguments: ["baz", "bzzd"],
  }

  test! {
    name: no_search_directory,
    values: ["foo=bar", "bar=foo", "baz", "bzzd"],
    overrides: [("foo", "bar"), ("bar", "foo")],
    search_directory: None,
    arguments: ["baz", "bzzd"],
  }

  test! {
    name: no_arguments,
    values: ["foo=bar", "bar=foo", "the-dir/"],
    overrides: [("foo", "bar"), ("bar", "foo")],
    search_directory: Some("the-dir/"),
    arguments: [],
  }

  test! {
    name: all_dot,
    values: ["foo=bar", "bar=foo", ".", "garnor"],
    overrides: [("foo", "bar"), ("bar", "foo")],
    search_directory: Some("."),
    arguments: ["garnor"],
  }

  test! {
    name: all_dot_dot,
    values: ["foo=bar", "bar=foo", "..", "garnor"],
    overrides: [("foo", "bar"), ("bar", "foo")],
    search_directory: Some(".."),
    arguments: ["garnor"],
  }

  test! {
    name: all_slash,
    values: ["foo=bar", "bar=foo", "/", "garnor"],
    overrides: [("foo", "bar"), ("bar", "foo")],
    search_directory: Some("/"),
    arguments: ["garnor"],
  }

  test! {
    name: search_directory_after_argument,
    values: ["foo=bar", "bar=foo", "baz", "bzzd", "bar/"],
    overrides: [("foo", "bar"), ("bar", "foo")],
    search_directory: None,
    arguments: ["baz", "bzzd", "bar/"],
  }

  test! {
    name: override_after_search_directory,
    values: ["..", "a=b"],
    overrides: [],
    search_directory: Some(".."),
    arguments: ["a=b"],
  }

  test! {
    name: override_after_argument,
    values: ["a", "a=b"],
    overrides: [],
    search_directory: None,
    arguments: ["a", "a=b"],
  }
}
