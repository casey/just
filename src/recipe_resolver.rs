use {super::*, CompileErrorKind::*};

pub(crate) struct RecipeResolver<'src: 'run, 'run> {
  assignments: &'run Table<'src, Assignment<'src>>,
  module_path: &'run str,
  modules: &'run Table<'src, Justfile<'src>>,
  resolved_recipes: Table<'src, Arc<Recipe<'src>>>,
  unresolved_recipes: Table<'src, UnresolvedRecipe<'src>>,
}

impl<'src: 'run, 'run> RecipeResolver<'src, 'run> {
  pub(crate) fn resolve_recipes(
    assignments: &'run Table<'src, Assignment<'src>>,
    module_path: &'run str,
    modules: &'run Table<'src, Justfile<'src>>,
    settings: &Settings,
    unresolved_recipes: Table<'src, UnresolvedRecipe<'src>>,
  ) -> CompileResult<'src, Table<'src, Arc<Recipe<'src>>>> {
    let mut resolver = Self {
      assignments,
      module_path,
      modules,
      resolved_recipes: Table::new(),
      unresolved_recipes,
    };

    while let Some(unresolved) = resolver.unresolved_recipes.pop() {
      resolver.resolve_recipe(&mut Vec::new(), unresolved)?;
    }

    for recipe in resolver.resolved_recipes.values() {
      for (i, parameter) in recipe.parameters.iter().enumerate() {
        if let Some(expression) = &parameter.default {
          for variable in expression.variables() {
            resolver.resolve_variable(&variable, &recipe.parameters[..i])?;
          }
        }
      }

      for dependency in &recipe.dependencies {
        for group in &dependency.arguments {
          for argument in group {
            for variable in argument.variables() {
              resolver.resolve_variable(&variable, &recipe.parameters)?;
            }
          }
        }
      }

      for line in &recipe.body {
        if line.is_comment() && settings.ignore_comments {
          continue;
        }

        for fragment in &line.fragments {
          if let Fragment::Interpolation { expression, .. } = fragment {
            for variable in expression.variables() {
              resolver.resolve_variable(&variable, &recipe.parameters)?;
            }
          }
        }
      }
    }

    Ok(resolver.resolved_recipes)
  }

  fn resolve_variable(
    &self,
    variable: &Token<'src>,
    parameters: &[Parameter],
  ) -> CompileResult<'src> {
    let name = variable.lexeme();

    let defined = self.assignments.contains_key(name)
      || parameters.iter().any(|p| p.name.lexeme() == name)
      || constants().contains_key(name);

    if !defined {
      return Err(variable.error(UndefinedVariable { variable: name }));
    }

    Ok(())
  }

  fn resolve_recipe(
    &mut self,
    stack: &mut Vec<&'src str>,
    recipe: UnresolvedRecipe<'src>,
  ) -> CompileResult<'src, Arc<Recipe<'src>>> {
    if let Some(resolved) = self.resolved_recipes.get(recipe.name()) {
      return Ok(Arc::clone(resolved));
    }

    stack.push(recipe.name());

    let dependencies = recipe
      .dependencies
      .iter()
      .map(|dependency| {
        self
          .resolve_dependency(dependency, &recipe, stack)?
          .ok_or_else(|| {
            dependency.recipe.last().error(UnknownDependency {
              recipe: recipe.name(),
              unknown: dependency.recipe.clone(),
            })
          })
      })
      .collect::<CompileResult<Vec<Arc<Recipe>>>>()?;

    stack.pop();

    let resolved = Arc::new(recipe.resolve(self.module_path, dependencies)?);
    self.resolved_recipes.insert(Arc::clone(&resolved));
    Ok(resolved)
  }

  fn resolve_dependency(
    &mut self,
    dependency: &UnresolvedDependency<'src>,
    recipe: &UnresolvedRecipe<'src>,
    stack: &mut Vec<&'src str>,
  ) -> CompileResult<'src, Option<Arc<Recipe<'src>>>> {
    let name = dependency.recipe.last().lexeme();

    if dependency.recipe.components() > 1 {
      // recipe is in a submodule and is thus already resolved
      Ok(Analyzer::resolve_recipe(
        &dependency.recipe,
        self.modules,
        &self.resolved_recipes,
      ))
    } else if let Some(resolved) = self.resolved_recipes.get(name) {
      // recipe is the current module and has already been resolved
      Ok(Some(Arc::clone(resolved)))
    } else if stack.contains(&name) {
      // recipe depends on itself
      let first = stack[0];
      stack.push(first);
      Err(
        dependency.recipe.last().error(CircularRecipeDependency {
          recipe: recipe.name(),
          circle: stack
            .iter()
            .skip_while(|name| **name != dependency.recipe.last().lexeme())
            .copied()
            .collect(),
        }),
      )
    } else if let Some(unresolved) = self.unresolved_recipes.remove(name) {
      // recipe is as of yet unresolved
      Ok(Some(self.resolve_recipe(stack, unresolved)?))
    } else {
      // recipe is unknown
      Ok(None)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  analysis_error! {
    name:   circular_recipe_dependency,
    input:  "a: b\nb: a",
    offset: 8,
    line:   1,
    column: 3,
    width:  1,
    kind:   CircularRecipeDependency{recipe: "b", circle: vec!["a", "b", "a"]},
  }

  analysis_error! {
    name:   self_recipe_dependency,
    input:  "a: a",
    offset: 3,
    line:   0,
    column: 3,
    width:  1,
    kind:   CircularRecipeDependency{recipe: "a", circle: vec!["a", "a"]},
  }

  analysis_error! {
    name:   unknown_dependency,
    input:  "a: b",
    offset: 3,
    line:   0,
    column: 3,
    width:  1,
    kind:   UnknownDependency{
      recipe: "a",
      unknown: Namepath::from(Name::from_identifier(
        Token{
          column: 3,
          kind: TokenKind::Identifier,
          length: 1,
          line: 0,
          offset: 3,
          path: Path::new("justfile"),
          src: "a: b" }))
    },
  }

  analysis_error! {
    name:   unknown_interpolation_variable,
    input:  "x:\n {{   hello}}",
    offset: 9,
    line:   1,
    column: 6,
    width:  5,
    kind:   UndefinedVariable{variable: "hello"},
  }

  analysis_error! {
    name:   unknown_second_interpolation_variable,
    input:  "wtf:=\"x\"\nx:\n echo\n foo {{wtf}} {{ lol }}",
    offset: 34,
    line:   3,
    column: 16,
    width:  3,
    kind:   UndefinedVariable{variable: "lol"},
  }

  analysis_error! {
    name:   unknown_variable_in_default,
    input:  "a f=foo:",
    offset: 4,
    line:   0,
    column: 4,
    width:  3,
    kind:   UndefinedVariable{variable: "foo"},
  }

  analysis_error! {
    name:   unknown_variable_in_dependency_argument,
    input:  "bar x:\nfoo: (bar baz)",
    offset: 17,
    line:   1,
    column: 10,
    width:  3,
    kind:   UndefinedVariable{variable: "baz"},
  }
}
