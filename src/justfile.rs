use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Justfile<'src> {
  pub(crate) recipes:     Table<'src, Rc<Recipe<'src>>>,
  pub(crate) assignments: Table<'src, Assignment<'src>>,
  pub(crate) aliases:     Table<'src, Alias<'src>>,
  pub(crate) settings:    Settings<'src>,
  pub(crate) warnings:    Vec<Warning>,
}

impl<'src> Justfile<'src> {
  pub(crate) fn first(&self) -> Option<&Recipe> {
    let mut first: Option<&Recipe<Dependency>> = None;
    for recipe in self.recipes.values() {
      if let Some(first_recipe) = first {
        if recipe.line_number() < first_recipe.line_number() {
          first = Some(recipe)
        }
      } else {
        first = Some(recipe);
      }
    }
    first
  }

  pub(crate) fn count(&self) -> usize {
    self.recipes.len()
  }

  pub(crate) fn suggest(&self, input: &str) -> Option<Suggestion> {
    let mut suggestions = self
      .recipes
      .keys()
      .map(|name| {
        (edit_distance(name, input), Suggestion {
          name,
          target: None,
        })
      })
      .chain(self.aliases.iter().map(|(name, alias)| {
        (edit_distance(name, input), Suggestion {
          name,
          target: Some(alias.target.name.lexeme()),
        })
      }))
      .filter(|(distance, _suggestion)| distance < &3)
      .collect::<Vec<(usize, Suggestion)>>();
    suggestions.sort_by_key(|(distance, _suggestion)| *distance);

    suggestions
      .into_iter()
      .map(|(_distance, suggestion)| suggestion)
      .next()
  }

  pub(crate) fn run<'run>(
    &'run self,
    config: &'run Config,
    search: &'run Search,
    overrides: &'run BTreeMap<String, String>,
    arguments: &'run [String],
  ) -> RunResult<'run, ()> {
    let unknown_overrides = overrides
      .keys()
      .filter(|name| !self.assignments.contains_key(name.as_str()))
      .map(String::as_str)
      .collect::<Vec<&str>>();

    if !unknown_overrides.is_empty() {
      return Err(RuntimeError::UnknownOverrides {
        overrides: unknown_overrides,
      });
    }

    let dotenv = if config.load_dotenv {
      load_dotenv(&config, &self.settings, &search.working_directory)?
    } else {
      BTreeMap::new()
    };

    let scope = {
      let mut scope = Scope::new();
      let mut unknown_overrides = Vec::new();

      for (name, value) in overrides {
        if let Some(assignment) = self.assignments.get(name) {
          scope.bind(assignment.export, assignment.name, value.clone());
        } else {
          unknown_overrides.push(name.as_ref());
        }
      }

      if !unknown_overrides.is_empty() {
        return Err(RuntimeError::UnknownOverrides {
          overrides: unknown_overrides,
        });
      }

      Evaluator::evaluate_assignments(
        &self.assignments,
        config,
        &dotenv,
        scope,
        &self.settings,
        search,
      )?
    };

    if let Subcommand::Evaluate { variables, .. } = &config.subcommand {
      let mut width = 0;

      for name in scope.names() {
        if !variables.is_empty() && !variables.iter().any(|variable| variable == name) {
          continue;
        }

        width = cmp::max(name.len(), width);
      }

      for binding in scope.bindings() {
        if !variables.is_empty()
          && !variables
            .iter()
            .any(|variable| variable == binding.name.lexeme())
        {
          continue;
        }

        println!(
          "{0:1$} := \"{2}\"",
          binding.name.lexeme(),
          width,
          binding.value
        );
      }

      return Ok(());
    }

    let argvec: Vec<&str> = if !arguments.is_empty() {
      arguments.iter().map(String::as_str).collect()
    } else if let Some(recipe) = self.first() {
      let min_arguments = recipe.min_arguments();
      if min_arguments > 0 {
        return Err(RuntimeError::DefaultRecipeRequiresArguments {
          recipe: recipe.name.lexeme(),
          min_arguments,
        });
      }
      vec![recipe.name()]
    } else {
      return Err(RuntimeError::NoRecipes);
    };

    let arguments = argvec.as_slice();

    let mut missing = vec![];
    let mut grouped = vec![];
    let mut rest = arguments;

    while let Some((argument, mut tail)) = rest.split_first() {
      if let Some(recipe) = self.get_recipe(argument) {
        if recipe.parameters.is_empty() {
          grouped.push((recipe, &[][..]));
        } else {
          let argument_range = recipe.argument_range();
          let argument_count = cmp::min(tail.len(), recipe.max_arguments());
          if !argument_range.range_contains(&argument_count) {
            return Err(RuntimeError::ArgumentCountMismatch {
              recipe:     recipe.name(),
              parameters: recipe.parameters.iter().collect(),
              found:      tail.len(),
              min:        recipe.min_arguments(),
              max:        recipe.max_arguments(),
            });
          }
          grouped.push((recipe, &tail[0..argument_count]));
          tail = &tail[argument_count..];
        }
      } else {
        missing.push(*argument);
      }
      rest = tail;
    }

    if !missing.is_empty() {
      let suggestion = if missing.len() == 1 {
        self.suggest(missing.first().unwrap())
      } else {
        None
      };
      return Err(RuntimeError::UnknownRecipes {
        recipes: missing,
        suggestion,
      });
    }

    let context = RecipeContext {
      settings: &self.settings,
      config,
      scope,
      search,
    };

    let mut ran = BTreeSet::new();
    for (recipe, arguments) in grouped {
      self.run_recipe(&context, recipe, arguments, &dotenv, &search, &mut ran)?
    }

    Ok(())
  }

  pub(crate) fn get_alias(&self, name: &str) -> Option<&Alias> {
    self.aliases.get(name)
  }

  pub(crate) fn get_recipe(&self, name: &str) -> Option<&Recipe<'src>> {
    if let Some(recipe) = self.recipes.get(name) {
      Some(recipe)
    } else if let Some(alias) = self.aliases.get(name) {
      Some(alias.target.as_ref())
    } else {
      None
    }
  }

  fn run_recipe<'run>(
    &self,
    context: &'run RecipeContext<'src, 'run>,
    recipe: &Recipe<'src>,
    arguments: &[&'run str],
    dotenv: &BTreeMap<String, String>,
    search: &'run Search,
    ran: &mut BTreeSet<Vec<String>>,
  ) -> RunResult<'src, ()> {
    let outer = Evaluator::evaluate_parameters(
      context.config,
      dotenv,
      &recipe.parameters,
      arguments,
      &context.scope,
      context.settings,
      search,
    )?;

    let scope = outer.child();

    let mut evaluator =
      Evaluator::recipe_evaluator(context.config, dotenv, &scope, context.settings, search);

    for Dependency { recipe, arguments } in &recipe.dependencies {
      let mut invocation = vec![recipe.name().to_owned()];

      for argument in arguments {
        invocation.push(evaluator.evaluate_expression(argument)?);
      }

      if !ran.contains(&invocation) {
        let arguments = invocation
          .iter()
          .skip(1)
          .map(String::as_ref)
          .collect::<Vec<&str>>();
        self.run_recipe(context, recipe, &arguments, dotenv, search, ran)?;
      }
    }

    recipe.run(context, dotenv, scope.child(), search, arguments)?;

    let mut invocation = vec![recipe.name().to_owned()];
    for argument in arguments.iter().cloned() {
      invocation.push(argument.to_owned());
    }

    ran.insert(invocation);
    Ok(())
  }

  pub(crate) fn public_recipes(&self, source_order: bool) -> Vec<&Recipe<Dependency>> {
    let mut recipes = self
      .recipes
      .values()
      .map(AsRef::as_ref)
      .filter(|recipe| recipe.public())
      .collect::<Vec<&Recipe<Dependency>>>();

    if source_order {
      recipes.sort_by_key(|recipe| recipe.name.offset);
    }

    recipes
  }
}

impl<'src> Display for Justfile<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    let mut items = self.recipes.len() + self.assignments.len() + self.aliases.len();
    for (name, assignment) in &self.assignments {
      if assignment.export {
        write!(f, "export ")?;
      }
      write!(f, "{} := {}", name, assignment.value)?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    for alias in self.aliases.values() {
      write!(f, "{}", alias)?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    for recipe in self.recipes.values() {
      write!(f, "{}", recipe)?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use testing::compile;
  use RuntimeError::*;

  run_error! {
    name: unknown_recipes,
    src: "a:\nb:\nc:",
    args: ["a", "x", "y", "z"],
    error: UnknownRecipes {
      recipes,
      suggestion,
    },
    check: {
      assert_eq!(recipes, &["x", "y", "z"]);
      assert_eq!(suggestion, None);
    }
  }

  run_error! {
    name: unknown_recipes_show_alias_suggestion,
    src: "
      foo:
        echo foo

      alias z := foo
    ",
    args: ["zz"],
    error: UnknownRecipes {
      recipes,
      suggestion,
    },
    check: {
      assert_eq!(recipes, &["zz"]);
      assert_eq!(suggestion, Some(Suggestion {
        name: "z",
        target: Some("foo"),
      }
    ));
    }
  }

  // This test exists to make sure that shebang recipes run correctly.  Although
  // this script is still executed by a shell its behavior depends on the value of
  // a variable and continuing even though a command fails, whereas in plain
  // recipes variables are not available in subsequent lines and execution stops
  // when a line fails.
  run_error! {
    name: run_shebang,
    src: "
      a:
        #!/usr/bin/env sh
        code=200
          x() { return $code; }
            x
              x
    ",
    args: ["a"],
    error: Code {
      recipe,
      line_number,
      code,
    },
    check: {
      assert_eq!(recipe, "a");
      assert_eq!(code, 200);
      assert_eq!(line_number, None);
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
    },
    check: {
      assert_eq!(recipe, "fail");
      assert_eq!(code, 100);
      assert_eq!(line_number, Some(2));
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
    },
    check: {
      assert_eq!(recipe, "a");
      assert_eq!(code, 150);
      assert_eq!(line_number, Some(2));
    }
  }

  run_error! {
    name: missing_some_arguments,
    src: "a b c d:",
    args: ["a", "b", "c"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
      assert_eq!(found, 2);
      assert_eq!(min, 3);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: missing_some_arguments_variadic,
    src: "a b c +d:",
    args: ["a", "B", "C"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
      assert_eq!(found, 2);
      assert_eq!(min, 3);
      assert_eq!(max, usize::MAX - 1);
    }
  }

  run_error! {
    name: missing_all_arguments,
    src: "a b c d:\n echo {{b}}{{c}}{{d}}",
    args: ["a"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
      assert_eq!(found, 0);
      assert_eq!(min, 3);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: missing_some_defaults,
    src: "a b c d='hello':",
    args: ["a", "b"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
      assert_eq!(found, 1);
      assert_eq!(min, 2);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: missing_all_defaults,
    src: "a b c='r' d='h':",
    args: ["a"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
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
      code: _,
      line_number,
      recipe,
    },
    check: {
      assert_eq!(recipe, "wut");
      assert_eq!(line_number, Some(7));
    }
  }

  macro_rules! test {
    ($name:ident, $input:expr, $expected:expr $(,)*) => {
      #[test]
      fn $name() {
        test($input, $expected);
      }
    };
  }

  fn test(input: &str, expected: &str) {
    let justfile = compile(input);
    let actual = format!("{:#}", justfile);
    assert_eq!(actual, expected);
    println!("Re-parsing...");
    let reparsed = compile(&actual);
    let redumped = format!("{:#}", reparsed);
    assert_eq!(redumped, actual);
  }

  test! {
    parse_empty,
    "

# hello


    ",
    "",
  }

  test! {
    parse_string_default,
    r#"

foo a="b\t":


  "#,
    r#"foo a="b\t":"#,
  }

  test! {
  parse_multiple,
    r#"
a:
b:
"#,
    r#"a:

b:"#,
  }

  test! {
    parse_variadic,
    r#"

foo +a:


  "#,
    r#"foo +a:"#,
  }

  test! {
    parse_variadic_string_default,
    r#"

foo +a="Hello":


  "#,
    r#"foo +a="Hello":"#,
  }

  test! {
    parse_raw_string_default,
    r#"

foo a='b\t':


  "#,
    r#"foo a='b\t':"#,
  }

  test! {
    parse_export,
    r#"
export a := "hello"

  "#,
    r#"export a := "hello""#,
  }

  test! {
  parse_alias_after_target,
    r#"
foo:
  echo a
alias f := foo
"#,
r#"alias f := foo

foo:
    echo a"#
  }

  test! {
  parse_alias_before_target,
    r#"
alias f := foo
foo:
  echo a
"#,
r#"alias f := foo

foo:
    echo a"#
  }

  test! {
  parse_alias_with_comment,
    r#"
alias f := foo #comment
foo:
  echo a
"#,
r#"alias f := foo

foo:
    echo a"#
  }

  test! {
  parse_complex,
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
    {{foo + bar}}abc{{goodbye + \"x\"}}xyz
    1
    2
    3

x:

y:

z:"
  }

  test! {
  parse_shebang,
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
    if [[ -f {{practicum}} ]]; then
    \treturn
    fi",
  }

  test! {
    parse_simple_shebang,
    "a:\n #!\n  print(1)",
    "a:\n    #!\n     print(1)",
  }

  test! {
  parse_assignments,
    r#"a := "0"
c := a + b + a + b
b := "1"
"#,
    r#"a := "0"

b := "1"

c := a + b + a + b"#,
  }

  test! {
  parse_assignment_backticks,
    "a := `echo hello`
c := a + b + a + b
b := `echo goodbye`",
    "a := `echo hello`

b := `echo goodbye`

c := a + b + a + b",
  }

  test! {
  parse_interpolation_backticks,
    r#"a:
  echo {{  `echo hello` + "blarg"   }} {{   `echo bob`   }}"#,
    r#"a:
    echo {{`echo hello` + "blarg"}} {{`echo bob`}}"#,
  }

  test! {
    eof_test,
    "x:\ny:\nz:\na b c: x y z",
    "a b c: x y z\n\nx:\n\ny:\n\nz:",
  }

  test! {
    string_quote_escape,
    r#"a := "hello\"""#,
    r#"a := "hello\"""#,
  }

  test! {
    string_escapes,
    r#"a := "\n\t\r\"\\""#,
    r#"a := "\n\t\r\"\\""#,
  }

  test! {
  parameters,
    "a b c:
  {{b}} {{c}}",
    "a b c:
    {{b}} {{c}}",
  }

  test! {
  unary_functions,
    "
x := arch()

a:
  {{os()}} {{os_family()}}",
    "x := arch()

a:
    {{os()}} {{os_family()}}",
  }

  test! {
  env_functions,
    r#"
x := env_var('foo',)

a:
  {{env_var_or_default('foo' + 'bar', 'baz',)}} {{env_var(env_var("baz"))}}"#,
    r#"x := env_var('foo')

a:
    {{env_var_or_default('foo' + 'bar', 'baz')}} {{env_var(env_var("baz"))}}"#,
  }

  test! {
    parameter_default_string,
    r#"
f x="abc":
"#,
    r#"f x="abc":"#,
  }

  test! {
    parameter_default_raw_string,
    r#"
f x='abc':
"#,
    r#"f x='abc':"#,
  }

  test! {
    parameter_default_backtick,
    r#"
f x=`echo hello`:
"#,
    r#"f x=`echo hello`:"#,
  }

  test! {
    parameter_default_concatination_string,
    r#"
f x=(`echo hello` + "foo"):
"#,
    r#"f x=(`echo hello` + "foo"):"#,
  }

  test! {
    parameter_default_concatination_variable,
    r#"
x := "10"
f y=(`echo hello` + x) +z="foo":
"#,
    r#"x := "10"

f y=(`echo hello` + x) +z="foo":"#,
  }

  test! {
    parameter_default_multiple,
    r#"
x := "10"
f y=(`echo hello` + x) +z=("foo" + "bar"):
"#,
    r#"x := "10"

f y=(`echo hello` + x) +z=("foo" + "bar"):"#,
  }

  test! {
    concatination_in_group,
    "x := ('0' + '1')",
    "x := ('0' + '1')",
  }

  test! {
    string_in_group,
    "x := ('0'   )",
    "x := ('0')",
  }

  #[rustfmt::skip]
  test! {
    escaped_dos_newlines,
    "@spam:\r
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
    } | less",
  }
}
