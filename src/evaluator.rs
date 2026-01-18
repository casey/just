use super::*;

pub(crate) struct Evaluator<'src: 'run, 'run> {
  assignments: Option<&'run Table<'src, Assignment<'src>>>,
  context: Option<ExecutionContext<'src, 'run>>,
  is_dependency: bool,
  non_const_assignments: Table<'src, Name<'src>>,
  scope: Scope<'src, 'run>,
}

impl<'src, 'run> Evaluator<'src, 'run> {
  fn context(
    &self,
    const_error: ConstError<'src>,
  ) -> Result<&ExecutionContext<'src, 'run>, ConstError<'src>> {
    self.context.as_ref().ok_or(const_error)
  }

  pub(crate) fn evaluate_settings(
    assignments: &'run Table<'src, Assignment<'src>>,
    config: &Config,
    name: Option<Name>,
    sets: Table<'src, Set<'src>>,
    scope: &'run Scope<'src, 'run>,
  ) -> RunResult<'src, Settings> {
    let mut scope = scope.child();

    if name.is_none() {
      let mut unknown_overrides = Vec::new();

      for (name, value) in &config.overrides {
        if let Some(assignment) = assignments.get(name) {
          scope.bind(Binding {
            export: assignment.export,
            file_depth: 0,
            name: assignment.name,
            prelude: false,
            private: assignment.private,
            value: value.clone(),
          });
        } else {
          unknown_overrides.push(name.clone());
        }
      }

      if !unknown_overrides.is_empty() {
        return Err(Error::UnknownOverrides {
          overrides: unknown_overrides,
        });
      }
    }

    let mut evaluator = Self {
      assignments: Some(assignments),
      context: None,
      is_dependency: false,
      non_const_assignments: Table::new(),
      scope,
    };

    for assignment in assignments.values() {
      match evaluator.evaluate_assignment(assignment) {
        Err(Error::Const { .. }) => evaluator.non_const_assignments.insert(assignment.name),
        Err(err) => return Err(err),
        Ok(_) => {}
      }
    }

    evaluator.evaluate_sets(sets)
  }

  fn evaluate_sets(&mut self, sets: Table<'src, Set<'src>>) -> RunResult<'src, Settings> {
    let mut settings = Settings::default();

    for (_name, set) in sets {
      match set.value {
        Setting::AllowDuplicateRecipes(value) => {
          settings.allow_duplicate_recipes = value;
        }
        Setting::AllowDuplicateVariables(value) => {
          settings.allow_duplicate_variables = value;
        }
        Setting::DotenvFilename(value) => {
          settings.dotenv_filename = Some(self.evaluate_expression(&value)?);
        }
        Setting::DotenvLoad(value) => {
          settings.dotenv_load = value;
        }
        Setting::DotenvPath(value) => {
          settings.dotenv_path = Some(self.evaluate_expression(&value)?.into());
        }
        Setting::DotenvOverride(value) => {
          settings.dotenv_override = value;
        }
        Setting::DotenvRequired(value) => {
          settings.dotenv_required = value;
        }
        Setting::Export(value) => {
          settings.export = value;
        }
        Setting::Fallback(value) => {
          settings.fallback = value;
        }
        Setting::IgnoreComments(value) => {
          settings.ignore_comments = value;
        }
        Setting::NoExitMessage(value) => {
          settings.no_exit_message = value;
        }
        Setting::PositionalArguments(value) => {
          settings.positional_arguments = value;
        }
        Setting::Quiet(value) => {
          settings.quiet = value;
        }
        Setting::ScriptInterpreter(value) => {
          settings.script_interpreter = Some(self.evaluate_interpreter(&value)?);
        }
        Setting::Shell(value) => {
          settings.shell = Some(self.evaluate_interpreter(&value)?);
        }
        Setting::Unstable(value) => {
          settings.unstable = value;
        }
        Setting::WindowsPowerShell(value) => {
          settings.windows_powershell = value;
        }
        Setting::WindowsShell(value) => {
          settings.windows_shell = Some(self.evaluate_interpreter(&value)?);
        }
        Setting::Tempdir(value) => {
          settings.tempdir = Some(self.evaluate_expression(&value)?);
        }
        Setting::WorkingDirectory(value) => {
          settings.working_directory = Some(self.evaluate_expression(&value)?.into());
        }
      }
    }

    Ok(settings)
  }

  pub(crate) fn evaluate_interpreter(
    &mut self,
    interpreter: &Interpreter<Expression<'src>>,
  ) -> RunResult<'src, Interpreter<String>> {
    Ok(Interpreter {
      command: self.evaluate_expression(&interpreter.command)?,
      arguments: interpreter
        .arguments
        .iter()
        .map(|argument| self.evaluate_expression(argument))
        .collect::<RunResult<Vec<String>>>()?,
    })
  }

  pub(crate) fn evaluate_assignments(
    config: &'run Config,
    dotenv: &'run BTreeMap<String, String>,
    module: &'run Justfile<'src>,
    parent: &'run Scope<'src, 'run>,
    search: &'run Search,
  ) -> RunResult<'src, Scope<'src, 'run>>
  where
    'src: 'run,
  {
    let context = ExecutionContext {
      config,
      dotenv,
      module,
      search,
    };

    let mut scope = parent.child();

    if !module.is_submodule() {
      let mut unknown_overrides = Vec::new();

      for (name, value) in &config.overrides {
        if let Some(assignment) = module.assignments.get(name) {
          scope.bind(Binding {
            export: assignment.export,
            file_depth: 0,
            name: assignment.name,
            prelude: false,
            private: assignment.private,
            value: value.clone(),
          });
        } else {
          unknown_overrides.push(name.clone());
        }
      }

      if !unknown_overrides.is_empty() {
        return Err(Error::UnknownOverrides {
          overrides: unknown_overrides,
        });
      }
    }

    let mut evaluator = Self {
      assignments: Some(&module.assignments),
      context: Some(context),
      is_dependency: false,
      non_const_assignments: Table::new(),
      scope,
    };

    for assignment in module.assignments.values() {
      evaluator.evaluate_assignment(assignment)?;
    }

    Ok(evaluator.scope)
  }

  fn evaluate_assignment(&mut self, assignment: &Assignment<'src>) -> RunResult<'src, &str> {
    let name = assignment.name.lexeme();

    if !self.scope.bound(name) {
      let value = self.evaluate_expression(&assignment.value)?;
      self.scope.bind(Binding {
        export: assignment.export,
        file_depth: 0,
        name: assignment.name,
        prelude: false,
        private: assignment.private,
        value,
      });
    }

    Ok(self.scope.value(name).unwrap())
  }

  fn function_context(&self, thunk: &Thunk<'src>) -> RunResult<'src, function::Context> {
    Ok(function::Context {
      execution_context: self.context(ConstError::FunctionCall(thunk.name()))?,
      is_dependency: self.is_dependency,
      name: thunk.name(),
      scope: &self.scope,
    })
  }

  pub(crate) fn evaluate_expression(
    &mut self,
    expression: &Expression<'src>,
  ) -> RunResult<'src, String> {
    match expression {
      Expression::And { lhs, rhs } => {
        let lhs = self.evaluate_expression(lhs)?;
        if lhs.is_empty() {
          return Ok(String::new());
        }
        self.evaluate_expression(rhs)
      }
      Expression::Assert {
        condition,
        error,
        name,
      } => {
        if self.evaluate_condition(condition)? {
          Ok(String::new())
        } else {
          Err(Error::Assert {
            message: self.evaluate_expression(error)?,
            name: *name,
          })
        }
      }
      Expression::Backtick { contents, token } => {
        let context = self.context(ConstError::Backtick(*token))?;

        if context.config.dry_run {
          return Ok(format!("`{contents}`"));
        }

        Self::run_command(context, &self.scope, contents, &[]).map_err(|output_error| {
          Error::Backtick {
            token: *token,
            output_error,
          }
        })
      }
      Expression::Call { thunk } => {
        use Thunk::*;
        match thunk {
          Nullary { function, .. } => function(self.function_context(thunk)?),
          Unary { function, arg, .. } => {
            let arg = self.evaluate_expression(arg)?;
            function(self.function_context(thunk)?, &arg)
          }
          UnaryOpt {
            function,
            args: (a, b),
            ..
          } => {
            let a = self.evaluate_expression(a)?;
            let b = match b.as_ref() {
              Some(b) => Some(self.evaluate_expression(b)?),
              None => None,
            };
            function(self.function_context(thunk)?, &a, b.as_deref())
          }
          UnaryPlus {
            function,
            args: (a, rest),
            ..
          } => {
            let a = self.evaluate_expression(a)?;
            let mut rest_evaluated = Vec::new();
            for arg in rest {
              rest_evaluated.push(self.evaluate_expression(arg)?);
            }
            function(self.function_context(thunk)?, &a, &rest_evaluated)
          }
          Binary {
            function,
            args: [a, b],
            ..
          } => {
            let a = self.evaluate_expression(a)?;
            let b = self.evaluate_expression(b)?;
            function(self.function_context(thunk)?, &a, &b)
          }
          BinaryPlus {
            function,
            args: ([a, b], rest),
            ..
          } => {
            let a = self.evaluate_expression(a)?;
            let b = self.evaluate_expression(b)?;
            let mut rest_evaluated = Vec::new();
            for arg in rest {
              rest_evaluated.push(self.evaluate_expression(arg)?);
            }
            function(self.function_context(thunk)?, &a, &b, &rest_evaluated)
          }
          Ternary {
            function,
            args: [a, b, c],
            ..
          } => {
            let a = self.evaluate_expression(a)?;
            let b = self.evaluate_expression(b)?;
            let c = self.evaluate_expression(c)?;
            function(self.function_context(thunk)?, &a, &b, &c)
          }
        }
        .map_err(|message| Error::FunctionCall {
          function: thunk.name(),
          message,
        })
      }
      Expression::Concatenation { lhs, rhs } => {
        let lhs = self.evaluate_expression(lhs)?;
        let rhs = self.evaluate_expression(rhs)?;
        Ok(lhs + &rhs)
      }
      Expression::Conditional {
        condition,
        then,
        otherwise,
      } => {
        if self.evaluate_condition(condition)? {
          self.evaluate_expression(then)
        } else {
          self.evaluate_expression(otherwise)
        }
      }
      Expression::FormatString { start, expressions } => {
        let mut value = start.cooked.clone();

        for (expression, string) in expressions {
          value.push_str(&self.evaluate_expression(expression)?);
          value.push_str(&string.cooked);
        }

        if start.kind.indented {
          Ok(unindent(&value))
        } else {
          Ok(value)
        }
      }
      Expression::Group { contents } => self.evaluate_expression(contents),
      Expression::Join { lhs: None, rhs } => Ok("/".to_string() + &self.evaluate_expression(rhs)?),
      Expression::Join {
        lhs: Some(lhs),
        rhs,
      } => {
        let lhs = self.evaluate_expression(lhs)?;
        let rhs = self.evaluate_expression(rhs)?;
        Ok(lhs + "/" + &rhs)
      }
      Expression::Or { lhs, rhs } => {
        let lhs = self.evaluate_expression(lhs)?;
        if !lhs.is_empty() {
          return Ok(lhs);
        }
        self.evaluate_expression(rhs)
      }
      Expression::StringLiteral { string_literal } => Ok(string_literal.cooked.clone()),
      Expression::Variable { name, .. } => {
        let variable = name.lexeme();
        if let Some(value) = self.scope.value(variable) {
          Ok(value.to_owned())
        } else if self.non_const_assignments.contains_key(name.lexeme()) {
          Err(ConstError::Variable(*name).into())
        } else if let Some(assignment) = self
          .assignments
          .and_then(|assignments| assignments.get(variable))
        {
          Ok(self.evaluate_assignment(assignment)?.to_owned())
        } else {
          Err(Error::internal(format!(
            "attempted to evaluate undefined variable `{variable}`"
          )))
        }
      }
    }
  }

  fn evaluate_condition(&mut self, condition: &Condition<'src>) -> RunResult<'src, bool> {
    let lhs_value = self.evaluate_expression(&condition.lhs)?;
    let rhs_value = self.evaluate_expression(&condition.rhs)?;
    let condition = match condition.operator {
      ConditionalOperator::Equality => lhs_value == rhs_value,
      ConditionalOperator::Inequality => lhs_value != rhs_value,
      ConditionalOperator::RegexMatch => Regex::new(&rhs_value)
        .map_err(|source| Error::RegexCompile { source })?
        .is_match(&lhs_value),
      ConditionalOperator::RegexMismatch => !Regex::new(&rhs_value)
        .map_err(|source| Error::RegexCompile { source })?
        .is_match(&lhs_value),
    };
    Ok(condition)
  }

  pub(crate) fn run_command(
    context: &ExecutionContext,
    scope: &Scope,
    command: &str,
    args: &[&str],
  ) -> Result<String, OutputError> {
    let mut cmd = context.module.settings.shell_command(context.config);

    cmd
      .arg(command)
      .args(args)
      .current_dir(context.working_directory())
      .export(
        &context.module.settings,
        context.dotenv,
        scope,
        &context.module.unexports,
      )
      .stdin(Stdio::inherit())
      .stderr(if context.config.verbosity.quiet() {
        Stdio::null()
      } else {
        Stdio::inherit()
      })
      .stdout(Stdio::piped());

    cmd.output_guard_stdout()
  }

  pub(crate) fn evaluate_line(
    &mut self,
    line: &Line<'src>,
    continued: bool,
  ) -> RunResult<'src, String> {
    let mut evaluated = String::new();
    for (i, fragment) in line.fragments.iter().enumerate() {
      match fragment {
        Fragment::Text { token } => {
          let lexeme = token
            .lexeme()
            .replace(Lexer::INTERPOLATION_ESCAPE, Lexer::INTERPOLATION_START);

          if i == 0 && continued {
            evaluated += lexeme.trim_start();
          } else {
            evaluated += &lexeme;
          }
        }
        Fragment::Interpolation { expression } => {
          evaluated += &self.evaluate_expression(expression)?;
        }
      }
    }
    Ok(evaluated)
  }

  pub(crate) fn evaluate_parameters(
    arguments: &[Vec<String>],
    context: &ExecutionContext<'src, 'run>,
    is_dependency: bool,
    parameters: &[Parameter<'src>],
    recipe: &Recipe<'src>,
    scope: &'run Scope<'src, 'run>,
  ) -> RunResult<'src, (Scope<'src, 'run>, Vec<String>)> {
    let mut evaluator = Self::new(context, is_dependency, scope);

    let mut positional = Vec::new();

    if arguments.len() != parameters.len() {
      return Err(Error::internal("arguments do not match parameter count"));
    }

    for (parameter, group) in parameters.iter().zip(arguments) {
      let values = if group.is_empty() {
        if let Some(ref default) = parameter.default {
          let value = evaluator.evaluate_expression(default)?;
          positional.push(value.clone());
          vec![value]
        } else if parameter.kind == ParameterKind::Star {
          Vec::new()
        } else {
          return Err(Error::internal("missing parameter without default"));
        }
      } else if parameter.kind.is_variadic() {
        positional.extend_from_slice(group);
        group.clone()
      } else {
        if group.len() != 1 {
          return Err(Error::internal(
            "multiple values for non-variadic parameter",
          ));
        }
        let value = group[0].clone();
        positional.push(value.clone());
        vec![value]
      };

      for value in &values {
        parameter.check_pattern_match(recipe, value)?;
      }

      evaluator.scope.bind(Binding {
        export: parameter.export,
        file_depth: 0,
        name: parameter.name,
        prelude: false,
        private: false,
        value: values.join(" "),
      });
    }

    Ok((evaluator.scope, positional))
  }

  pub(crate) fn new(
    context: &ExecutionContext<'src, 'run>,
    is_dependency: bool,
    scope: &'run Scope<'src, 'run>,
  ) -> Self {
    Self {
      assignments: None,
      context: Some(*context),
      is_dependency,
      non_const_assignments: Table::new(),
      scope: scope.child(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  run_error! {
    name: backtick_code,
    src: "
      a:
       echo {{`f() { return 100; }; f`}}
    ",
    args: ["a"],
    error: Error::Backtick {
      token,
      output_error: OutputError::Code(code),
    },
    check: {
      assert_eq!(code, 100);
      assert_eq!(token.lexeme(), "`f() { return 100; }; f`");
    }
  }

  run_error! {
    name: export_assignment_backtick,
    src: r#"
      export exported_variable := "A"
      b := `echo $exported_variable`

      recipe:
        echo {{b}}
    "#,
    args: ["--quiet", "recipe"],
    error: Error::Backtick {
        token,
        output_error: OutputError::Code(_),
    },
    check: {
      assert_eq!(token.lexeme(), "`echo $exported_variable`");
    }
  }
}
