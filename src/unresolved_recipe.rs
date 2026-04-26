use super::*;

pub(crate) type UnresolvedRecipe<'src> = Recipe<'src, UnresolvedDependency<'src>>;

impl<'src> UnresolvedRecipe<'src> {
  pub(crate) fn resolve(
    self,
    assignments: &Table<'src, Assignment<'src>>,
    functions: &Table<'src, FunctionDefinition<'src>>,
    modulepath: &Modulepath,
    resolved: Vec<Arc<Recipe<'src>>>,
    settings: &Settings,
  ) -> CompileResult<'src, Recipe<'src>> {
    assert_eq!(
      self.dependencies.len(),
      resolved.len(),
      "UnresolvedRecipe::resolve: dependency count not equal to resolved count: {} != {}",
      self.dependencies.len(),
      resolved.len()
    );

    let mut variable_references = HashSet::new();

    for (i, parameter) in self.parameters.iter().enumerate() {
      if let Some(expression) = &parameter.default {
        Self::resolve_expression(
          assignments,
          expression,
          functions,
          &self.parameters[..i],
          &mut variable_references,
        )?;
      }
    }

    for dependency in &self.dependencies {
      for argument in &dependency.arguments {
        Self::resolve_expression(
          assignments,
          argument,
          functions,
          &self.parameters,
          &mut variable_references,
        )?;
      }
    }

    for attribute in &self.attributes {
      if let Attribute::Confirm(Some(expression)) | Attribute::WorkingDirectory(expression) =
        attribute
      {
        Self::resolve_expression(
          assignments,
          expression,
          functions,
          &self.parameters,
          &mut variable_references,
        )?;
      }
    }

    for line in &self.body {
      if line.is_comment() && settings.ignore_comments {
        continue;
      }

      for fragment in &line.fragments {
        if let Fragment::Interpolation { expression, .. } = fragment {
          Self::resolve_expression(
            assignments,
            expression,
            functions,
            &self.parameters,
            &mut variable_references,
          )?;
        }
      }
    }

    for (unresolved, resolved) in self.dependencies.iter().zip(&resolved) {
      assert_eq!(unresolved.recipe.last().lexeme(), resolved.name.lexeme());
      if !resolved
        .argument_range()
        .contains(&unresolved.arguments.len())
      {
        return Err(unresolved.recipe.last().error(
          CompileErrorKind::DependencyArgumentCountMismatch {
            dependency: unresolved.recipe.clone(),
            found: unresolved.arguments.len(),
            min: resolved.min_arguments(),
            max: resolved.max_arguments(),
          },
        ));
      }
    }

    let dependencies = self
      .dependencies
      .into_iter()
      .zip(resolved)
      .map(|(unresolved, resolved)| Dependency {
        arguments: resolved.group_arguments(&unresolved.arguments),
        recipe: resolved,
      })
      .collect();

    let mut recipe_path = modulepath.clone();

    recipe_path.components.push(self.name.lexeme().into());

    Ok(Recipe {
      attributes: self.attributes,
      body: self.body,
      dependencies,
      doc: self.doc,
      file_depth: self.file_depth,
      import_offsets: self.import_offsets,
      module_path: Some(modulepath.clone()),
      name: self.name,
      parameters: self.parameters,
      priors: self.priors,
      private: self.private,
      quiet: self.quiet,
      recipe_path: Some(recipe_path),
      shebang: self.shebang,
      variable_references,
    })
  }

  fn resolve_expression(
    assignments: &Table<'src, Assignment<'src>>,
    expression: &Expression<'src>,
    functions: &Table<'src, FunctionDefinition<'src>>,
    parameters: &[Parameter],
    variable_references: &mut HashSet<Number>,
  ) -> CompileResult<'src> {
    for reference in expression.references() {
      match reference {
        Reference::Variable(variable) => {
          Self::resolve_variable(assignments, parameters, variable, variable_references)?;
        }
        Reference::Call { name, arguments } => {
          Analyzer::resolve_call(functions, name, arguments)?;
        }
      }
    }
    Ok(())
  }

  fn resolve_variable(
    assignments: &Table<'src, Assignment<'src>>,
    parameters: &[Parameter],
    variable: Name<'src>,
    variable_references: &mut HashSet<Number>,
  ) -> CompileResult<'src> {
    let name = variable.lexeme();

    if parameters.iter().any(|p| p.name.lexeme() == name) {
      Ok(())
    } else if let Some(assignment) = assignments.get(name) {
      variable_references.insert(assignment.number);
      Ok(())
    } else if constants().contains_key(name) {
      Ok(())
    } else {
      Err(variable.error(CompileErrorKind::UndefinedVariable { variable: name }))
    }
  }
}
