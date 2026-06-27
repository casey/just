use super::*;

pub(crate) type UnresolvedRecipe<'src> = Recipe<'src, UnresolvedDependency<'src>>;

fn parse_count(value: &str) -> Option<usize> {
  static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^(0|[1-9][0-9]*)$").unwrap());

  if REGEX.is_match(value) {
    value.parse().ok()
  } else {
    None
  }
}

impl<'src> UnresolvedRecipe<'src> {
  pub(crate) fn resolve(
    mut self,
    assignments: &Table<'src, Assignment<'src>>,
    evaluator: &mut Evaluator<'src, '_>,
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
      if let Some(expression) = &parameter.value {
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
      if dependency.starred() && !settings.lists {
        return Err(
          dependency
            .recipe
            .last()
            .error(CompileErrorKind::MappedDependencyWithoutListsSetting),
        );
      }

      for argument in &dependency.arguments {
        Self::resolve_expression(
          assignments,
          &argument.expression,
          functions,
          &self.parameters,
          &mut variable_references,
        )?;
      }
    }

    for attribute in &self.attributes {
      let mut resolve_expression = |expression, parameters| {
        Self::resolve_expression(
          assignments,
          expression,
          functions,
          parameters,
          &mut variable_references,
        )
      };

      match attribute {
        Attribute::Confirm(Some(expression)) | Attribute::WorkingDirectory(expression) => {
          resolve_expression(expression, &self.parameters)?;
        }
        Attribute::Arg {
          help_property,
          pattern_property,
          ..
        } => {
          if let Some((_key, expression)) = help_property {
            resolve_expression(expression, &[])?;
          }
          if let Some((_key, expression)) = pattern_property {
            resolve_expression(expression, &[])?;
          }
        }
        Attribute::Env(key, value) => {
          resolve_expression(key, &[])?;
          resolve_expression(value, &[])?;
        }
        _ => {}
      }
    }

    let attributes = self
      .attributes
      .into_items()
      .map(|(mut attribute, name)| {
        if let Attribute::Arg {
          help,
          help_property: Some((_key, expression)),
          name: arg,
          ..
        } = &mut attribute
        {
          let value = evaluator.evaluate_value_const(expression)?;
          if !value.is_empty() {
            let value = value.join();
            self
              .parameters
              .iter_mut()
              .find(|parameter| parameter.name.lexeme() == arg.cooked)
              .unwrap()
              .help = Some(value.clone());
            *help = Some(value);
          }
        }

        if let Attribute::Arg {
          name: arg,
          pattern,
          pattern_property: Some((key, expression)),
          ..
        } = &mut attribute
        {
          let value = evaluator.evaluate_value_const(expression)?;
          if !value.is_empty() {
            let compiled = Pattern::new(&value, *key)?;
            self
              .parameters
              .iter_mut()
              .find(|parameter| parameter.name.lexeme() == arg.cooked)
              .unwrap()
              .pattern = Some(compiled.clone());
            *pattern = Some(compiled);
          }
        }

        if let Attribute::Arg {
          max,
          min,
          name: arg,
          ..
        } = &attribute
          && (min.is_some() || max.is_some())
        {
          let parameter = self
            .parameters
            .iter()
            .find(|parameter| parameter.name.lexeme() == arg.cooked)
            .unwrap();

          let variadic = parameter.kind.is_variadic();
          let has_default = parameter.default.is_some();

          let key = min
            .as_ref()
            .map(|(key, _)| *key)
            .or_else(|| max.as_ref().map(|(key, _)| *key))
            .unwrap();

          if variadic {
            return Err(key.error(CompileErrorKind::ArgAttributeVariadicMinMax {
              parameter: arg.cooked.clone(),
            }));
          }

          let min = if let Some((key, expression)) = min {
            let value = evaluator.evaluate_value_const(expression)?.join();
            parse_count(&value).ok_or_else(|| {
              key.error(CompileErrorKind::ArgAttributeExpectedInteger {
                key: "min",
                value: value.clone(),
              })
            })?
          } else {
            0
          };

          let max = if let Some((key, Some(expression))) = max {
            let value = evaluator.evaluate_value_const(expression)?.join();
            Some(parse_count(&value).ok_or_else(|| {
              key.error(CompileErrorKind::ArgAttributeExpectedInteger {
                key: "max",
                value: value.clone(),
              })
            })?)
          } else {
            None
          };

          if let Some(max) = max
            && min > max
          {
            return Err(key.error(CompileErrorKind::ArgAttributeMinGreaterThanMax {
              parameter: arg.cooked.clone(),
            }));
          }

          if has_default && min > 0 {
            return Err(key.error(CompileErrorKind::ArgAttributeDefaultWithMin {
              parameter: arg.cooked.clone(),
            }));
          }

          self
            .parameters
            .iter_mut()
            .find(|parameter| parameter.name.lexeme() == arg.cooked)
            .unwrap()
            .bound = Some(Bound { max, min });
        }

        Ok((attribute, name))
      })
      .collect::<CompileResult<AttributeSet>>()?;

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
        .argument_range(settings)
        .contains(&unresolved.arguments.len())
      {
        return Err(unresolved.recipe.last().error(
          CompileErrorKind::DependencyArgumentCountMismatch {
            dependency: unresolved.recipe.clone(),
            found: unresolved.arguments.len(),
            min: resolved.min_arguments(),
            max: resolved.max_arguments(settings),
          },
        ));
      }
    }

    let dependencies = self
      .dependencies
      .into_iter()
      .zip(resolved)
      .map(|(unresolved, resolved)| Dependency {
        arguments: resolved.group_arguments(&unresolved.arguments, settings),
        recipe: resolved,
      })
      .collect();

    let mut recipe_path = modulepath.clone();

    recipe_path.components.push(self.name.lexeme().into());

    Ok(Recipe {
      attributes,
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
