use crate::common::*;

use CompilationErrorKind::*;

pub(crate) struct Analyzer<'a> {
  recipes: Table<'a, Recipe<'a>>,
  assignments: Table<'a, Assignment<'a>>,
  aliases: Table<'a, Alias<'a>>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn analyze(module: Module<'a>) -> CompilationResult<'a, Justfile> {
    let analyzer = Analyzer::new();

    analyzer.justfile(module)
  }

  pub(crate) fn new() -> Analyzer<'a> {
    Analyzer {
      recipes: empty(),
      assignments: empty(),
      aliases: empty(),
    }
  }

  pub(crate) fn justfile(mut self, module: Module<'a>) -> CompilationResult<'a, Justfile<'a>> {
    for item in module.items {
      match item {
        Item::Alias(alias) => {
          self.analyze_alias(&alias)?;
          self.aliases.insert(alias);
        }
        Item::Assignment(assignment) => {
          self.analyze_assignment(&assignment)?;
          self.assignments.insert(assignment);
        }
        Item::Recipe(recipe) => {
          self.analyze_recipe(&recipe)?;
          self.recipes.insert(recipe);
        }
      }
    }

    let recipes = self.recipes;
    let assignments = self.assignments;
    let aliases = self.aliases;

    AssignmentResolver::resolve_assignments(&assignments)?;

    RecipeResolver::resolve_recipes(&recipes, &assignments)?;

    for recipe in recipes.values() {
      for parameter in &recipe.parameters {
        if assignments.contains_key(parameter.name.lexeme()) {
          return Err(parameter.name.token().error(ParameterShadowsVariable {
            parameter: parameter.name.lexeme(),
          }));
        }
      }

      for dependency in &recipe.dependencies {
        if !recipes[dependency.lexeme()].parameters.is_empty() {
          return Err(dependency.error(DependencyHasParameters {
            recipe: recipe.name(),
            dependency: dependency.lexeme(),
          }));
        }
      }
    }

    AliasResolver::resolve_aliases(&aliases, &recipes)?;

    Ok(Justfile {
      warnings: module.warnings,
      recipes,
      assignments,
      aliases,
    })
  }

  fn analyze_recipe(&self, recipe: &Recipe<'a>) -> CompilationResult<'a, ()> {
    if let Some(original) = self.recipes.get(recipe.name.lexeme()) {
      return Err(recipe.name.token().error(DuplicateRecipe {
        recipe: original.name(),
        first: original.line_number(),
      }));
    }

    let mut parameters = BTreeSet::new();
    let mut passed_default = false;

    for parameter in &recipe.parameters {
      if parameters.contains(parameter.name.lexeme()) {
        return Err(parameter.name.token().error(DuplicateParameter {
          recipe: recipe.name.lexeme(),
          parameter: parameter.name.lexeme(),
        }));
      }
      parameters.insert(parameter.name.lexeme());

      if parameter.default.is_some() {
        passed_default = true;
      } else if passed_default {
        return Err(
          parameter
            .name
            .token()
            .error(RequiredParameterFollowsDefaultParameter {
              parameter: parameter.name.lexeme(),
            }),
        );
      }
    }

    let mut dependencies = BTreeSet::new();
    for dependency in &recipe.dependencies {
      if dependencies.contains(dependency.lexeme()) {
        return Err(dependency.token().error(DuplicateDependency {
          recipe: recipe.name.lexeme(),
          dependency: dependency.lexeme(),
        }));
      }
      dependencies.insert(dependency.lexeme());
    }

    let mut continued = false;
    for line in &recipe.body {
      if !recipe.shebang && !continued {
        if let Some(Fragment::Text { token }) = line.fragments.first() {
          let text = token.lexeme();

          if text.starts_with(' ') || text.starts_with('\t') {
            return Err(token.error(ExtraLeadingWhitespace));
          }
        }
      }

      continued = line.is_continuation();
    }

    Ok(())
  }

  fn analyze_assignment(&self, assignment: &Assignment<'a>) -> CompilationResult<'a, ()> {
    if self.assignments.contains_key(assignment.name.lexeme()) {
      return Err(assignment.name.token().error(DuplicateVariable {
        variable: assignment.name.lexeme(),
      }));
    }
    Ok(())
  }

  fn analyze_alias(&self, alias: &Alias<'a>) -> CompilationResult<'a, ()> {
    let name = alias.name.lexeme();

    if let Some(original) = self.aliases.get(name) {
      return Err(alias.name.token().error(DuplicateAlias {
        alias: name,
        first: original.line_number(),
      }));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  analysis_error! {
    name: duplicate_alias,
    input: "alias foo = bar\nalias foo = baz",
    offset: 22,
    line: 1,
    column: 6,
    width: 3,
    kind: DuplicateAlias { alias: "foo", first: 0 },
  }

  analysis_error! {
    name: unknown_alias_target,
    input: "alias foo = bar\n",
    offset: 6,
    line: 0,
    column: 6,
    width: 3,
    kind: UnknownAliasTarget {alias: "foo", target: "bar"},
  }

  analysis_error! {
    name: alias_shadows_recipe_before,
    input: "bar: \n  echo bar\nalias foo = bar\nfoo:\n  echo foo",
    offset: 23,
    line: 2,
    column: 6,
    width: 3,
    kind: AliasShadowsRecipe {alias: "foo", recipe_line: 3},
  }

  analysis_error! {
    name: alias_shadows_recipe_after,
    input: "foo:\n  echo foo\nalias foo = bar\nbar:\n  echo bar",
    offset: 22,
    line: 2,
    column: 6,
    width: 3,
    kind: AliasShadowsRecipe { alias: "foo", recipe_line: 0 },
  }

  analysis_error! {
    name:   required_after_default,
    input:  "hello arg='foo' bar:",
    offset:  16,
    line:   0,
    column: 16,
    width:  3,
    kind:   RequiredParameterFollowsDefaultParameter{parameter: "bar"},
  }

  analysis_error! {
    name:   duplicate_parameter,
    input:  "a b b:",
    offset:  4,
    line:   0,
    column: 4,
    width:  1,
    kind:   DuplicateParameter{recipe: "a", parameter: "b"},
  }

  analysis_error! {
    name:   duplicate_variadic_parameter,
    input:  "a b +b:",
    offset: 5,
    line:   0,
    column: 5,
    width:  1,
    kind:   DuplicateParameter{recipe: "a", parameter: "b"},
  }

  analysis_error! {
    name:   parameter_shadows_varible,
    input:  "foo = \"h\"\na foo:",
    offset:  12,
    line:   1,
    column: 2,
    width:  3,
    kind:   ParameterShadowsVariable{parameter: "foo"},
  }

  analysis_error! {
    name:   dependency_has_parameters,
    input:  "foo arg:\nb: foo",
    offset:  12,
    line:   1,
    column: 3,
    width:  3,
    kind:   DependencyHasParameters{recipe: "b", dependency: "foo"},
  }

  analysis_error! {
    name:   duplicate_dependency,
    input:  "a b c: b c z z",
    offset:  13,
    line:   0,
    column: 13,
    width:  1,
    kind:   DuplicateDependency{recipe: "a", dependency: "z"},
  }

  analysis_error! {
    name:   duplicate_recipe,
    input:  "a:\nb:\na:",
    offset:  6,
    line:   2,
    column: 0,
    width:  1,
    kind:   DuplicateRecipe{recipe: "a", first: 0},
  }

  analysis_error! {
    name:   duplicate_variable,
    input:  "a = \"0\"\na = \"0\"",
    offset:  8,
    line:   1,
    column: 0,
    width:  1,
    kind:   DuplicateVariable{variable: "a"},
  }

  analysis_error! {
    name:   extra_whitespace,
    input:  "a:\n blah\n  blarg",
    offset:  10,
    line:   2,
    column: 1,
    width:  6,
    kind:   ExtraLeadingWhitespace,
  }
}
