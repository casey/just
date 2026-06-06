use {super::*, CompileErrorKind::*};

enum Resolution<'src> {
  Disabled(BTreeSet<Modulepath>),
  Resolved(Arc<Recipe<'src>>),
}

enum DependencyResolution<'src> {
  Disabled(BTreeSet<Modulepath>),
  Resolved(Arc<Recipe<'src>>),
  Unknown,
}

pub(crate) struct RecipeResolver<'src: 'run, 'run> {
  absent: &'run BTreeSet<String>,
  assignments: &'run Table<'src, Assignment<'src>>,
  disabled: Table<'src, Disabled<'src>>,
  functions: &'run Table<'src, FunctionDefinition<'src>>,
  modulepath: &'run Modulepath,
  modules: &'run Table<'src, Justfile<'src>>,
  resolved_recipes: Table<'src, Arc<Recipe<'src>>>,
  settings: &'run Settings,
  unresolved_recipes: Table<'src, UnresolvedRecipe<'src>>,
}

impl<'src: 'run, 'run> RecipeResolver<'src, 'run> {
  pub(crate) fn resolve_recipes(
    absent: &'run BTreeSet<String>,
    assignments: &'run Table<'src, Assignment<'src>>,
    functions: &'run Table<'src, FunctionDefinition<'src>>,
    modulepath: &'run Modulepath,
    modules: &'run Table<'src, Justfile<'src>>,
    settings: &'run Settings,
    unresolved_recipes: Table<'src, UnresolvedRecipe<'src>>,
  ) -> CompileResult<'src, (Table<'src, Arc<Recipe<'src>>>, Table<'src, Disabled<'src>>)> {
    let mut resolver = Self {
      absent,
      assignments,
      disabled: Table::new(),
      functions,
      modulepath,
      modules,
      resolved_recipes: Table::new(),
      settings,
      unresolved_recipes,
    };

    while let Some(unresolved) = resolver.unresolved_recipes.pop() {
      resolver.resolve_recipe(&mut Vec::new(), unresolved)?;
    }

    Ok((resolver.resolved_recipes, resolver.disabled))
  }

  fn resolve_recipe(
    &mut self,
    stack: &mut Vec<&'src str>,
    recipe: UnresolvedRecipe<'src>,
  ) -> CompileResult<'src, Resolution<'src>> {
    if let Some(resolved) = self.resolved_recipes.get(recipe.name()) {
      return Ok(Resolution::Resolved(Arc::clone(resolved)));
    }

    if let Some(disabled) = self.disabled.get(recipe.name()) {
      return Ok(Resolution::Disabled(disabled.modules.clone()));
    }

    stack.push(recipe.name());

    let mut dependencies = Vec::new();
    let mut disabled_by = BTreeSet::new();

    for dependency in &recipe.dependencies {
      match self.resolve_dependency(dependency, &recipe, stack)? {
        DependencyResolution::Resolved(resolved) => dependencies.push(resolved),
        DependencyResolution::Disabled(modules) => disabled_by.extend(modules),
        DependencyResolution::Unknown => {
          return Err(dependency.recipe.last().error(UnknownDependency {
            recipe: recipe.name(),
            unknown: dependency.recipe.clone(),
          }));
        }
      }
    }

    stack.pop();

    if disabled_by.is_empty() {
      let resolved = Arc::new(recipe.resolve(
        self.assignments,
        self.functions,
        self.modulepath,
        dependencies,
        self.settings,
      )?);
      self.resolved_recipes.insert(Arc::clone(&resolved));
      Ok(Resolution::Resolved(resolved))
    } else {
      self.disabled.insert(Disabled {
        name: recipe.name,
        modules: disabled_by.clone(),
      });
      Ok(Resolution::Disabled(disabled_by))
    }
  }

  fn resolve_dependency(
    &mut self,
    dependency: &UnresolvedDependency<'src>,
    recipe: &UnresolvedRecipe<'src>,
    stack: &mut Vec<&'src str>,
  ) -> CompileResult<'src, DependencyResolution<'src>> {
    let name = dependency.recipe.last().lexeme();

    if dependency.recipe.components() > 1 {
      // recipe is in a submodule and is thus already resolved
      Ok(self.resolve_submodule_dependency(&dependency.recipe))
    } else if let Some(resolved) = self.resolved_recipes.get(name) {
      // recipe is the current module and has already been resolved
      Ok(DependencyResolution::Resolved(Arc::clone(resolved)))
    } else if let Some(disabled) = self.disabled.get(name) {
      // recipe is in the current module and has already been disabled
      Ok(DependencyResolution::Disabled(disabled.modules.clone()))
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
      Ok(match self.resolve_recipe(stack, unresolved)? {
        Resolution::Resolved(resolved) => DependencyResolution::Resolved(resolved),
        Resolution::Disabled(modules) => DependencyResolution::Disabled(modules),
      })
    } else {
      // recipe is unknown
      Ok(DependencyResolution::Unknown)
    }
  }

  fn resolve_submodule_dependency(&self, path: &Namepath<'src>) -> DependencyResolution<'src> {
    let (name, prefix) = path.split_last();

    let mut modules = self.modules;
    let mut recipes = &self.resolved_recipes;
    let mut absent = self.absent;
    let mut disabled = &self.disabled;
    let mut walked = Vec::new();

    for component in prefix {
      let lexeme = component.lexeme();
      walked.push(lexeme.to_string());

      if let Some(module) = modules.get(lexeme) {
        modules = &module.modules;
        recipes = &module.recipes;
        absent = &module.absent;
        disabled = &module.disabled;
      } else if absent.contains(lexeme) {
        return DependencyResolution::Disabled(BTreeSet::from([Modulepath {
          components: walked,
          spaced: false,
        }]));
      } else {
        return DependencyResolution::Unknown;
      }
    }

    if let Some(resolved) = recipes.get(name.lexeme()) {
      DependencyResolution::Resolved(Arc::clone(resolved))
    } else if let Some(disabled) = disabled.get(name.lexeme()) {
      DependencyResolution::Disabled(disabled.modules.clone())
    } else {
      DependencyResolution::Unknown
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
