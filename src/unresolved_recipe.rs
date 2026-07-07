use super::*;

pub(crate) type UnresolvedRecipe<'src> = Recipe<'src, UnresolvedDependency<'src>>;

impl<'src> UnresolvedRecipe<'src> {
  pub(crate) fn resolve(
    mut self,
    evaluator: &mut Evaluator<'src, '_>,
    modulepath: &Modulepath,
    resolved: Vec<Arc<Recipe<'src>>>,
    settings: &Settings,
    variable_resolver: &mut VariableResolver<'src, '_>,
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
      let context = ExpressionContext::from(&self.parameters[..i]);
      if let Some(expression) = &parameter.default {
        variable_resolver.resolve_expression(expression, &context, &mut variable_references)?;
      }
      if let Some(expression) = &parameter.value {
        variable_resolver.resolve_expression(expression, &context, &mut variable_references)?;
      }
    }

    let parameters = ExpressionContext::from(self.parameters.as_slice());
    let empty = ExpressionContext::new();

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
        variable_resolver.resolve_expression(
          &argument.expression,
          &parameters,
          &mut variable_references,
        )?;
      }
    }

    for attribute in &self.attributes {
      let mut resolve_expression = |expression, context| {
        variable_resolver.resolve_expression(expression, context, &mut variable_references)
      };

      match attribute {
        Attribute::Arg {
          help_property,
          pattern_property,
          ..
        } => {
          if let Some((_key, expression)) = help_property {
            resolve_expression(expression, &empty)?;
          }
          if let Some((_key, expression)) = pattern_property {
            resolve_expression(expression, &empty)?;
          }
        }
        Attribute::Cache {
          extra,
          inputs,
          outputs,
        } => {
          if let Some(extra) = extra {
            resolve_expression(extra, &parameters)?;
          }
          if let Some(inputs) = inputs {
            resolve_expression(inputs, &parameters)?;
          }
          if let Some(outputs) = outputs {
            resolve_expression(outputs, &parameters)?;
          }
        }
        Attribute::Confirm(Some(expression)) | Attribute::WorkingDirectory(expression) => {
          resolve_expression(expression, &parameters)?;
        }
        Attribute::Doc(Some(expression)) => {
          resolve_expression(expression, &empty)?;
        }
        Attribute::Env(key, value) => {
          resolve_expression(key, &empty)?;
          resolve_expression(value, &empty)?;
        }
        Attribute::Android
        | Attribute::Confirm(None)
        | Attribute::Continue(_)
        | Attribute::Default
        | Attribute::Doc(None)
        | Attribute::Dragonfly
        | Attribute::ExitMessage
        | Attribute::Extension(_)
        | Attribute::Freebsd
        | Attribute::Group(_)
        | Attribute::Linux
        | Attribute::Macos
        | Attribute::Metadata(_)
        | Attribute::Netbsd
        | Attribute::NoCd
        | Attribute::NoExitMessage
        | Attribute::NoQuiet
        | Attribute::Openbsd
        | Attribute::Parallel
        | Attribute::PositionalArguments
        | Attribute::Private
        | Attribute::Script(_)
        | Attribute::Shell
        | Attribute::Unix
        | Attribute::Windows => {}
      }
    }

    let script = self.is_script(settings);

    let attributes = self
      .attributes
      .into_items()
      .map(|(mut attribute, name)| {
        if let Attribute::Doc(Some(expression)) = &attribute {
          let value = evaluator.evaluate_value_const(expression)?;
          self.doc = if value.is_empty() {
            None
          } else {
            Some(value.join())
          };
        }

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
        Ok((attribute, name))
      })
      .collect::<CompileResult<AttributeSet>>()?;

    let mut continued = false;

    for line in &self.body {
      if !script && !continued && line.is_comment() && settings.ignore_comments {
        continue;
      }

      for fragment in &line.fragments {
        if let Fragment::Interpolation { expression, .. } = fragment {
          variable_resolver.resolve_expression(
            expression,
            &parameters,
            &mut variable_references,
          )?;
        }
      }

      continued = line.is_continuation();
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
        path: unresolved.recipe,
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
      number: self.number,
      parameters: self.parameters,
      priors: self.priors,
      private: self.private,
      quiet: self.quiet,
      recipe_path: Some(recipe_path),
      shebang: self.shebang,
      variable_references,
    })
  }
}
