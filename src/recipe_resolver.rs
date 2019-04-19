use crate::common::*;

use CompilationErrorKind::*;

// There are borrow issues here that seems too difficult to solve.
// The errors derived from the variable token has too short a lifetime,
// so we create a new error from its contents, which do live long
// enough.
//
// I suspect the solution here is to give recipes, pieces, and expressions
// two lifetime parameters instead of one, with one being the lifetime
// of the struct, and the second being the lifetime of the tokens
// that it contains.

pub struct RecipeResolver<'a: 'b, 'b> {
  stack: Vec<&'a str>,
  seen: BTreeSet<&'a str>,
  resolved: BTreeSet<&'a str>,
  recipes: &'b BTreeMap<&'a str, Recipe<'a>>,
  assignments: &'b BTreeMap<&'a str, Expression<'a>>,
  text: &'a str,
}

impl<'a, 'b> RecipeResolver<'a, 'b> {
  pub fn resolve_recipes(
    recipes: &BTreeMap<&'a str, Recipe<'a>>,
    assignments: &BTreeMap<&'a str, Expression<'a>>,
    text: &'a str,
  ) -> CompilationResult<'a, ()> {
    let mut resolver = RecipeResolver {
      seen: empty(),
      stack: empty(),
      resolved: empty(),
      assignments,
      text,
      recipes,
    };

    for recipe in recipes.values() {
      resolver.resolve_recipe(recipe)?;
      resolver.seen = empty();
    }

    for recipe in recipes.values() {
      for parameter in &recipe.parameters {
        if let Some(expression) = &parameter.default {
          for (function, argc) in expression.functions() {
            resolver.resolve_function(function, argc)?;
          }
          for variable in expression.variables() {
            resolver.resolve_variable(variable, &[])?;
          }
        }
      }

      for line in &recipe.lines {
        for fragment in line {
          if let Fragment::Expression { ref expression, .. } = *fragment {
            for (function, argc) in expression.functions() {
              resolver.resolve_function(function, argc)?;
            }
            for variable in expression.variables() {
              resolver.resolve_variable(variable, &recipe.parameters)?;
            }
          }
        }
      }
    }

    Ok(())
  }

  fn resolve_function(&self, function: &Token, argc: usize) -> CompilationResult<'a, ()> {
    resolve_function(function, argc).map_err(|error| CompilationError {
      offset: error.offset,
      line: error.line,
      column: error.column,
      width: error.width,
      kind: UnknownFunction {
        function: &self.text[error.offset..error.offset + error.width],
      },
      text: self.text,
    })
  }

  fn resolve_variable(
    &self,
    variable: &Token,
    parameters: &[Parameter],
  ) -> CompilationResult<'a, ()> {
    let name = variable.lexeme();
    let undefined =
      !self.assignments.contains_key(name) && !parameters.iter().any(|p| p.name == name);
    if undefined {
      let error = variable.error(UndefinedVariable { variable: name });
      return Err(CompilationError {
        offset: error.offset,
        line: error.line,
        column: error.column,
        width: error.width,
        kind: UndefinedVariable {
          variable: &self.text[error.offset..error.offset + error.width],
        },
        text: self.text,
      });
    }

    Ok(())
  }

  fn resolve_recipe(&mut self, recipe: &Recipe<'a>) -> CompilationResult<'a, ()> {
    if self.resolved.contains(recipe.name) {
      return Ok(());
    }
    self.stack.push(recipe.name);
    self.seen.insert(recipe.name);
    for dependency_token in &recipe.dependency_tokens {
      match self.recipes.get(dependency_token.lexeme()) {
        Some(dependency) => {
          if !self.resolved.contains(dependency.name) {
            if self.seen.contains(dependency.name) {
              let first = self.stack[0];
              self.stack.push(first);
              return Err(
                dependency_token.error(CircularRecipeDependency {
                  recipe: recipe.name,
                  circle: self
                    .stack
                    .iter()
                    .skip_while(|name| **name != dependency.name)
                    .cloned()
                    .collect(),
                }),
              );
            }
            self.resolve_recipe(dependency)?;
          }
        }
        None => {
          return Err(dependency_token.error(UnknownDependency {
            recipe: recipe.name,
            unknown: dependency_token.lexeme(),
          }));
        }
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

  error_test! {
    name:   circular_recipe_dependency,
    input:  "a: b\nb: a",
    offset: 8,
    line:   1,
    column: 3,
    width:  1,
    kind:   CircularRecipeDependency{recipe: "b", circle: vec!["a", "b", "a"]},
  }

  error_test! {
    name:   self_recipe_dependency,
    input:  "a: a",
    offset: 3,
    line:   0,
    column: 3,
    width:  1,
    kind:   CircularRecipeDependency{recipe: "a", circle: vec!["a", "a"]},
  }

  error_test! {
    name:   unknown_dependency,
    input:  "a: b",
    offset: 3,
    line:   0,
    column: 3,
    width:  1,
    kind:   UnknownDependency{recipe: "a", unknown: "b"},
  }

  error_test! {
    name:   unknown_interpolation_variable,
    input:  "x:\n {{   hello}}",
    offset: 9,
    line:   1,
    column: 6,
    width:  5,
    kind:   UndefinedVariable{variable: "hello"},
  }

  error_test! {
    name:   unknown_second_interpolation_variable,
    input:  "wtf=\"x\"\nx:\n echo\n foo {{wtf}} {{ lol }}",
    offset: 33,
    line:   3,
    column: 16,
    width:  3,
    kind:   UndefinedVariable{variable: "lol"},
  }

  error_test! {
    name:   unknown_function_in_interpolation,
    input:  "a:\n echo {{bar()}}",
    offset: 11,
    line:   1,
    column: 8,
    width:  3,
    kind:   UnknownFunction{function: "bar"},
  }

  error_test! {
    name:   unknown_function_in_default,
    input:  "a f=baz():",
    offset: 4,
    line:   0,
    column: 4,
    width:  3,
    kind:   UnknownFunction{function: "baz"},
  }

  error_test! {
    name:   unknown_variable_in_default,
    input:  "a f=foo:",
    offset: 4,
    line:   0,
    column: 4,
    width:  3,
    kind:   UndefinedVariable{variable: "foo"},
  }
}
