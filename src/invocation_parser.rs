use super::*;

#[allow(clippy::doc_markdown)]
/// The invocation parser is responsible for grouping command-line positional
/// arguments into invocations, which consist of a recipe and its arguments.
///
/// Invocation parsing is substantially complicated by the fact that recipe
/// paths can be given on the command line as multiple arguments, i.e., "foo"
/// "bar" baz", or as a single "::"-separated argument.
///
/// Error messages produced by the invocation parser should use the format of
/// the recipe path as passed on the command line.
///
/// Additionally, if a recipe is specified with a "::"-separated path, extra
/// components of that path after a valid recipe must not be used as arguments,
/// whereas arguments after multiple argument path may be used as arguments. As
/// an example, `foo bar baz` may refer to recipe `foo::bar` with argument
/// `baz`, but `foo::bar::baz` is an error, since `bar` is a recipe, not a
/// module.
pub(crate) struct InvocationParser<'src: 'run, 'run> {
  arguments: &'run [&'run str],
  next: usize,
  root: &'run Justfile<'src>,
}

impl<'src: 'run, 'run> InvocationParser<'src, 'run> {
  pub(crate) fn parse_invocations(
    root: &'run Justfile<'src>,
    arguments: &'run [&'run str],
  ) -> RunResult<'src, Vec<Invocation<'src, 'run>>> {
    let mut invocations = Vec::new();

    let mut invocation_parser = Self {
      arguments,
      next: 0,
      root,
    };

    loop {
      invocations.push(invocation_parser.parse_invocation()?);

      if invocation_parser.next == arguments.len() {
        break;
      }
    }

    Ok(invocations)
  }

  fn parse_invocation(&mut self) -> RunResult<'src, Invocation<'src, 'run>> {
    let recipe = if let Some(next) = self.next() {
      if next.contains(':') {
        let module_path =
          ModulePath::try_from([next].as_slice()).map_err(|()| Error::UnknownRecipe {
            recipe: next.into(),
            suggestion: None,
          })?;
        let (recipe, _) = self.resolve_recipe(true, &module_path.path)?;
        self.next += 1;
        recipe
      } else {
        let (recipe, consumed) = self.resolve_recipe(false, self.rest())?;
        self.next += consumed;
        recipe
      }
    } else {
      let (recipe, consumed) = self.resolve_recipe(false, self.rest())?;
      assert_eq!(consumed, 0);
      recipe
    };

    let rest = self.rest();

    let argument_range = recipe.argument_range();
    let argument_count = cmp::min(rest.len(), recipe.max_arguments());
    if !argument_range.range_contains(&argument_count) {
      return Err(Error::ArgumentCountMismatch {
        recipe: recipe.name(),
        parameters: recipe.parameters.clone(),
        found: rest.len(),
        min: recipe.min_arguments(),
        max: recipe.max_arguments(),
      });
    }

    let arguments = recipe.group_arguments(&rest[..argument_count]);

    for (group, parameter) in arguments.iter().zip(&recipe.parameters) {
      for argument in group {
        parameter.check_pattern_match(recipe, argument)?;
      }
    }

    self.next += argument_count;

    Ok(Invocation {
      arguments: arguments
        .into_iter()
        .map(|group| group.into_iter().map(str::to_string).collect())
        .collect(),
      recipe,
    })
  }

  fn resolve_recipe(
    &self,
    module_path: bool,
    args: &[impl AsRef<str>],
  ) -> RunResult<'src, (&'run Recipe<'src>, usize)> {
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
            path: path.join("::"),
          });
        }
        return Ok((recipe, i + 1));
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
      Ok((recipe, args.len()))
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

    let invocations = InvocationParser::parse_invocations(&justfile, &["foo"]).unwrap();

    assert_eq!(invocations.len(), 1);
    assert_eq!(invocations[0].recipe.namepath(), "foo");
    assert!(invocations[0].arguments.is_empty());
  }

  #[test]
  fn single_with_argument() {
    let justfile = testing::compile("foo bar:");

    let invocations = InvocationParser::parse_invocations(&justfile, &["foo", "baz"]).unwrap();

    assert_eq!(invocations.len(), 1);
    assert_eq!(invocations[0].recipe.namepath(), "foo");
    assert_eq!(invocations[0].arguments, vec![vec![String::from("baz")]]);
  }

  #[test]
  fn single_argument_count_mismatch() {
    let justfile = testing::compile("foo bar:");

    assert_matches!(
      InvocationParser::parse_invocations(&justfile, &["foo"]).unwrap_err(),
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
      InvocationParser::parse_invocations(&justfile, &["bar"]).unwrap_err(),
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
      InvocationParser::parse_invocations(&justfile, &["bar", "baz"]).unwrap_err(),
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

    let invocations =
      InvocationParser::parse_invocations(&compilation.justfile, &["foo", "bar"]).unwrap();

    assert_eq!(invocations.len(), 1);
    assert_eq!(invocations[0].recipe.namepath(), "foo::bar");
    assert!(invocations[0].arguments.is_empty());
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
      InvocationParser::parse_invocations(&compilation.justfile, &["foo", "zzz"]).unwrap_err(),
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
      InvocationParser::parse_invocations(&compilation.justfile, &["foo::zzz"]).unwrap_err(),
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
      InvocationParser::parse_invocations(&compilation.justfile, &["foo::bar::baz"]).unwrap_err(),
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
      InvocationParser::parse_invocations(&compilation.justfile, &[]).unwrap_err(),
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
      InvocationParser::parse_invocations(&compilation.justfile, &[]).unwrap_err(),
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
      InvocationParser::parse_invocations(&compilation.justfile, &[]).unwrap_err(),
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

    let invocations = InvocationParser::parse_invocations(
      &justfile,
      &["BAR", "0", "FOO", "1", "2", "BAZ", "3", "4", "5"],
    )
    .unwrap();

    assert_eq!(invocations.len(), 3);
    assert_eq!(invocations[0].recipe.namepath(), "BAR");
    assert_eq!(invocations[0].arguments, vec![vec![String::from("0")]]);
    assert_eq!(invocations[1].recipe.namepath(), "FOO");
    assert_eq!(
      invocations[1].arguments,
      vec![vec![String::from("1")], vec![String::from("2")]]
    );
    assert_eq!(invocations[2].recipe.namepath(), "BAZ");
    assert_eq!(
      invocations[2].arguments,
      vec![vec![
        String::from("3"),
        String::from("4"),
        String::from("5")
      ]]
    );
  }
}
