use prelude::*;

use runtime_error::RuntimeError;
use edit_distance::edit_distance;

use recipe::Recipe;
use RunOptions;
use Expression;
use assignment_evaluator::evaluate_assignments;

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
    arguments: &[&'a str],
    options:   &RunOptions<'a>,
  ) -> Result<(), RuntimeError<'a>> {
    let unknown_overrides = options.overrides.keys().cloned()
      .filter(|name| !self.assignments.contains_key(name))
      .collect::<Vec<_>>();

    if !unknown_overrides.is_empty() {
      return Err(RuntimeError::UnknownOverrides{overrides: unknown_overrides});
    }

    let scope = evaluate_assignments(&self.assignments, &options.overrides, options.quiet)?;
    if options.evaluate {
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
          if !contains(&argument_range, argument_count) {
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
      return Err(RuntimeError::UnknownRecipes{recipes: missing, suggestion: suggestion});
    }

    let mut ran = empty();
    for (recipe, arguments) in grouped {
      self.run_recipe(recipe, arguments, &scope, &mut ran, options)?
    }

    Ok(())
  }

  fn run_recipe<'c>(
    &'c self,
    recipe:    &Recipe<'a>,
    arguments: &[&'a str],
    scope:     &Map<&'c str, String>,
    ran:       &mut Set<&'a str>,
    options:   &RunOptions<'a>,
  ) -> Result<(), RuntimeError> {
    for dependency_name in &recipe.dependencies {
      if !ran.contains(dependency_name) {
        self.run_recipe(&self.recipes[dependency_name], &[], scope, ran, options)?;
      }
    }
    recipe.run(arguments, scope, &self.exports, options)?;
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
