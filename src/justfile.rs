use std::path::PathBuf;
use std::sync::atomic::Ordering;

use common::*;

use edit_distance::edit_distance;

use recipe::INTERRUPTED;

pub struct Justfile<'a> {
  pub recipes:     Map<&'a str, Recipe<'a>>,
  pub assignments: Map<&'a str, Expression<'a>>,
  pub exports:     Set<&'a str>,
}

impl<'a, 'b> Justfile<'a> where 'a: 'b {
  pub fn first(&self) -> Option<&Recipe> {
    let mut first: Option<&Recipe> = None;
    for recipe in self.recipes.values() {
      if let Some(first_recipe) = first {
        if recipe.line_number < first_recipe.line_number {
          first = Some(recipe)
        }
      } else {
        first = Some(recipe);
      }
    }
    first
  }

  pub fn count(&self) -> usize {
    self.recipes.len()
  }

  pub fn suggest(&self, name: &str) -> Option<&'a str> {
    let mut suggestions = self.recipes.keys()
      .map(|suggestion| (edit_distance(suggestion, name), suggestion))
      .collect::<Vec<_>>();
    suggestions.sort();
    if let Some(&(distance, suggestion)) = suggestions.first() {
      if distance < 3 {
        return Some(suggestion)
      }
    }
    None
  }

  pub fn run(
    &'a self,
    invocation_directory: Result<PathBuf, String>,
    arguments:     &[&'a str],
    configuration: &Configuration<'a>,
  ) -> RunResult<'a, ()> {
    let unknown_overrides = configuration.overrides.keys().cloned()
      .filter(|name| !self.assignments.contains_key(name))
      .collect::<Vec<_>>();

    if !unknown_overrides.is_empty() {
      return Err(RuntimeError::UnknownOverrides{overrides: unknown_overrides});
    }

    let dotenv = load_dotenv()?;

    let scope = AssignmentEvaluator::evaluate_assignments(
      &self.assignments,
      &invocation_directory,
      &dotenv,
      &configuration.overrides,
      configuration.quiet,
      configuration.shell,
      configuration.dry_run,
    )?;

    if configuration.evaluate {
      let mut width = 0;
      for name in scope.keys() {
        width = cmp::max(name.len(), width);
      }

      for (name, value) in scope {
        println!("{0:1$} = \"{2}\"", name, width, value);
      }
      return Ok(());
    }

    let mut missing = vec![];
    let mut grouped = vec![];
    let mut rest    = arguments;

    while let Some((argument, mut tail)) = rest.split_first() {
      if let Some(recipe) = self.recipes.get(argument) {
        if recipe.parameters.is_empty() {
          grouped.push((recipe, &tail[0..0]));
        } else {
          let argument_range = recipe.argument_range();
          let argument_count = cmp::min(tail.len(), recipe.max_arguments());
          if !argument_range.range_contains(argument_count) {
            return Err(RuntimeError::ArgumentCountMismatch {
              recipe: recipe.name,
              found:  tail.len(),
              min:    recipe.min_arguments(),
              max:    recipe.max_arguments(),
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
      return Err(RuntimeError::UnknownRecipes{recipes: missing, suggestion});
    }

    let mut ran = empty();
    for (recipe, arguments) in grouped {
      self.run_recipe(&invocation_directory, recipe, arguments, &scope, &dotenv, configuration, &mut ran)?
    }

    if INTERRUPTED.load(Ordering::SeqCst) {
      eprintln!("Interrupted.");
    }

    Ok(())
  }

  fn run_recipe<'c>(
    &'c self,
    invocation_directory: &Result<PathBuf, String>,
    recipe:        &Recipe<'a>,
    arguments:     &[&'a str],
    scope:         &Map<&'c str, String>,
    dotenv:        &Map<String, String>,
    configuration: &Configuration<'a>,
    ran:           &mut Set<&'a str>,
  ) -> RunResult<()> {
    if INTERRUPTED.load(Ordering::SeqCst) {
      return Ok(())
    }

    for dependency_name in &recipe.dependencies {
      if !ran.contains(dependency_name) {
        self.run_recipe(invocation_directory, &self.recipes[dependency_name], &[], scope, dotenv, configuration, ran)?;
      }
    }
    recipe.run(invocation_directory, arguments, scope, dotenv, &self.exports, configuration)?;
    ran.insert(recipe.name);
    Ok(())
  }
}

impl<'a> Display for Justfile<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let mut items = self.recipes.len() + self.assignments.len();
    for (name, expression) in &self.assignments {
      if self.exports.contains(name) {
        write!(f, "export ")?;
      }
      write!(f, "{} = {}", name, expression)?;
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
mod test {
  use super::*;
  use testing::parse_success;
  use RuntimeError::*;

  fn no_cwd_err() -> Result<PathBuf, String> {
    Err(String::from("no cwd in tests"))
  }

  #[test]
  fn unknown_recipes() {
    match parse_success("a:\nb:\nc:").run(no_cwd_err(), &["a", "x", "y", "z"], &Default::default()).unwrap_err() {
      UnknownRecipes{recipes, suggestion} => {
        assert_eq!(recipes, &["x", "y", "z"]);
        assert_eq!(suggestion, None);
      }
      other => panic!("expected an unknown recipe error, but got: {}", other),
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

    match parse_success(text).run(no_cwd_err(), &["a"], &Default::default()).unwrap_err() {
      Code{recipe, line_number, code} => {
        assert_eq!(recipe, "a");
        assert_eq!(code, 200);
        assert_eq!(line_number, None);
      },
      other => panic!("expected a code run error, but got: {}", other),
    }
  }

  #[test]
  fn code_error() {
    match parse_success("fail:\n @exit 100")
      .run(no_cwd_err(), &["fail"], &Default::default()).unwrap_err() {
      Code{recipe, line_number, code} => {
        assert_eq!(recipe, "fail");
        assert_eq!(code, 100);
        assert_eq!(line_number, Some(2));
      },
      other => panic!("expected a code run error, but got: {}", other),
    }
  }

  #[test]
  fn run_args() {
    let text = r#"
a return code:
 @x() { {{return}} {{code + "0"}}; }; x"#;

    match parse_success(text).run(no_cwd_err(), &["a", "return", "15"], &Default::default()).unwrap_err() {
      Code{recipe, line_number, code} => {
        assert_eq!(recipe, "a");
        assert_eq!(code, 150);
        assert_eq!(line_number, Some(3));
      },
      other => panic!("expected a code run error, but got: {}", other),
    }
  }

  #[test]
  fn missing_some_arguments() {
    match parse_success("a b c d:").run(no_cwd_err(), &["a", "b", "c"], &Default::default()).unwrap_err() {
      ArgumentCountMismatch{recipe, found, min, max} => {
        assert_eq!(recipe, "a");
        assert_eq!(found, 2);
        assert_eq!(min, 3);
        assert_eq!(max, 3);
      },
      other => panic!("expected a code run error, but got: {}", other),
    }
  }

  #[test]
  fn missing_some_arguments_variadic() {
    match parse_success("a b c +d:").run(no_cwd_err(), &["a", "B", "C"], &Default::default()).unwrap_err() {
      ArgumentCountMismatch{recipe, found, min, max} => {
        assert_eq!(recipe, "a");
        assert_eq!(found, 2);
        assert_eq!(min, 3);
        assert_eq!(max, usize::MAX - 1);
      },
      other => panic!("expected a code run error, but got: {}", other),
    }
  }

  #[test]
  fn missing_all_arguments() {
    match parse_success("a b c d:\n echo {{b}}{{c}}{{d}}")
          .run(no_cwd_err(), &["a"], &Default::default()).unwrap_err() {
      ArgumentCountMismatch{recipe, found, min, max} => {
        assert_eq!(recipe, "a");
        assert_eq!(found, 0);
        assert_eq!(min, 3);
        assert_eq!(max, 3);
      },
      other => panic!("expected a code run error, but got: {}", other),
    }
  }

  #[test]
  fn missing_some_defaults() {
    match parse_success("a b c d='hello':").run(no_cwd_err(), &["a", "b"], &Default::default()).unwrap_err() {
      ArgumentCountMismatch{recipe, found, min, max} => {
        assert_eq!(recipe, "a");
        assert_eq!(found, 1);
        assert_eq!(min, 2);
        assert_eq!(max, 3);
      },
      other => panic!("expected a code run error, but got: {}", other),
    }
  }

  #[test]
  fn missing_all_defaults() {
    match parse_success("a b c='r' d='h':").run(no_cwd_err(), &["a"], &Default::default()).unwrap_err() {
      ArgumentCountMismatch{recipe, found, min, max} => {
        assert_eq!(recipe, "a");
        assert_eq!(found, 0);
        assert_eq!(min, 1);
        assert_eq!(max, 3);
      },
      other => panic!("expected a code run error, but got: {}", other),
    }
  }

  #[test]
  fn unknown_overrides() {
    let mut configuration: Configuration = Default::default();
    configuration.overrides.insert("foo", "bar");
    configuration.overrides.insert("baz", "bob");
    match parse_success("a:\n echo {{`f() { return 100; }; f`}}")
          .run(no_cwd_err(), &["a"], &configuration).unwrap_err() {
      UnknownOverrides{overrides} => {
        assert_eq!(overrides, &["baz", "foo"]);
      },
      other => panic!("expected a code run error, but got: {}", other),
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

    let configuration = Configuration {
      quiet: true,
      ..Default::default()
    };

    match parse_success(text).run(no_cwd_err(), &["wut"], &configuration).unwrap_err() {
      Code{code: _, line_number, recipe} => {
        assert_eq!(recipe, "wut");
        assert_eq!(line_number, Some(8));
      },
      other => panic!("expected a recipe code errror, but got: {}", other),
    }
  }
}
