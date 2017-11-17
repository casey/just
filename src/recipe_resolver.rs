use common::*;
use recipe::Recipe;
use compilation_error::{CompilationError, CompilationErrorKind};

use Fragment;
use Expression;

pub fn resolve_recipes<'a>(
  recipes:     &Map<&'a str, Recipe<'a>>,
  assignments: &Map<&'a str, Expression<'a>>,
  text:        &'a str,
) -> Result<(), CompilationError<'a>> {
  let mut resolver = Resolver {
    seen:              empty(),
    stack:             empty(),
    resolved:          empty(),
    recipes:           recipes,
  };

  for recipe in recipes.values() {
    resolver.resolve(recipe)?;
    resolver.seen = empty();
  }

  for recipe in recipes.values() {
    for line in &recipe.lines {
      for fragment in line {
        if let Fragment::Expression{ref expression, ..} = *fragment {
          for variable in expression.variables() {
            let name = variable.lexeme;
            let undefined = !assignments.contains_key(name)
              && !recipe.parameters.iter().any(|p| p.name == name);
            if undefined {
              // There's a borrow issue here that seems too difficult to solve.
              // The error derived from the variable token has too short a lifetime,
              // so we create a new error from its contents, which do live long
              // enough.
              //
              // I suspect the solution here is to give recipes, pieces, and expressions
              // two lifetime parameters instead of one, with one being the lifetime
              // of the struct, and the second being the lifetime of the tokens
              // that it contains
              let error = variable.error(CompilationErrorKind::UndefinedVariable{variable: name});
              return Err(CompilationError {
                text:   text,
                index:  error.index,
                line:   error.line,
                column: error.column,
                width:  error.width,
                kind:   CompilationErrorKind::UndefinedVariable {
                  variable: &text[error.index..error.index + error.width.unwrap()],
                }
              });
            }
          }
        }
      }
    }
  }

  Ok(())
}

struct Resolver<'a: 'b, 'b> {
  stack:    Vec<&'a str>,
  seen:     Set<&'a str>,
  resolved: Set<&'a str>,
  recipes:  &'b Map<&'a str, Recipe<'a>>,
}

impl<'a, 'b> Resolver<'a, 'b> {
  fn resolve(&mut self, recipe: &Recipe<'a>) -> Result<(), CompilationError<'a>> {
    if self.resolved.contains(recipe.name) {
      return Ok(())
    }
    self.stack.push(recipe.name);
    self.seen.insert(recipe.name);
    for dependency_token in &recipe.dependency_tokens {
      match self.recipes.get(dependency_token.lexeme) {
        Some(dependency) => if !self.resolved.contains(dependency.name) {
          if self.seen.contains(dependency.name) {
            let first = self.stack[0];
            self.stack.push(first);
            return Err(dependency_token.error(CompilationErrorKind::CircularRecipeDependency {
              recipe: recipe.name,
              circle: self.stack.iter()
                .skip_while(|name| **name != dependency.name)
                .cloned().collect()
            }));
          }
          self.resolve(dependency)?;
        },
        None => return Err(dependency_token.error(CompilationErrorKind::UnknownDependency {
          recipe:  recipe.name,
          unknown: dependency_token.lexeme
        })),
      }
    }
    self.resolved.insert(recipe.name);
    self.stack.pop();
    Ok(())
  }
}

