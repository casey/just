use super::*;

#[allow(clippy::doc_markdown)]
/// The argument parser is responsible for grouping positional arguments into
/// argument groups, which consist of a path to a recipe and its arguments.
///
/// Argument parsing is substantially complicated by the fact that recipe paths
/// can be given on the command line as multiple arguments, i.e., "foo" "bar"
/// baz", or as a single "::"-separated argument.
///
/// Error messages produced by the argument parser should use the format of the
/// recipe path as passed on the command line.
///
/// Additionally, if a recipe is specified with a "::"-separated path, extra
/// components of that path after a valid recipe must not be used as arguments,
/// whereas arguments after multiple argument path may be used as arguments. As
/// an example, `foo bar baz` may refer to recipe `foo::bar` with argument
/// `baz`, but `foo::bar::baz` is an error, since `bar` is a recipe, not a
/// module.
pub(crate) struct ArgumentParser<'src: 'run, 'run> {
  arguments: &'run [&'run str],
  next: usize,
  root: &'run Justfile<'src>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct ArgumentGroup<'run> {
  pub(crate) positional_arguments: Vec<&'run str>,
  pub(crate) flag_arguments: BTreeMap<String, Option<&'run str>>,
  pub(crate) path: Vec<String>,
}

impl<'src: 'run, 'run> ArgumentParser<'src, 'run> {
  pub(crate) fn parse_arguments(
    root: &'run Justfile<'src>,
    arguments: &'run [&'run str],
  ) -> RunResult<'src, Vec<ArgumentGroup<'run>>> {
    let mut groups = Vec::new();

    let mut invocation_parser = Self {
      arguments,
      next: 0,
      root,
    };

    loop {
      groups.push(invocation_parser.parse_group()?);

      if invocation_parser.next == arguments.len() {
        break;
      }
    }

    Ok(groups)
  }

  fn parse_group(&mut self) -> RunResult<'src, ArgumentGroup<'run>> {
    let (recipe, path) = if let Some(next) = self.next() {
      if next.contains(':') {
        let module_path =
          ModulePath::try_from([next].as_slice()).map_err(|()| Error::UnknownRecipe {
            recipe: next.into(),
            suggestion: None,
          })?;
        let (recipe, path, _) = self.resolve_recipe(true, &module_path.path)?;
        self.next += 1;
        (recipe, path)
      } else {
        let (recipe, path, consumed) = self.resolve_recipe(false, self.rest())?;
        self.next += consumed;
        (recipe, path)
      }
    } else {
      let (recipe, path, consumed) = self.resolve_recipe(false, self.rest())?;
      assert_eq!(consumed, 0);
      (recipe, path)
    };

    let (consumed, positional_arguments, flag_arguments) =
      Self::parse_recipe_arguments(recipe, self.rest())?;
    self.next += consumed;

    Ok(ArgumentGroup {
      positional_arguments,
      flag_arguments,
      path,
    })
  }

  fn parse_recipe_arguments(
    recipe: &'run Recipe<'src>,
    args: &[&'run str],
  ) -> RunResult<'src, (usize, Vec<&'run str>, BTreeMap<String, Option<&'run str>>)> {
    // if the recipie does not contain flags, use the original parsing behavior
    let mut verbatim = recipe.flags.is_empty();

    let min_positional = recipe.min_arguments();
    let max_positional = recipe.max_arguments();

    let mut consumed = 0;
    let mut positional_arguments = Vec::new();
    let mut flag_arguments = BTreeMap::new();
    loop {
      if consumed >= args.len() {
        break;
      }

      let current = args[consumed];

      // allow trailing flags
      if (!current.starts_with("--") || verbatim) && positional_arguments.len() >= max_positional {
        break;
      }

      if verbatim {
        positional_arguments.push(current);
        consumed += 1;
      } else if current == "--" {
        verbatim = true;
        consumed += 1;
      } else if let Some(flag) = current.strip_prefix("--") {
        let Some(parameter) = recipe.flags.get(flag) else {
          return Err(Error::internal("TODO: fix error message, unknown flag"));
        };
        if parameter.default.is_some() {
          if consumed + 1 >= args.len() {
            return Err(Error::internal(
              "TODO: fix error message, missing option argument",
            ));
          }
          flag_arguments.insert(parameter.name.to_string(), Some(args[consumed + 1]));
          consumed += 2;
        } else {
          flag_arguments.insert(parameter.name.to_string(), None);
          consumed += 1;
        }
      } else {
        positional_arguments.push(current);
        consumed += 1;
      }
    }

    if positional_arguments.len() < min_positional {
      return Err(Error::ArgumentCountMismatch {
        recipe: recipe.name(),
        parameters: recipe.parameters.clone(),
        found: positional_arguments.len(),
        min: recipe.min_arguments(),
        max: recipe.max_arguments(),
      });
    }

    Ok((consumed, positional_arguments, flag_arguments))
  }

  fn resolve_recipe(
    &self,
    module_path: bool,
    args: &[impl AsRef<str>],
  ) -> RunResult<'src, (&'run Recipe<'src>, Vec<String>, usize)> {
    let mut current = self.root;
    let mut path = Vec::new();

    for (i, arg) in args.iter().enumerate() {
      let arg = arg.as_ref();

      path.push(arg.to_string());

      if let Some(module) = current.modules.get(arg) {
        current = module;
      } else if let Some(recipe) = current.get_recipe(arg) {
        if module_path && i + 1 < args.len() {
          return Err(Error::ExpectedSubmoduleButFoundRecipe {
            path: if module_path {
              path.join("::")
            } else {
              path.join(" ")
            },
          });
        }
        return Ok((recipe, path, i + 1));
      } else {
        if module_path && i + 1 < args.len() {
          return Err(Error::UnknownSubmodule {
            path: path.join("::"),
          });
        }

        return Err(Error::UnknownRecipe {
          recipe: if module_path {
            path.join("::")
          } else {
            path.join(" ")
          },
          suggestion: current.suggest_recipe(arg),
        });
      }
    }

    if let Some(recipe) = &current.default {
      recipe.check_can_be_default_recipe()?;
      path.push(recipe.name().into());
      Ok((recipe, path, args.len()))
    } else if current.recipes.is_empty() {
      Err(Error::NoRecipes)
    } else {
      Err(Error::NoDefaultRecipe)
    }
  }

  fn next(&self) -> Option<&'run str> {
    self.arguments.get(self.next).copied()
  }

  fn rest(&self) -> &[&'run str] {
    &self.arguments[self.next..]
  }
}

#[cfg(test)]
mod tests {
  use {super::*, tempfile::TempDir};

  trait TempDirExt {
    fn write(&self, path: &str, content: &str);
  }

  impl TempDirExt for TempDir {
    fn write(&self, path: &str, content: &str) {
      let path = self.path().join(path);
      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, content).unwrap();
    }
  }

  #[test]
  fn single_no_arguments() {
    let justfile = testing::compile("foo:");

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        positional_arguments: Vec::new(),
        flag_arguments: BTreeMap::new(),
      }],
    );
  }

  #[test]
  fn single_with_argument() {
    let justfile = testing::compile("foo bar:");

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "baz"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        positional_arguments: vec!["baz"],
        flag_arguments: BTreeMap::new(),
      }],
    );
  }

  #[test]
  fn single_argument_count_mismatch() {
    let justfile = testing::compile("foo bar:");

    assert_matches!(
      ArgumentParser::parse_arguments(&justfile, &["foo"]).unwrap_err(),
      Error::ArgumentCountMismatch {
        recipe: "foo",
        found: 0,
        min: 1,
        max: 1,
        ..
      },
    );
  }

  #[test]
  fn single_unknown() {
    let justfile = testing::compile("foo:");

    assert_matches!(
      ArgumentParser::parse_arguments(&justfile, &["bar"]).unwrap_err(),
      Error::UnknownRecipe {
        recipe,
        suggestion: None
      } if recipe == "bar",
    );
  }

  #[test]
  fn multiple_unknown() {
    let justfile = testing::compile("foo:");

    assert_matches!(
      ArgumentParser::parse_arguments(&justfile, &["bar", "baz"]).unwrap_err(),
      Error::UnknownRecipe {
        recipe,
        suggestion: None
      } if recipe == "bar",
    );
  }

  #[test]
  fn recipe_in_submodule() {
    let loader = Loader::new();
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("justfile");
    fs::write(&path, "mod foo").unwrap();
    fs::create_dir(tempdir.path().join("foo")).unwrap();
    fs::write(tempdir.path().join("foo/mod.just"), "bar:").unwrap();
    let compilation = Compiler::compile(&loader, &path).unwrap();

    assert_eq!(
      ArgumentParser::parse_arguments(&compilation.justfile, &["foo", "bar"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into(), "bar".into()],
        positional_arguments: Vec::new(),
        flag_arguments: BTreeMap::new(),
      }],
    );
  }

  #[test]
  fn recipe_in_submodule_unknown() {
    let loader = Loader::new();
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("justfile");
    fs::write(&path, "mod foo").unwrap();
    fs::create_dir(tempdir.path().join("foo")).unwrap();
    fs::write(tempdir.path().join("foo/mod.just"), "bar:").unwrap();
    let compilation = Compiler::compile(&loader, &path).unwrap();

    assert_matches!(
      ArgumentParser::parse_arguments(&compilation.justfile, &["foo", "zzz"]).unwrap_err(),
      Error::UnknownRecipe {
        recipe,
        suggestion: None
      } if recipe == "foo zzz",
    );
  }

  #[test]
  fn recipe_in_submodule_path_unknown() {
    let tempdir = tempfile::tempdir().unwrap();
    tempdir.write("justfile", "mod foo");
    tempdir.write("foo.just", "bar:");

    let loader = Loader::new();
    let compilation = Compiler::compile(&loader, &tempdir.path().join("justfile")).unwrap();

    assert_matches!(
      ArgumentParser::parse_arguments(&compilation.justfile, &["foo::zzz"]).unwrap_err(),
      Error::UnknownRecipe {
        recipe,
        suggestion: None
      } if recipe == "foo::zzz",
    );
  }

  #[test]
  fn module_path_not_consumed() {
    let tempdir = tempfile::tempdir().unwrap();
    tempdir.write("justfile", "mod foo");
    tempdir.write("foo.just", "bar:");

    let loader = Loader::new();
    let compilation = Compiler::compile(&loader, &tempdir.path().join("justfile")).unwrap();

    assert_matches!(
      ArgumentParser::parse_arguments(&compilation.justfile, &["foo::bar::baz"]).unwrap_err(),
      Error::ExpectedSubmoduleButFoundRecipe {
        path,
      } if path == "foo::bar",
    );
  }

  #[test]
  fn no_recipes() {
    let tempdir = tempfile::tempdir().unwrap();
    tempdir.write("justfile", "");

    let loader = Loader::new();
    let compilation = Compiler::compile(&loader, &tempdir.path().join("justfile")).unwrap();

    assert_matches!(
      ArgumentParser::parse_arguments(&compilation.justfile, &[]).unwrap_err(),
      Error::NoRecipes,
    );
  }

  #[test]
  fn default_recipe_requires_arguments() {
    let tempdir = tempfile::tempdir().unwrap();
    tempdir.write("justfile", "foo bar:");

    let loader = Loader::new();
    let compilation = Compiler::compile(&loader, &tempdir.path().join("justfile")).unwrap();

    assert_matches!(
      ArgumentParser::parse_arguments(&compilation.justfile, &[]).unwrap_err(),
      Error::DefaultRecipeRequiresArguments {
        recipe: "foo",
        min_arguments: 1,
      },
    );
  }

  #[test]
  fn no_default_recipe() {
    let tempdir = tempfile::tempdir().unwrap();
    tempdir.write("justfile", "import 'foo.just'");
    tempdir.write("foo.just", "bar:");

    let loader = Loader::new();
    let compilation = Compiler::compile(&loader, &tempdir.path().join("justfile")).unwrap();

    assert_matches!(
      ArgumentParser::parse_arguments(&compilation.justfile, &[]).unwrap_err(),
      Error::NoDefaultRecipe,
    );
  }

  #[test]
  fn complex_grouping() {
    let justfile = testing::compile(
      "
FOO A B='blarg':
  echo foo: {{A}} {{B}}

BAR X:
  echo bar: {{X}}

BAZ +Z:
  echo baz: {{Z}}
",
    );

    assert_eq!(
      ArgumentParser::parse_arguments(
        &justfile,
        &["BAR", "0", "FOO", "1", "2", "BAZ", "3", "4", "5"]
      )
      .unwrap(),
      vec![
        ArgumentGroup {
          path: vec!["BAR".into()],
          positional_arguments: vec!["0"],
          flag_arguments: BTreeMap::new(),
        },
        ArgumentGroup {
          path: vec!["FOO".into()],
          positional_arguments: vec!["1", "2"],
          flag_arguments: BTreeMap::new(),
        },
        ArgumentGroup {
          path: vec!["BAZ".into()],
          positional_arguments: vec!["3", "4", "5"],
          flag_arguments: BTreeMap::new(),
        },
      ],
    );
  }

  #[test]
  fn options() {
    let justfile = testing::compile(
      "
RULE --flag --option='foo' positional:
  echo {{ flag }} {{ option }} {{ positional }}
",
    );

    fn argument_group<'a, const M: usize, const N: usize>(
      items: [&'a str; M],
      kv: [(&'a str, Option<&'a str>); N],
    ) -> ArgumentGroup<'a> {
      ArgumentGroup {
        path: vec!["RULE".into()],
        positional_arguments: items.to_vec(),
        flag_arguments: kv.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
      }
    }

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["RULE", "1"]).unwrap(),
      vec![argument_group(["1"], [])],
    );

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["RULE", "--flag", "1"]).unwrap(),
      vec![argument_group(["1"], [("flag", None)])],
    );

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["RULE", "--option", "1", "2"]).unwrap(),
      vec![argument_group(["2"], [("option", Some("1"))])],
    );

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["RULE", "--option", "1", "--flag", "2"])
        .unwrap(),
      vec![argument_group(
        ["2"],
        [("option", Some("1")), ("flag", None)]
      )],
    );

    // options swallow arguments
    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["RULE", "--option", "--flag", "2"]).unwrap(),
      vec![argument_group(["2"], [("option", Some("--flag"))])],
    );

    // arguments can be parsed in any order
    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["RULE", "2", "--option", "1", "--flag"])
        .unwrap(),
      vec![argument_group(
        ["2"],
        [("option", Some("1")), ("flag", None)]
      )],
    );

    // -- allows to parse unknown arguments
    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["RULE", "--", "--foo"]).unwrap(),
      vec![argument_group(["--foo"], [])],
    );

    // -- resets for each recipie
    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["RULE", "--", "--foo", "RULE", "--flag", "2"])
        .unwrap(),
      vec![
        argument_group(["--foo"], []),
        argument_group(["2"], [("flag", None)]),
      ],
    );

    // unknown arguments are forbidden
    ArgumentParser::parse_arguments(&justfile, &["RULE", "2", "--foo"]).unwrap_err();
  }
}
