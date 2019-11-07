use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Justfile<'a> {
  pub(crate) recipes: Table<'a, Recipe<'a>>,
  pub(crate) assignments: Table<'a, Assignment<'a>>,
  pub(crate) aliases: Table<'a, Alias<'a>>,
  pub(crate) warnings: Vec<Warning<'a>>,
}

impl<'a> Justfile<'a> {
  pub(crate) fn first(&self) -> Option<&Recipe> {
    let mut first: Option<&Recipe> = None;
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

  pub(crate) fn suggest(&self, name: &str) -> Option<&'a str> {
    let mut suggestions = self
      .recipes
      .keys()
      .map(|suggestion| (edit_distance(suggestion, name), suggestion))
      .collect::<Vec<_>>();
    suggestions.sort();
    if let Some(&(distance, suggestion)) = suggestions.first() {
      if distance < 3 {
        return Some(suggestion);
      }
    }
    None
  }

  pub(crate) fn run(
    &'a self,
    config: &'a Config,
    working_directory: &'a Path,
  ) -> RunResult<'a, ()> {
    let argvec: Vec<&str> = if !config.arguments.is_empty() {
      config
        .arguments
        .iter()
        .map(|argument| argument.as_str())
        .collect()
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

    let unknown_overrides = config
      .overrides
      .keys()
      .filter(|name| !self.assignments.contains_key(name.as_str()))
      .map(|name| name.as_str())
      .collect::<Vec<&str>>();

    if !unknown_overrides.is_empty() {
      return Err(RuntimeError::UnknownOverrides {
        overrides: unknown_overrides,
      });
    }

    let dotenv = load_dotenv()?;

    let scope = AssignmentEvaluator::evaluate_assignments(
      config,
      working_directory,
      &dotenv,
      &self.assignments,
    )?;

    if config.subcommand == Subcommand::Evaluate {
      let mut width = 0;
      for name in scope.keys() {
        width = cmp::max(name.len(), width);
      }

      for (name, (_export, value)) in scope {
        println!("{0:1$} := \"{2}\"", name, width, value);
      }
      return Ok(());
    }

    let mut missing = vec![];
    let mut grouped = vec![];
    let mut rest = arguments;

    while let Some((argument, mut tail)) = rest.split_first() {
      if let Some(recipe) = self.get_recipe(argument) {
        if recipe.parameters.is_empty() {
          grouped.push((recipe, &tail[0..0]));
        } else {
          let argument_range = recipe.argument_range();
          let argument_count = cmp::min(tail.len(), recipe.max_arguments());
          if !argument_range.range_contains(&argument_count) {
            return Err(RuntimeError::ArgumentCountMismatch {
              recipe: recipe.name(),
              parameters: recipe.parameters.iter().collect(),
              found: tail.len(),
              min: recipe.min_arguments(),
              max: recipe.max_arguments(),
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
      config,
      scope,
      working_directory,
    };

    let mut ran = empty();
    for (recipe, arguments) in grouped {
      self.run_recipe(&context, recipe, arguments, &dotenv, &mut ran)?
    }

    Ok(())
  }

  pub(crate) fn get_alias(&self, name: &str) -> Option<&Alias> {
    self.aliases.get(name)
  }

  pub(crate) fn get_recipe(&self, name: &str) -> Option<&Recipe<'a>> {
    if let Some(recipe) = self.recipes.get(name) {
      Some(recipe)
    } else if let Some(alias) = self.aliases.get(name) {
      self.recipes.get(alias.target.lexeme())
    } else {
      None
    }
  }

  fn run_recipe<'b>(
    &self,
    context: &'b RecipeContext<'a>,
    recipe: &Recipe<'a>,
    arguments: &[&'a str],
    dotenv: &BTreeMap<String, String>,
    ran: &mut BTreeSet<&'a str>,
  ) -> RunResult<()> {
    for dependency_name in &recipe.dependencies {
      let lexeme = dependency_name.lexeme();
      if !ran.contains(lexeme) {
        self.run_recipe(context, &self.recipes[lexeme], &[], dotenv, ran)?;
      }
    }
    recipe.run(context, arguments, dotenv)?;
    ran.insert(recipe.name());
    Ok(())
  }
}

impl<'a> Display for Justfile<'a> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    let mut items = self.recipes.len() + self.assignments.len() + self.aliases.len();
    for (name, assignment) in &self.assignments {
      if assignment.export {
        write!(f, "export ")?;
      }
      write!(f, "{} := {}", name, assignment.expression)?;
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

  use crate::runtime_error::RuntimeError::*;
  use crate::testing::{compile, config};

  #[test]
  fn unknown_recipes() {
    let justfile = compile("a:\nb:\nc:");
    let config = config(&["a", "x", "y", "z"]);
    let dir = env::current_dir().unwrap();

    match justfile.run(&config, &dir).unwrap_err() {
      UnknownRecipes {
        recipes,
        suggestion,
      } => {
        assert_eq!(recipes, &["x", "y", "z"]);
        assert_eq!(suggestion, None);
      }
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn run_shebang() {
    // this test exists to make sure that shebang recipes
    // run correctly. although this script is still
    // executed by a shell its behavior depends on the value of a
    // variable and continuing even though a command fails,
    // whereas in plain recipes variables are not available
    // in subsequent lines and execution stops when a line
    // fails
    let text = "
a:
 #!/usr/bin/env sh
 code=200
  x() { return $code; }
    x
      x
";
    let justfile = compile(text);
    let config = config(&["a"]);
    let dir = env::current_dir().unwrap();
    match justfile.run(&config, &dir).unwrap_err() {
      Code {
        recipe,
        line_number,
        code,
      } => {
        assert_eq!(recipe, "a");
        assert_eq!(code, 200);
        assert_eq!(line_number, None);
      }
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn code_error() {
    let justfile = compile("fail:\n @exit 100");
    let config = config(&["fail"]);
    let dir = env::current_dir().unwrap();
    match justfile.run(&config, &dir).unwrap_err() {
      Code {
        recipe,
        line_number,
        code,
      } => {
        assert_eq!(recipe, "fail");
        assert_eq!(code, 100);
        assert_eq!(line_number, Some(2));
      }
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn run_args() {
    let text = r#"
a return code:
 @x() { {{return}} {{code + "0"}}; }; x"#;
    let justfile = compile(text);
    let config = config(&["a", "return", "15"]);
    let dir = env::current_dir().unwrap();

    match justfile.run(&config, &dir).unwrap_err() {
      Code {
        recipe,
        line_number,
        code,
      } => {
        assert_eq!(recipe, "a");
        assert_eq!(code, 150);
        assert_eq!(line_number, Some(3));
      }
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn missing_some_arguments() {
    let justfile = compile("a b c d:");
    let config = config(&["a", "b", "c"]);
    let dir = env::current_dir().unwrap();
    match justfile.run(&config, &dir).unwrap_err() {
      ArgumentCountMismatch {
        recipe,
        parameters,
        found,
        min,
        max,
      } => {
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
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn missing_some_arguments_variadic() {
    let justfile = compile("a b c +d:");
    let config = config(&["a", "B", "C"]);
    let dir = env::current_dir().unwrap();
    match justfile.run(&config, &dir).unwrap_err() {
      ArgumentCountMismatch {
        recipe,
        parameters,
        found,
        min,
        max,
      } => {
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
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn missing_all_arguments() {
    let justfile = compile("a b c d:\n echo {{b}}{{c}}{{d}}");
    let config = config(&["a"]);
    let dir = env::current_dir().unwrap();

    match justfile.run(&config, &dir).unwrap_err() {
      ArgumentCountMismatch {
        recipe,
        parameters,
        found,
        min,
        max,
      } => {
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
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn missing_some_defaults() {
    let justfile = compile("a b c d='hello':");
    let config = config(&["a", "b"]);
    let dir = env::current_dir().unwrap();

    match justfile.run(&config, &dir).unwrap_err() {
      ArgumentCountMismatch {
        recipe,
        parameters,
        found,
        min,
        max,
      } => {
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
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn missing_all_defaults() {
    let justfile = compile("a b c='r' d='h':");
    let config = &config(&["a"]);
    let dir = env::current_dir().unwrap();

    match justfile.run(&config, &dir).unwrap_err() {
      ArgumentCountMismatch {
        recipe,
        parameters,
        found,
        min,
        max,
      } => {
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
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn unknown_overrides() {
    let config = config(&["foo=bar", "baz=bob", "a"]);
    let justfile = compile("a:\n echo {{`f() { return 100; }; f`}}");
    let dir = env::current_dir().unwrap();

    match justfile.run(&config, &dir).unwrap_err() {
      UnknownOverrides { overrides } => {
        assert_eq!(overrides, &["baz", "foo"]);
      }
      other => panic!("unexpected error: {}", other),
    }
  }

  #[test]
  fn export_failure() {
    let text = r#"
export foo = "a"
baz = "c"
export bar = "b"
export abc = foo + bar + baz

wut:
  echo $foo $bar $baz
"#;

    let config = config(&["--quiet", "wut"]);

    let justfile = compile(text);
    let dir = env::current_dir().unwrap();

    match justfile.run(&config, &dir).unwrap_err() {
      Code {
        code: _,
        line_number,
        recipe,
      } => {
        assert_eq!(recipe, "wut");
        assert_eq!(line_number, Some(8));
      }
      other => panic!("unexpected error: {}", other),
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
