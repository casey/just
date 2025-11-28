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
  pub(crate) arguments: Vec<&'run str>,
  pub(crate) flags: BTreeMap<String, Option<String>>,
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

    let rest = self.rest();
    let mut positional = Vec::new();
    let mut flags = BTreeMap::new();
    let mut consumed = 0;
    let mut force_positional = false;

    while consumed < rest.len() {
      let token = rest[consumed];

      if token == "--" {
        force_positional = true;
        consumed += 1;
        continue;
      }

      if !force_positional && token.starts_with("--") {
        let (flag_name, flag_value) = if let Some(eq_pos) = token.find('=') {
          (&token[2..eq_pos], Some(&token[eq_pos + 1..]))
        } else {
          (&token[2..], None)
        };

        if let Some(flag_spec) = recipe.flags.get(flag_name) {
          if flags.contains_key(flag_name) {
            return Err(Error::DuplicateFlag {
              recipe: recipe.name().to_string(),
              flag: flag_name.to_string(),
            });
          }

          match flag_spec.arity {
            FlagArity::Switch => {
              if flag_value.is_some() {
                return Err(Error::UnexpectedFlagValue {
                  recipe: recipe.name().to_string(),
                  flag: flag_name.to_string(),
                });
              }
              flags.insert(flag_name.to_string(), Some("true".to_string()));
              consumed += 1;
            }
            FlagArity::WithValue => {
              let value = if let Some(v) = flag_value {
                v.to_string()
              } else {
                consumed += 1;
                if consumed < rest.len() && !rest[consumed].starts_with("--") {
                  let v = rest[consumed].to_string();
                  consumed += 1;
                  v
                } else {
                  return Err(Error::MissingFlagValue {
                    recipe: recipe.name().to_string(),
                    flag: flag_name.to_string(),
                  });
                }
              };
              flags.insert(flag_name.to_string(), Some(value));
              if flag_value.is_some() {
                consumed += 1;
              }
            }
          }
        } else {
          let suggestion = recipe
            .flags
            .keys()
            .filter_map(|name| {
              let distance = edit_distance::edit_distance(flag_name, name);
              (distance < 3).then_some((distance, name))
            })
            .min_by_key(|(distance, _)| *distance)
            .map(|(_, name)| name.clone());

          return Err(Error::UnknownFlag {
            recipe: recipe.name().to_string(),
            flag: flag_name.to_string(),
            suggestion,
          });
        }
        continue;
      }

      if positional.len() >= recipe.max_arguments() {
        break;
      }

      if positional.len() >= recipe.min_arguments()
        && consumed < rest.len()
        && self.resolve_recipe(false, &rest[consumed..]).is_ok()
      {
        break;
      }

      positional.push(token);
      consumed += 1;
    }

    let argument_range = recipe.argument_range();
    if !argument_range.range_contains(&positional.len()) {
      return Err(Error::ArgumentCountMismatch {
        recipe: recipe.name(),
        parameters: recipe.parameters.clone(),
        found: positional.len(),
        min: recipe.min_arguments(),
        max: recipe.max_arguments(),
      });
    }

    for (flag_name, flag_spec) in &recipe.flags {
      if !flags.contains_key(flag_name) {
        let default_value = match flag_spec.arity {
          FlagArity::Switch => Some("false".to_string()),
          FlagArity::WithValue => flag_spec.default.as_ref().map(ToString::to_string),
        };
        flags.insert(flag_name.clone(), default_value);
      }
    }

    self.next += consumed;

    Ok(ArgumentGroup {
      arguments: positional,
      flags,
      path,
    })
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
        arguments: Vec::new(),
        flags: BTreeMap::new(),
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
        arguments: vec!["baz"],
        flags: BTreeMap::new(),
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
        arguments: Vec::new(),
        flags: BTreeMap::new(),
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
          arguments: vec!["0"],
          flags: BTreeMap::new(),
        },
        ArgumentGroup {
          path: vec!["FOO".into()],
          arguments: vec!["1", "2"],
          flags: BTreeMap::new(),
        },
        ArgumentGroup {
          path: vec!["BAZ".into()],
          arguments: vec!["3", "4", "5"],
          flags: BTreeMap::new(),
        },
      ],
    );
  }

  fn make_test_recipe<'src>(
    name: Name<'src>,
    parameters: Vec<Parameter<'src>>,
    flags: BTreeMap<String, FlagSpec<'src>>,
  ) -> Recipe<'src> {
    Recipe {
      attributes: AttributeSet::default(),
      body: Vec::new(),
      dependencies: Vec::new(),
      doc: None,
      file_depth: 0,
      flags,
      import_offsets: Vec::new(),
      name,
      namepath: Some(name.lexeme().to_string()),
      parameters,
      priors: 0,
      private: false,
      quiet: false,
      shebang: false,
    }
  }

  fn make_justfile_with_recipe(recipe: Recipe<'_>) -> Justfile<'_> {
    let mut justfile = Justfile {
      aliases: Table::new(),
      assignments: Table::new(),
      default: None,
      doc: None,
      groups: Vec::new(),
      loaded: Vec::new(),
      module_path: String::new(),
      modules: Table::new(),
      name: None,
      recipes: Table::new(),
      settings: Settings::default(),
      source: PathBuf::new(),
      unexports: HashSet::new(),
      unstable_features: BTreeSet::new(),
      warnings: Vec::new(),
      working_directory: PathBuf::new(),
    };
    justfile.recipes.insert(Arc::new(recipe));
    justfile
  }

  #[test]
  fn with_switch_flag() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    let mut expected_flags = BTreeMap::new();
    expected_flags.insert("verbose".to_string(), Some("true".to_string()));

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--verbose"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        arguments: Vec::new(),
        flags: expected_flags,
      }],
    );
  }

  #[test]
  fn with_switch_flag_defaults_to_false() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    let mut expected_flags = BTreeMap::new();
    expected_flags.insert("verbose".to_string(), Some("false".to_string()));

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        arguments: Vec::new(),
        flags: expected_flags,
      }],
    );
  }

  #[test]
  fn with_value_flag() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "count".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::WithValue,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    let mut expected_flags = BTreeMap::new();
    expected_flags.insert("count".to_string(), Some("42".to_string()));

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--count", "42"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        arguments: Vec::new(),
        flags: expected_flags,
      }],
    );
  }

  #[test]
  fn with_value_flag_equals() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "count".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::WithValue,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    let mut expected_flags = BTreeMap::new();
    expected_flags.insert("count".to_string(), Some("42".to_string()));

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--count=42"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        arguments: Vec::new(),
        flags: expected_flags,
      }],
    );
  }

  #[test]
  fn with_multiple_flags() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );
    flags.insert(
      "count".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::WithValue,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    let mut expected_flags = BTreeMap::new();
    expected_flags.insert("count".to_string(), Some("10".to_string()));
    expected_flags.insert("verbose".to_string(), Some("true".to_string()));

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--verbose", "--count", "10"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        arguments: Vec::new(),
        flags: expected_flags,
      }],
    );
  }

  #[test]
  fn flags_with_positional_args() {
    let base = testing::compile("foo arg:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    let mut expected_flags = BTreeMap::new();
    expected_flags.insert("verbose".to_string(), Some("true".to_string()));

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--verbose", "value"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        arguments: vec!["value"],
        flags: expected_flags,
      }],
    );
  }

  #[test]
  fn flags_after_positional_args() {
    let base = testing::compile("foo arg:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    let mut expected_flags = BTreeMap::new();
    expected_flags.insert("verbose".to_string(), Some("true".to_string()));

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "value", "--verbose"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        arguments: vec!["value"],
        flags: expected_flags,
      }],
    );
  }

  #[test]
  fn double_dash_sentinel() {
    let base = testing::compile("foo arg:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    let mut expected_flags = BTreeMap::new();
    expected_flags.insert("verbose".to_string(), Some("false".to_string()));

    assert_eq!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--", "--verbose"]).unwrap(),
      vec![ArgumentGroup {
        path: vec!["foo".into()],
        arguments: vec!["--verbose"],
        flags: expected_flags,
      }],
    );
  }

  #[test]
  fn unknown_flag_error() {
    let justfile = testing::compile("foo:");

    assert_matches!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--unknown"]).unwrap_err(),
      Error::UnknownFlag {
        recipe,
        flag,
        suggestion: None
      } if recipe == "foo" && flag == "unknown",
    );
  }

  #[test]
  fn unknown_flag_with_suggestion() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    assert_matches!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--verbos"]).unwrap_err(),
      Error::UnknownFlag {
        recipe,
        flag,
        suggestion: Some(s)
      } if recipe == "foo" && flag == "verbos" && s == "verbose",
    );
  }

  #[test]
  fn duplicate_flag_error() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    assert_matches!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--verbose", "--verbose"]).unwrap_err(),
      Error::DuplicateFlag {
        recipe,
        flag
      } if recipe == "foo" && flag == "verbose",
    );
  }

  #[test]
  fn missing_flag_value_error() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "count".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::WithValue,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    assert_matches!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--count"]).unwrap_err(),
      Error::MissingFlagValue {
        recipe,
        flag
      } if recipe == "foo" && flag == "count",
    );
  }

  #[test]
  fn missing_flag_value_when_next_is_flag() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "count".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::WithValue,
        default: None,
      },
    );
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    assert_matches!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--count", "--verbose"]).unwrap_err(),
      Error::MissingFlagValue {
        recipe,
        flag
      } if recipe == "foo" && flag == "count",
    );
  }

  #[test]
  fn unexpected_flag_value_error() {
    let base = testing::compile("foo:");
    let orig = base.recipes.get("foo").unwrap();

    let mut flags = BTreeMap::new();
    flags.insert(
      "verbose".to_string(),
      FlagSpec {
        name: orig.name,
        arity: FlagArity::Switch,
        default: None,
      },
    );

    let recipe = make_test_recipe(orig.name, orig.parameters.clone(), flags);
    let justfile = make_justfile_with_recipe(recipe);

    assert_matches!(
      ArgumentParser::parse_arguments(&justfile, &["foo", "--verbose=true"]).unwrap_err(),
      Error::UnexpectedFlagValue {
        recipe,
        flag
      } if recipe == "foo" && flag == "verbose",
    );
  }
}
