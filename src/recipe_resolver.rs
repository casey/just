use common::*;

pub fn resolve_recipes<'a>(
  recipes:     &Map<&'a str, Recipe<'a>>,
  assignments: &Map<&'a str, Expression<'a>>,
  text:        &'a str,
) -> Result<(), CompilationError<'a>> {
  let mut resolver = RecipeResolver {
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

struct RecipeResolver<'a: 'b, 'b> {
  stack:    Vec<&'a str>,
  seen:     Set<&'a str>,
  resolved: Set<&'a str>,
  recipes:  &'b Map<&'a str, Recipe<'a>>,
}

impl<'a, 'b> RecipeResolver<'a, 'b> {
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

#[cfg(test)]
mod test {
  use super::*;
  use testing::parse_error;

#[test]
fn circular_recipe_dependency() {
  let text = "a: b\nb: a";
  parse_error(text, CompilationError {
    text:   text,
    index:  8,
    line:   1,
    column: 3,
    width:  Some(1),
    kind:   CompilationErrorKind::CircularRecipeDependency{recipe: "b", circle: vec!["a", "b", "a"]}
  });
}

#[test]
fn self_recipe_dependency() {
  let text = "a: a";
  parse_error(text, CompilationError {
    text:   text,
    index:  3,
    line:   0,
    column: 3,
    width:  Some(1),
    kind:   CompilationErrorKind::CircularRecipeDependency{recipe: "a", circle: vec!["a", "a"]}
  });
}


#[test]
fn unknown_dependency() {
  let text = "a: b";
  parse_error(text, CompilationError {
    text:   text,
    index:  3,
    line:   0,
    column: 3,
    width:  Some(1),
    kind:   CompilationErrorKind::UnknownDependency{recipe: "a", unknown: "b"}
  });
}

#[test]
fn unknown_interpolation_variable() {
  let text = "x:\n {{   hello}}";
  parse_error(text, CompilationError {
    text:   text,
    index:  9,
    line:   1,
    column: 6,
    width:  Some(5),
    kind:   CompilationErrorKind::UndefinedVariable{variable: "hello"},
  });
}

#[test]
fn unknown_second_interpolation_variable() {
  let text = "wtf=\"x\"\nx:\n echo\n foo {{wtf}} {{ lol }}";
  parse_error(text, CompilationError {
    text:   text,
    index:  33,
    line:   3,
    column: 16,
    width:  Some(3),
    kind:   CompilationErrorKind::UndefinedVariable{variable: "lol"},
  });
}

}
