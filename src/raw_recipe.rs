use crate::common::*;

pub(crate) type RawRecipe<'src> = Recipe<'src, RawDependency<'src>>;

impl<'src> RawRecipe<'src> {
  pub(crate) fn resolve(self, resolved: Vec<Dependency<'src>>) -> Recipe<'src> {
    assert_eq!(self.dependencies.len(), resolved.len());
    for (unresolved, resolved) in self.dependencies.iter().zip(&resolved) {
      assert_eq!(unresolved.recipe.lexeme(), resolved.recipe.name.lexeme());
    }
    Recipe {
      dependencies: resolved,
      doc: self.doc,
      body: self.body,
      name: self.name,
      parameters: self.parameters,
      private: self.private,
      quiet: self.quiet,
      shebang: self.shebang,
    }
  }
}
