use {super::*, CompileErrorKind::*};

pub(crate) struct RecipeResolver<'src: 'run, 'run> {
  absent_modules: &'run BTreeSet<String>,
  disabled_recipes: Table<'src, Disabled<'src>>,
  evaluator: &'run mut Evaluator<'src, 'run>,
  modulepath: &'run Modulepath,
  modules: &'run Table<'src, Justfile<'src>>,
  resolved_recipes: Table<'src, Arc<Recipe<'src>>>,
  settings: &'run Settings,
  unresolved_recipes: Table<'src, UnresolvedRecipe<'src>>,
  variable_resolver: &'run VariableResolver<'src, 'run>,
}

impl<'src: 'run, 'run> RecipeResolver<'src, 'run> {
  pub(crate) fn resolve_recipes(
    absent_modules: &'run BTreeSet<String>,
    evaluator: &'run mut Evaluator<'src, 'run>,
    modulepath: &'run Modulepath,
    modules: &'run Table<'src, Justfile<'src>>,
    settings: &'run Settings,
    unresolved_recipes: Table<'src, UnresolvedRecipe<'src>>,
    variable_resolver: &'run VariableResolver<'src, 'run>,
  ) -> CompileResult<'src, (Table<'src, Arc<Recipe<'src>>>, Table<'src, Disabled<'src>>)> {
    let mut resolver = Self {
      absent_modules,
      disabled_recipes: Table::new(),
      evaluator,
      modulepath,
      modules,
      resolved_recipes: Table::new(),
      settings,
      unresolved_recipes,
      variable_resolver,
    };

    while let Some(unresolved) = resolver.unresolved_recipes.pop() {
      resolver.resolve_recipe(&mut Vec::new(), unresolved)?;
    }

    Ok((resolver.resolved_recipes, resolver.disabled_recipes))
  }

  fn resolve_recipe(
    &mut self,
    stack: &mut Vec<&'src str>,
    recipe: UnresolvedRecipe<'src>,
  ) -> CompileResult<'src, Resolution<Arc<Recipe<'src>>>> {
    if let Some(resolved) = self.resolved_recipes.get(recipe.name()) {
      return Ok(Resolution::Resolved(Arc::clone(resolved)));
    }

    if let Some(disabled) = self.disabled_recipes.get(recipe.name()) {
      return Ok(Resolution::Disabled(disabled.modules.clone()));
    }

    stack.push(recipe.name());

    let mut dependencies = Vec::new();
    let mut disabled_by = BTreeSet::new();

    for dependency in &recipe.dependencies {
      match self
        .resolve_dependency(dependency, &recipe, stack)?
        .ok_or_else(|| {
          dependency.recipe.last().error(UnknownDependency {
            recipe: recipe.name(),
            unknown: dependency.recipe.clone(),
          })
        })? {
        Resolution::Resolved(resolved) => dependencies.push(resolved),
        Resolution::Disabled(modules) => disabled_by.extend(modules),
      }
    }

    stack.pop();

    if disabled_by.is_empty() {
      let resolved = Arc::new(recipe.resolve(
        self.evaluator,
        self.modulepath,
        dependencies,
        self.settings,
        self.variable_resolver,
      )?);
      self.resolved_recipes.insert(Arc::clone(&resolved));
      Ok(Resolution::Resolved(resolved))
    } else {
      self.disabled_recipes.insert(Disabled {
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
  ) -> CompileResult<'src, Option<Resolution<Arc<Recipe<'src>>>>> {
    let name = dependency.recipe.last().lexeme();

    if dependency.recipe.components() > 1 {
      // recipe is in a submodule and is thus already resolved
      Ok(Resolution::resolve_recipe(
        &dependency.recipe,
        self.absent_modules,
        &self.disabled_recipes,
        self.modules,
        &self.resolved_recipes,
      ))
    } else if let Some(resolved) = self.resolved_recipes.get(name) {
      // recipe is the current module and has already been resolved
      Ok(Some(Resolution::Resolved(Arc::clone(resolved))))
    } else if let Some(disabled) = self.disabled_recipes.get(name) {
      // recipe is in the current module and has already been disabled
      Ok(Some(Resolution::Disabled(disabled.modules.clone())))
    } else if stack.contains(&name) {
      // recipe depends on itself
      stack.push(name);
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
