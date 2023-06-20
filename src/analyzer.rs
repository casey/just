use {super::*, CompileErrorKind::*};

const VALID_ALIAS_ATTRIBUTES: [Attribute; 1] = [Attribute::Private];

#[derive(Debug)]
pub(crate) struct Import {
  path: PathBuf,
  line: usize,
}

impl Import {
  pub(crate) fn path(&self) -> &Path {
    self.path.as_ref()
  }
}

#[derive(Default)]
pub(crate) struct Analyzer<'src> {
  assignments: Table<'src, Assignment<'src>>,
  aliases: Table<'src, Alias<'src, Name<'src>>>,
  sets: Table<'src, Set<'src>>,
}

impl<'src> Analyzer<'src> {
  /// Inspect an AST for nodes representing an import of another justfile and collect them
  pub(crate) fn get_imports(ast: &Ast<'src>) -> Vec<Import> {
    ast
      .items
      .iter()
      .filter_map(|item| {
        if let Item::Include { name, path } = item {
          Some(Import {
            path: Path::new(path).to_owned(),
            line: name.line,
          })
        } else {
          None
        }
      })
      .collect()
  }

  /// Peform analysis on a root Ast, which may have imported other justfiles provided in
  /// `imported_asts`
  pub(crate) fn analyze<'a>(
    root_ast: &'a Ast<'src>,
    imported_asts: &'a [AstImport<'src>],
  ) -> CompileResult<'src, Justfile<'src>> {
    let mut analyzer = Analyzer::default();
    let unresolved_recipes = analyzer.build_tables(root_ast, imported_asts)?;

    let settings = Settings::from_setting_iter(analyzer.sets.into_iter().map(|(_, set)| set.value));
    AssignmentResolver::resolve_assignments(&analyzer.assignments)?;

    let recipes = Analyzer::resolve_recipes(unresolved_recipes, &settings, &analyzer.assignments)?;

    let mut aliases = Table::new();
    while let Some(alias) = analyzer.aliases.pop() {
      aliases.insert(Self::resolve_alias(&recipes, alias)?);
    }

    Ok(Justfile {
      warnings: root_ast.warnings.clone(),
      first: recipes
        .values()
        .fold(None, |accumulator, next| match accumulator {
          None => Some(Rc::clone(next)),
          Some(previous) => Some(if previous.line_number() < next.line_number() {
            previous
          } else {
            Rc::clone(next)
          }),
        }),
      aliases,
      assignments: analyzer.assignments,
      recipes,
      settings,
    })
  }

  fn build_tables<'a>(
    &mut self,
    root_ast: &'a Ast<'src>,
    imported_asts: &'a [AstImport<'src>],
  ) -> CompileResult<'src, Vec<&'a Recipe<'src, UnresolvedDependency<'src>>>> {
    let mut recipes = Vec::new();
    recipes.extend(self.build_table_from_items(&root_ast.items)?.into_iter());

    for import in imported_asts {
      recipes.extend(self.build_table_from_items(&import.ast.items)?.into_iter());
    }
    Ok(recipes)
  }

  fn build_table_from_items<'a>(
    &mut self,
    items: &'a [Item<'src>],
  ) -> CompileResult<'src, Vec<&'a Recipe<'src, UnresolvedDependency<'src>>>> {
    let mut recipes = Vec::new();

    for item in items {
      match item {
        Item::Alias(alias) => {
          self.analyze_alias(alias)?;
          self.aliases.insert(alias.clone());
        }
        Item::Assignment(assignment) => {
          self.analyze_assignment(assignment)?;
          self.assignments.insert(assignment.clone());
        }
        Item::Comment(_) | Item::Include { .. } => (),
        Item::Recipe(recipe) => {
          if recipe.enabled() {
            Self::analyze_recipe(recipe)?;
            recipes.push(recipe);
          }
        }
        Item::Set(set) => {
          self.analyze_set(set)?;
          self.sets.insert(set.clone());
        }
      }
    }

    Ok(recipes)
  }

  fn resolve_recipes<'a>(
    recipes: Vec<&'a Recipe<'src, UnresolvedDependency<'src>>>,
    settings: &Settings<'src>,
    assignments: &Table<'src, Assignment<'src>>,
  ) -> CompileResult<'src, Table<'src, Rc<Recipe<'src>>>> {
    let mut recipe_table: Table<'src, UnresolvedRecipe<'src>> = Table::default();

    for recipe in recipes {
      if let Some(original) = recipe_table.get(recipe.name.lexeme()) {
        if !settings.allow_duplicate_recipes {
          return Err(recipe.name.token().error(DuplicateRecipe {
            recipe: original.name(),
            first: original.line_number(),
          }));
        }
      }
      recipe_table.insert(recipe.clone());
    }

    let recipes = RecipeResolver::resolve_recipes(recipe_table, assignments)?;
    Ok(recipes)
  }

  fn analyze_recipe(recipe: &UnresolvedRecipe<'src>) -> CompileResult<'src, ()> {
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

  fn analyze_assignment(&self, assignment: &Assignment<'src>) -> CompileResult<'src, ()> {
    if self.assignments.contains_key(assignment.name.lexeme()) {
      return Err(assignment.name.token().error(DuplicateVariable {
        variable: assignment.name.lexeme(),
      }));
    }
    Ok(())
  }

  fn analyze_alias(&self, alias: &Alias<'src, Name<'src>>) -> CompileResult<'src, ()> {
    let name = alias.name.lexeme();

    if let Some(original) = self.aliases.get(name) {
      return Err(alias.name.token().error(DuplicateAlias {
        alias: name,
        first: original.line_number(),
      }));
    }

    for attr in &alias.attributes {
      if !VALID_ALIAS_ATTRIBUTES.contains(attr) {
        return Err(alias.name.token().error(AliasInvalidAttribute {
          alias: name,
          attr: *attr,
        }));
      }
    }

    Ok(())
  }

  fn analyze_set(&self, set: &Set<'src>) -> CompileResult<'src, ()> {
    if let Some(original) = self.sets.get(set.name.lexeme()) {
      return Err(set.name.error(DuplicateSet {
        setting: original.name.lexeme(),
        first: original.name.line,
      }));
    }

    Ok(())
  }

  fn resolve_alias(
    recipes: &Table<'src, Rc<Recipe<'src>>>,
    alias: Alias<'src, Name<'src>>,
  ) -> CompileResult<'src, Alias<'src>> {
    let token = alias.name.token();
    // Make sure the alias doesn't conflict with any recipe
    if let Some(recipe) = recipes.get(alias.name.lexeme()) {
      return Err(token.error(AliasShadowsRecipe {
        alias: alias.name.lexeme(),
        recipe_line: recipe.line_number(),
      }));
    }

    // Make sure the target recipe exists
    match recipes.get(alias.target.lexeme()) {
      Some(target) => Ok(alias.resolve(Rc::clone(target))),
      None => Err(token.error(UnknownAliasTarget {
        alias: alias.name.lexeme(),
        target: alias.target.lexeme(),
      })),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  analysis_error! {
    name: duplicate_alias,
    input: "alias foo := bar\nalias foo := baz",
    offset: 23,
    line: 1,
    column: 6,
    width: 3,
    kind: DuplicateAlias { alias: "foo", first: 0 },
  }

  analysis_error! {
    name: unknown_alias_target,
    input: "alias foo := bar\n",
    offset: 6,
    line: 0,
    column: 6,
    width: 3,
    kind: UnknownAliasTarget {alias: "foo", target: "bar"},
  }

  analysis_error! {
    name: alias_shadows_recipe_before,
    input: "bar: \n  echo bar\nalias foo := bar\nfoo:\n  echo foo",
    offset: 23,
    line: 2,
    column: 6,
    width: 3,
    kind: AliasShadowsRecipe {alias: "foo", recipe_line: 3},
  }

  analysis_error! {
    name: alias_shadows_recipe_after,
    input: "foo:\n  echo foo\nalias foo := bar\nbar:\n  echo bar",
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
    input:  "a := \"0\"\na := \"0\"",
    offset: 9,
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
