use crate::common::*;
use CompilationErrorKind::*;

pub(crate) struct AliasResolver<'a, 'b>
where
  'a: 'b,
{
  aliases: &'b Table<'a, Alias<'a>>,
  recipes: &'b Table<'a, Rc<Recipe<'a>>>,
}

impl<'a: 'b, 'b> AliasResolver<'a, 'b> {
  pub(crate) fn resolve_aliases(
    aliases: &Table<'a, Alias<'a>>,
    recipes: &Table<'a, Rc<Recipe<'a>>>,
  ) -> CompilationResult<'a, ()> {
    let resolver = AliasResolver { aliases, recipes };

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
    let token = alias.name.token();
    // Make sure the alias doesn't conflict with any recipe
    if let Some(recipe) = self.recipes.get(alias.name.lexeme()) {
      return Err(token.error(AliasShadowsRecipe {
        alias: alias.name.lexeme(),
        recipe_line: recipe.line_number(),
      }));
    }

    // Make sure the target recipe exists
    if self.recipes.get(alias.target.lexeme()).is_none() {
      return Err(token.error(UnknownAliasTarget {
        alias: alias.name.lexeme(),
        target: alias.target.lexeme(),
      }));
    }

    Ok(())
  }
}
