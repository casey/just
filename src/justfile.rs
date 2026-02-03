use {super::*, serde::Serialize};

#[derive(Debug, PartialEq, Serialize)]
pub(crate) struct Justfile<'src> {
  pub(crate) aliases: Table<'src, Alias<'src>>,
  pub(crate) assignments: Table<'src, Assignment<'src>>,
  #[serde(rename = "first", serialize_with = "keyed::serialize_option")]
  pub(crate) default: Option<Arc<Recipe<'src>>>,
  pub(crate) doc: Option<String>,
  pub(crate) groups: Vec<StringLiteral<'src>>,
  #[serde(skip)]
  pub(crate) loaded: Vec<PathBuf>,
  #[serde(skip)]
  pub(crate) module_path: String,
  pub(crate) modules: Table<'src, Self>,
  #[serde(skip)]
  pub(crate) name: Option<Name<'src>>,
  #[serde(skip)]
  pub(crate) private: bool,
  pub(crate) recipes: Table<'src, Arc<Recipe<'src>>>,
  pub(crate) settings: Settings,
  pub(crate) source: PathBuf,
  pub(crate) unexports: HashSet<String>,
  #[serde(skip)]
  pub(crate) unstable_features: BTreeSet<UnstableFeature>,
  pub(crate) warnings: Vec<Warning>,
  #[serde(skip)]
  pub(crate) working_directory: PathBuf,
}

impl<'src> Justfile<'src> {
  fn find_suggestion(
    input: &str,
    candidates: impl Iterator<Item = Suggestion<'src>>,
  ) -> Option<Suggestion<'src>> {
    candidates
      .map(|suggestion| (edit_distance(input, suggestion.name), suggestion))
      .filter(|(distance, _suggestion)| *distance < 3)
      .min_by_key(|(distance, _suggestion)| *distance)
      .map(|(_distance, suggestion)| suggestion)
  }

  pub(crate) fn suggest_recipe(&self, input: &str) -> Option<Suggestion<'src>> {
    Self::find_suggestion(
      input,
      self
        .recipes
        .values()
        .filter(|recipe| recipe.is_public())
        .map(|recipe| Suggestion {
          name: recipe.name(),
          target: None,
        })
        .chain(
          self
            .aliases
            .values()
            .filter(|alias| alias.is_public())
            .map(|alias| Suggestion {
              name: alias.name.lexeme(),
              target: Some(alias.target.name.lexeme()),
            }),
        ),
    )
  }

  pub(crate) fn suggest_variable(&self, input: &str) -> Option<Suggestion<'src>> {
    Self::find_suggestion(
      input,
      self
        .assignments
        .keys()
        .map(|name| Suggestion { name, target: None }),
    )
  }

  fn evaluate_scopes<'run>(
    &'run self,
    arena: &'run Arena<Scope<'src, 'run>>,
    config: &'run Config,
    dotenv: &'run BTreeMap<String, String>,
    root: &'run Scope<'src, 'run>,
    scopes: &mut BTreeMap<String, (&'run Self, &'run Scope<'src, 'run>)>,
    search: &'run Search,
  ) -> RunResult<'src> {
    let scope = Evaluator::evaluate_assignments(config, dotenv, self, root, search)?;

    let scope = arena.alloc(scope);
    scopes.insert(self.module_path.clone(), (self, scope));

    for module in self.modules.values() {
      module.evaluate_scopes(arena, config, dotenv, scope, scopes, search)?;
    }

    Ok(())
  }

  pub(crate) fn run(
    &self,
    config: &Config,
    search: &Search,
    arguments: &[String],
  ) -> RunResult<'src> {
    let unknown_overrides = config
      .overrides
      .keys()
      .filter(|name| !self.assignments.contains_key(name.as_str()))
      .cloned()
      .collect::<Vec<String>>();

    if !unknown_overrides.is_empty() {
      return Err(Error::UnknownOverrides {
        overrides: unknown_overrides,
      });
    }

    let dotenv = if config.load_dotenv {
      load_dotenv(config, &self.settings, &search.working_directory)?
    } else {
      BTreeMap::new()
    };

    let root = Scope::root();
    let arena = Arena::new();
    let mut scopes = BTreeMap::new();
    self.evaluate_scopes(&arena, config, &dotenv, &root, &mut scopes, search)?;

    let scope = scopes.get(&self.module_path).unwrap().1;

    match &config.subcommand {
      Subcommand::Command {
        binary, arguments, ..
      } => {
        let mut command = if config.shell_command {
          let mut command = self.settings.shell_command(config);
          command.arg(binary);
          command
        } else {
          Command::new(binary)
        };

        command
          .args(arguments)
          .current_dir(&search.working_directory);

        let scope = scope.child();

        command.export(&self.settings, &dotenv, &scope, &self.unexports);

        let (result, caught) = command.status_guard();

        let status = result.map_err(|io_error| Error::CommandInvoke {
          binary: binary.clone(),
          arguments: arguments.clone(),
          io_error,
        })?;

        if !status.success() {
          return Err(Error::CommandStatus {
            binary: binary.clone(),
            arguments: arguments.clone(),
            status,
          });
        }

        if let Some(signal) = caught {
          return Err(Error::Interrupted { signal });
        }

        return Ok(());
      }
      Subcommand::Evaluate { variable, .. } => {
        if let Some(variable) = variable {
          if let Some(value) = scope.value(variable) {
            print!("{value}");
          } else {
            return Err(Error::EvalUnknownVariable {
              suggestion: self.suggest_variable(variable),
              variable: variable.clone(),
            });
          }
        } else {
          let width = scope.names().fold(0, |max, name| name.len().max(max));

          for binding in scope.bindings() {
            if !binding.private {
              println!(
                "{0:1$} := \"{2}\"",
                binding.name.lexeme(),
                width,
                binding.value
              );
            }
          }
        }

        return Ok(());
      }
      _ => {}
    }

    let arguments = arguments.iter().map(String::as_str).collect::<Vec<&str>>();

    let invocations = InvocationParser::parse_invocations(self, &arguments)?;

    if config.one && invocations.len() > 1 {
      return Err(Error::ExcessInvocations {
        invocations: invocations.len(),
      });
    }

    let ran = Ran::default();
    for invocation in invocations {
      Self::run_recipe(
        &invocation.arguments,
        config,
        &dotenv,
        false,
        &ran,
        invocation.recipe,
        &scopes,
        search,
      )?;
    }

    Ok(())
  }

  pub(crate) fn check_unstable(&self, config: &Config) -> RunResult<'src> {
    if let Some(&unstable_feature) = self.unstable_features.iter().next() {
      config.require_unstable(self, unstable_feature)?;
    }

    for module in self.modules.values() {
      module.check_unstable(config)?;
    }

    Ok(())
  }

  pub(crate) fn get_alias(&self, name: &str) -> Option<&Alias<'src>> {
    self.aliases.get(name)
  }

  pub(crate) fn get_recipe(&self, name: &str) -> Option<&Recipe<'src>> {
    self
      .recipes
      .get(name)
      .map(Arc::as_ref)
      .or_else(|| self.aliases.get(name).map(|alias| alias.target.as_ref()))
  }

  pub(crate) fn is_submodule(&self) -> bool {
    self.name.is_some()
  }

  pub(crate) fn name(&self) -> &'src str {
    self.name.map(|name| name.lexeme()).unwrap_or_default()
  }

  fn run_recipe(
    arguments: &[Vec<String>],
    config: &Config,
    dotenv: &BTreeMap<String, String>,
    is_dependency: bool,
    ran: &Ran,
    recipe: &Recipe<'src>,
    scopes: &BTreeMap<String, (&Self, &Scope<'src, '_>)>,
    search: &Search,
  ) -> RunResult<'src> {
    {
      let mutex = ran.mutex(recipe, arguments);

      let mut guard = mutex.lock().unwrap();

      if *guard {
        return Ok(());
      }

      *guard = true;
    }

    if !config.yes && !recipe.confirm()? {
      return Err(Error::NotConfirmed {
        recipe: recipe.name(),
      });
    }

    let (module, scope) = scopes
      .get(recipe.module_path())
      .expect("failed to retrieve scope for module");

    let context = ExecutionContext {
      config,
      dotenv,
      module,
      search,
    };

    let (outer, positional) = Evaluator::evaluate_parameters(
      arguments,
      &context,
      is_dependency,
      &recipe.parameters,
      recipe,
      scope,
    )?;

    let scope = outer.child();

    let mut evaluator = Evaluator::new(&context, true, &scope);

    Self::run_dependencies(
      config,
      &context,
      recipe.priors(),
      dotenv,
      &mut evaluator,
      ran,
      recipe,
      scopes,
      search,
    )?;

    recipe.run(&context, &scope, &positional, is_dependency)?;

    Self::run_dependencies(
      config,
      &context,
      recipe.subsequents(),
      dotenv,
      &mut evaluator,
      &Ran::default(),
      recipe,
      scopes,
      search,
    )?;

    Ok(())
  }

  fn run_dependencies<'run>(
    config: &Config,
    context: &ExecutionContext<'src, 'run>,
    dependencies: &[Dependency<'src>],
    dotenv: &BTreeMap<String, String>,
    evaluator: &mut Evaluator<'src, 'run>,
    ran: &Ran,
    recipe: &Recipe<'src>,
    scopes: &BTreeMap<String, (&Self, &Scope<'src, 'run>)>,
    search: &Search,
  ) -> RunResult<'src> {
    if context.config.no_dependencies {
      return Ok(());
    }

    let mut evaluated = Vec::new();
    for Dependency { recipe, arguments } in dependencies {
      let mut grouped = Vec::new();
      for group in arguments {
        let evaluated_group = group
          .iter()
          .map(|argument| evaluator.evaluate_expression(argument))
          .collect::<RunResult<Vec<String>>>()?;
        grouped.push(evaluated_group);
      }
      evaluated.push((recipe, grouped));
    }

    if recipe.is_parallel() {
      thread::scope::<_, RunResult>(|thread_scope| {
        let mut handles = Vec::new();
        for (recipe, arguments) in evaluated {
          handles.push(thread_scope.spawn(move || {
            Self::run_recipe(
              &arguments, config, dotenv, true, ran, recipe, scopes, search,
            )
          }));
        }
        for handle in handles {
          handle
            .join()
            .map_err(|_| Error::internal("parallel dependency thread panicked"))??;
        }
        Ok(())
      })?;
    } else {
      for (recipe, arguments) in evaluated {
        Self::run_recipe(
          &arguments, config, dotenv, true, ran, recipe, scopes, search,
        )?;
      }
    }

    Ok(())
  }

  pub(crate) fn public_modules(&self, config: &Config) -> Vec<&Justfile> {
    let mut modules = self
      .modules
      .values()
      .filter(|module| !module.private)
      .collect::<Vec<&Justfile>>();

    if config.unsorted {
      modules.sort_by_key(|module| {
        module
          .name
          .map(|name| name.token.offset)
          .unwrap_or_default()
      });
    }

    modules
  }

  pub(crate) fn public_recipes(&self, config: &Config) -> Vec<&Recipe> {
    let mut recipes = self
      .recipes
      .values()
      .map(AsRef::as_ref)
      .filter(|recipe| recipe.is_public())
      .collect::<Vec<&Recipe>>();

    if config.unsorted {
      recipes.sort_by_key(|recipe| (&recipe.import_offsets, recipe.name.offset));
    }

    recipes
  }

  pub(crate) fn groups(&self) -> Vec<&str> {
    self
      .groups
      .iter()
      .map(|group| group.cooked.as_str())
      .collect()
  }

  pub(crate) fn public_groups(&self, config: &Config) -> Vec<String> {
    let mut groups = Vec::new();

    for recipe in self.recipes.values() {
      if recipe.is_public() {
        for group in recipe.groups() {
          groups.push((recipe.import_offsets.as_slice(), recipe.name.offset, group));
        }
      }
    }

    for submodule in self.public_modules(config) {
      for group in submodule.groups() {
        groups.push((&[], submodule.name.unwrap().offset, group.to_string()));
      }
    }

    if config.unsorted {
      groups.sort();
    } else {
      groups.sort_by(|(_, _, a), (_, _, b)| a.cmp(b));
    }

    let mut seen = HashSet::new();

    groups.retain(|(_, _, group)| seen.insert(group.clone()));

    groups.into_iter().map(|(_, _, group)| group).collect()
  }
}

impl ColorDisplay for Justfile<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    let mut items = self.recipes.len() + self.assignments.len() + self.aliases.len();
    for (name, assignment) in &self.assignments {
      if assignment.export {
        write!(f, "export ")?;
      }
      write!(f, "{name} := {}", assignment.value)?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    for alias in self.aliases.values() {
      write!(f, "{alias}")?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    for recipe in self.recipes.values() {
      write!(f, "{}", recipe.color_display(color))?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    Ok(())
  }
}

impl<'src> Keyed<'src> for Justfile<'src> {
  fn key(&self) -> &'src str {
    self.name()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use testing::compile;
  use Error::*;

  run_error! {
    name: unknown_recipe_no_suggestion,
    src: "a:\nb:\nc:",
    args: ["a", "xyz", "y", "z"],
    error: UnknownRecipe {
      recipe,
      suggestion,
    },
    check: {
      assert_eq!(recipe, "xyz");
      assert_eq!(suggestion, None);
    }
  }

  run_error! {
    name: unknown_recipe_with_suggestion,
    src: "a:\nb:\nc:",
    args: ["a", "x", "y", "z"],
    error: UnknownRecipe {
      recipe,
      suggestion,
    },
    check: {
      assert_eq!(recipe, "x");
      assert_eq!(suggestion, Some(Suggestion {
        name: "a",
        target: None,
      }));
    }
  }

  run_error! {
    name: unknown_recipe_show_alias_suggestion,
    src: "
      foo:
        echo foo

      alias z := foo
    ",
    args: ["zz"],
    error: UnknownRecipe {
      recipe,
      suggestion,
    },
    check: {
      assert_eq!(recipe, "zz");
      assert_eq!(suggestion, Some(Suggestion {
        name: "z",
        target: Some("foo"),
      }
    ));
    }
  }

  run_error! {
    name: code_error,
    src: "
      fail:
        @exit 100
    ",
    args: ["fail"],
    error: Code {
      recipe,
      line_number,
      code,
      print_message,
    },
    check: {
      assert_eq!(recipe, "fail");
      assert_eq!(code, 100);
      assert_eq!(line_number, Some(2));
      assert!(print_message);
    }
  }

  run_error! {
    name: run_args,
    src: r#"
      a return code:
        @x() { {{return}} {{code + "0"}}; }; x
    "#,
    args: ["a", "return", "15"],
    error: Code {
      recipe,
      line_number,
      code,
      print_message,
    },
    check: {
      assert_eq!(recipe, "a");
      assert_eq!(code, 150);
      assert_eq!(line_number, Some(2));
      assert!(print_message);
    }
  }

  run_error! {
    name: missing_some_arguments,
    src: "a b c d:",
    args: ["a", "b", "c"],
    error: PositionalArgumentCountMismatch {
      recipe,
      found,
      min,
      max,
    },
    check: {
      assert_eq!(recipe.name(), "a");
      assert_eq!(found, 2);
      assert_eq!(min, 3);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: missing_some_arguments_variadic,
    src: "a b c +d:",
    args: ["a", "B", "C"],
    error: PositionalArgumentCountMismatch {
      recipe,
      found,
      min,
      max,
    },
    check: {
      assert_eq!(recipe.name(), "a");
      assert_eq!(found, 2);
      assert_eq!(min, 3);
      assert_eq!(max, usize::MAX - 1);
    }
  }

  run_error! {
    name: missing_all_arguments,
    src: "a b c d:\n echo {{b}}{{c}}{{d}}",
    args: ["a"],
    error: PositionalArgumentCountMismatch {
      recipe,
      found,
      min,
      max,
    },
    check: {
      assert_eq!(recipe.name(), "a");
      assert_eq!(found, 0);
      assert_eq!(min, 3);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: missing_some_defaults,
    src: "a b c d='hello':",
    args: ["a", "b"],
    error: PositionalArgumentCountMismatch {
      recipe,
      found,
      min,
      max,
    },
    check: {
      assert_eq!(recipe.name(), "a");
      assert_eq!(found, 1);
      assert_eq!(min, 2);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: missing_all_defaults,
    src: "a b c='r' d='h':",
    args: ["a"],
    error: PositionalArgumentCountMismatch {
      recipe,
      found,
      min,
      max,
    },
    check: {
      assert_eq!(recipe.name(), "a");
      assert_eq!(found, 0);
      assert_eq!(min, 1);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: unknown_overrides,
    src: "
      a:
       echo {{`f() { return 100; }; f`}}
    ",
    args: ["foo=bar", "baz=bob", "a"],
    error: UnknownOverrides { overrides },
    check: {
      assert_eq!(overrides, &["baz", "foo"]);
    }
  }

  run_error! {
    name: export_failure,
    src: r#"
      export foo := "a"
      baz := "c"
      export bar := "b"
      export abc := foo + bar + baz

      wut:
        echo $foo $bar $baz
    "#,
    args: ["--quiet", "wut"],
    error: Code {
      recipe,
      line_number,
      print_message,
      ..
    },
    check: {
      assert_eq!(recipe, "wut");
      assert_eq!(line_number, Some(7));
      assert!(print_message);
    }
  }

  fn case(input: &str, expected: &str) {
    let justfile = compile(input);
    let actual = format!("{}", justfile.color_display(Color::never()));
    assert_eq!(actual, expected);
    let reparsed = compile(&actual);
    let redumped = format!("{}", reparsed.color_display(Color::never()));
    assert_eq!(redumped, actual);
  }

  #[test]
  fn parse_empty() {
    case(
      "

# hello


    ",
      "",
    );
  }

  #[test]
  fn parse_string_default() {
    case(
      r#"

foo a="b\t":


  "#,
      r#"foo a="b\t":"#,
    );
  }

  #[test]
  fn parse_multiple() {
    case(
      r"
a:
b:
", r"a:

b:",
    );
  }

  #[test]
  fn parse_variadic() {
    case(
      r"

foo +a:


  ",
      r"foo +a:",
    );
  }

  #[test]
  fn parse_variadic_string_default() {
    case(
      r#"

foo +a="Hello":


  "#,
      r#"foo +a="Hello":"#,
    );
  }

  #[test]
  fn parse_raw_string_default() {
    case(
      r"

foo a='b\t':


  ",
      r"foo a='b\t':",
    );
  }

  #[test]
  fn parse_export() {
    case(
      r#"
export a := "hello"

  "#,
      r#"export a := "hello""#,
    );
  }

  #[test]
  fn parse_alias_after_target() {
    case(
      r"
foo:
  echo a
alias f := foo
",
      r"alias f := foo

foo:
    echo a",
    );
  }

  #[test]
  fn parse_alias_before_target() {
    case(
      r"
alias f := foo
foo:
  echo a
",
      r"alias f := foo

foo:
    echo a",
    );
  }

  #[test]
  fn parse_alias_with_comment() {
    case(
      r"
alias f := foo #comment
foo:
  echo a
",
      r"alias f := foo

foo:
    echo a",
    );
  }

  #[test]
  fn parse_complex() {
    case(
      "
x:
y:
z:
foo := \"xx\"
bar := foo
goodbye := \"y\"
hello a b    c   : x y    z #hello
  #! blah
  #blarg
  {{ foo + bar}}abc{{ goodbye\t  + \"x\" }}xyz
  1
  2
  3
",
      "bar := foo

foo := \"xx\"

goodbye := \"y\"

hello a b c: x y z
    #! blah
    #blarg
    {{ foo + bar }}abc{{ goodbye + \"x\" }}xyz
    1
    2
    3

x:

y:

z:",
    );
  }

  #[test]
  fn parse_shebang() {
    case(
      "
practicum := 'hello'
install:
\t#!/bin/sh
\tif [[ -f {{practicum}} ]]; then
\t\treturn
\tfi
",
      "practicum := 'hello'

install:
    #!/bin/sh
    if [[ -f {{ practicum }} ]]; then
    \treturn
    fi",
    );
  }

  #[test]
  fn parse_simple_shebang() {
    case("a:\n #!\n  print(1)", "a:\n    #!\n     print(1)");
  }

  #[test]
  fn parse_assignments() {
    case(
      r#"a := "0"
c := a + b + a + b
b := "1"
"#,
      r#"a := "0"

b := "1"

c := a + b + a + b"#,
    );
  }

  #[test]
  fn parse_assignment_backticks() {
    case(
      "a := `echo hello`
c := a + b + a + b
b := `echo goodbye`",
      "a := `echo hello`

b := `echo goodbye`

c := a + b + a + b",
    );
  }

  #[test]
  fn parse_interpolation_backticks() {
    case(
      r#"a:
  echo {{  `echo hello` + "blarg"   }} {{   `echo bob`   }}"#,
      r#"a:
    echo {{ `echo hello` + "blarg" }} {{ `echo bob` }}"#,
    );
  }

  #[test]
  fn eof_test() {
    case("x:\ny:\nz:\na b c: x y z", "a b c: x y z\n\nx:\n\ny:\n\nz:");
  }

  #[test]
  fn string_quote_escape() {
    case(r#"a := "hello\"""#, r#"a := "hello\"""#);
  }

  #[test]
  fn string_escapes() {
    case(r#"a := "\n\t\r\"\\""#, r#"a := "\n\t\r\"\\""#);
  }

  #[test]
  fn parameters() {
    case(
      "a b c:
  {{b}} {{c}}",
      "a b c:
    {{ b }} {{ c }}",
    );
  }

  #[test]
  fn unary_functions() {
    case(
      "
x := arch()

a:
  {{os()}} {{os_family()}} {{num_cpus()}}",
      "x := arch()

a:
    {{ os() }} {{ os_family() }} {{ num_cpus() }}",
    );
  }

  #[test]
  fn env_functions() {
    case(
      r#"
x := env_var('foo',)

a:
  {{env_var_or_default('foo' + 'bar', 'baz',)}} {{env_var(env_var("baz"))}}"#,
      r#"x := env_var('foo')

a:
    {{ env_var_or_default('foo' + 'bar', 'baz') }} {{ env_var(env_var("baz")) }}"#,
    );
  }

  #[test]
  fn parameter_default_string() {
    case(
      r#"
f x="abc":
"#,
      r#"f x="abc":"#,
    );
  }

  #[test]
  fn parameter_default_raw_string() {
    case(
      r"
f x='abc':
",
      r"f x='abc':",
    );
  }

  #[test]
  fn parameter_default_backtick() {
    case(
      r"
f x=`echo hello`:
",
      r"f x=`echo hello`:",
    );
  }

  #[test]
  fn parameter_default_concatenation_string() {
    case(
      r#"
f x=(`echo hello` + "foo"):
"#,
      r#"f x=(`echo hello` + "foo"):"#,
    );
  }

  #[test]
  fn parameter_default_concatenation_variable() {
    case(
      r#"
x := "10"
f y=(`echo hello` + x) +z="foo":
"#,
      r#"x := "10"

f y=(`echo hello` + x) +z="foo":"#,
    );
  }

  #[test]
  fn parameter_default_multiple() {
    case(
      r#"
x := "10"
f y=(`echo hello` + x) +z=("foo" + "bar"):
"#,
      r#"x := "10"

f y=(`echo hello` + x) +z=("foo" + "bar"):"#,
    );
  }

  #[test]
  fn concatenation_in_group() {
    case("x := ('0' + '1')", "x := ('0' + '1')");
  }

  #[test]
  fn string_in_group() {
    case("x := ('0'   )", "x := ('0')");
  }

  #[rustfmt::skip]
  #[test]
  fn escaped_dos_newlines() {
    case("@spam:\r
\t{ \\\r
\t\tfiglet test; \\\r
\t\tcargo build --color always 2>&1; \\\r
\t\tcargo test  --color always -- --color always 2>&1; \\\r
\t} | less\r
",
"@spam:
    { \\
    \tfiglet test; \\
    \tcargo build --color always 2>&1; \\
    \tcargo test  --color always -- --color always 2>&1; \\
    } | less");
  }
}
