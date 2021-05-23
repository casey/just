use crate::common::*;

pub(crate) type UnresolvedRecipe<'src> = Recipe<'src, UnresolvedDependency<'src>>;

impl<'src> UnresolvedRecipe<'src> {
  pub(crate) fn resolve(
    self,
    resolved: Vec<Rc<Recipe<'src>>>,
  ) -> CompilationResult<'src, Recipe<'src>> {
    assert_eq!(self.dependencies.len(), resolved.len());
    for (unresolved, resolved) in self.dependencies.iter().zip(&resolved) {
      assert_eq!(unresolved.recipe.lexeme(), resolved.name.lexeme());
      if !resolved
        .argument_range()
        .contains(&unresolved.arguments.len())
      {
        return Err(unresolved.recipe.error(
          CompilationErrorKind::DependencyArgumentCountMismatch {
            dependency: unresolved.recipe.lexeme(),
            found:      unresolved.arguments.len(),
            min:        resolved.min_arguments(),
            max:        resolved.max_arguments(),
          },
        ));
      }
    }

    let dependencies = self
      .dependencies
      .into_iter()
      .zip(resolved)
      .map(|(unresolved, resolved)| Dependency {
        recipe:    resolved,
        arguments: unresolved.arguments,
      })
      .collect();

    Ok(Recipe {
      doc: self.doc,
      body: self.body,
      name: self.name,
      parameters: self.parameters,
      private: self.private,
      quiet: self.quiet,
      shebang: self.shebang,
      dependencies,
    })
  }
}

impl<'src> Display for UnresolvedRecipe<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if let Some(doc) = self.doc {
      writeln!(f, "# {}", doc)?;
    }

    if self.quiet {
      write!(f, "@{}", self.name)?;
    } else {
      write!(f, "{}", self.name)?;
    }

    for parameter in &self.parameters {
      write!(f, " {}", parameter)?;
    }
    write!(f, ":")?;
    for dependency in &self.dependencies {
      write!(f, " {}", dependency)?;
    }

    for (i, line) in self.body.iter().enumerate() {
      if i == 0 {
        writeln!(f)?;
      }
      for (j, fragment) in line.fragments.iter().enumerate() {
        if j == 0 {
          write!(f, "    ")?;
        }
        match fragment {
          Fragment::Text { token } => write!(f, "{}", token.lexeme())?,
          Fragment::Interpolation { expression, .. } => write!(f, "{{{{{}}}}}", expression)?,
        }
      }
      if i + 1 < self.body.len() {
        writeln!(f)?;
      }
    }
    Ok(())
  }
}
