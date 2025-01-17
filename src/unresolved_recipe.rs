use super::*;

pub(crate) type UnresolvedRecipe<'src> = Recipe<'src, UnresolvedDependency<'src>>;

impl<'src> UnresolvedRecipe<'src> {
  pub(crate) fn resolve(
    self,
    resolved: Vec<Rc<Recipe<'src>>>,
  ) -> CompileResult<'src, Recipe<'src>> {
    assert_eq!(
      self.dependencies.len(),
      resolved.len(),
      "UnresolvedRecipe::resolve: dependency count not equal to resolved count: {} != {}",
      self.dependencies.len(),
      resolved.len()
    );

    for (unresolved, resolved) in self.dependencies.iter().zip(&resolved) {
      assert_eq!(unresolved.recipe.lexeme(), resolved.name.lexeme());
      if !resolved
        .argument_range()
        .contains(&unresolved.arguments.len())
      {
        return Err(
          unresolved
            .recipe
            .error(CompileErrorKind::DependencyArgumentCountMismatch {
              dependency: unresolved.recipe.lexeme(),
              found: unresolved.arguments.len(),
              min: resolved.min_arguments(),
              max: resolved.max_arguments(),
            }),
        );
      }
    }

    let statements = if self.attributes.contains(AttributeDiscriminant::NoShell) {
      let mut statements = Vec::new();

      for line in &self.body {
        let mut words = Vec::new();
        for fragment in &line.fragments {
          match fragment {
            Fragment::Text { token } => {
              for word in token.lexeme().split_whitespace() {
                words.push(Word::Text(word.into()));
              }
            }
            Fragment::Interpolation { expression } => {
              words.push(Word::Expression(expression.clone()));
            }
          }
        }

        statements.push(Statement { words });
      }

      Some(statements)
    } else {
      None
    };

    let dependencies = self
      .dependencies
      .into_iter()
      .zip(resolved)
      .map(|(unresolved, resolved)| Dependency {
        recipe: resolved,
        arguments: unresolved.arguments,
      })
      .collect();

    Ok(Recipe {
      attributes: self.attributes,
      body: self.body,
      dependencies,
      doc: self.doc,
      file_depth: self.file_depth,
      import_offsets: self.import_offsets,
      name: self.name,
      namepath: self.namepath,
      parameters: self.parameters,
      priors: self.priors,
      private: self.private,
      quiet: self.quiet,
      shebang: self.shebang,
      statements,
    })
  }
}
