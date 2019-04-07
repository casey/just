use common::*;
use CompilationErrorKind::*;

pub struct AliasResolver<'a, 'b> where 'a: 'b {
  aliases: &'b Map<&'a str, Alias<'a>>,
  recipes: &'b Map<&'a str, Recipe<'a>>,
}

impl<'a: 'b, 'b> AliasResolver<'a, 'b> {
  pub fn resolve_aliases(
    aliases: &Map<&'a str, Alias<'a>>,
    recipes: &Map<&'a str, Recipe<'a>>,
  ) -> CompilationResult<'a, ()> {
    let resolver = AliasResolver {
      aliases,
      recipes,
    };

    resolver.resolve()?;

    Ok(())
  }

  fn resolve(&self) -> CompilationResult<'a, ()> {
    for alias in self.aliases.values() {
      self.resolve_alias(alias)?;
    }

    Ok(())
  }

  fn resolve_alias(&self, alias: &Alias<'a>) -> CompilationResult<'a, ()> {
    // Make sure the alias doesn't conflict with any recipe
    if let Some(recipe) = self.recipes.get(alias.name) {
      let error_kind = AliasShadowsRecipe { alias: alias.name, recipe_line: recipe.line_number };
      return Err(CompilationError {
        text: "", index: 0, line: alias.line_number, column: 0,
        width: None, 
        kind: error_kind,
      });
    }

    // Make sure the target recipe exists 
    if None == self.recipes.get(alias.target) {
      let error_kind = UnknownAliasTarget {
        alias: alias.name,
        target: alias.target,
      };
      return Err(CompilationError {
        text: "",
        index: 0,
        line: alias.line_number,
        column: 0,
        width: None,
        kind: error_kind,
      })
    }

    Ok(())
  }
}