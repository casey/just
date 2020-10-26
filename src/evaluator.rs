use crate::common::*;

pub(crate) struct Evaluator<'src: 'run, 'run> {
  assignments: Option<&'run Table<'src, Assignment<'src>>>,
  config:      &'run Config,
  dotenv:      &'run BTreeMap<String, String>,
  scope:       Scope<'src, 'run>,
  settings:    &'run Settings<'run>,
  search:      &'run Search,
}

impl<'src, 'run> Evaluator<'src, 'run> {
  pub(crate) fn evaluate_assignments(
    assignments: &'run Table<'src, Assignment<'src>>,
    config: &'run Config,
    dotenv: &'run BTreeMap<String, String>,
    overrides: Scope<'src, 'run>,
    settings: &'run Settings<'run>,
    search: &'run Search,
  ) -> RunResult<'src, Scope<'src, 'run>> {
    let mut evaluator = Evaluator {
      scope: overrides,
      assignments: Some(assignments),
      config,
      dotenv,
      settings,
      search,
    };

    for assignment in assignments.values() {
      evaluator.evaluate_assignment(assignment)?;
    }

    Ok(evaluator.scope)
  }

  fn evaluate_assignment(&mut self, assignment: &Assignment<'src>) -> RunResult<'src, &str> {
    let name = assignment.name.lexeme();

    if !self.scope.bound(name) {
      let value = self.evaluate_expression(&assignment.value)?;
      self.scope.bind(assignment.export, assignment.name, value);
    }

    Ok(self.scope.value(name).unwrap())
  }

  pub(crate) fn evaluate_expression(
    &mut self,
    expression: &Expression<'src>,
  ) -> RunResult<'src, String> {
    match expression {
      Expression::Variable { name, .. } => {
        let variable = name.lexeme();
        if let Some(value) = self.scope.value(variable) {
          Ok(value.to_owned())
        } else if let Some(assignment) = self
          .assignments
          .and_then(|assignments| assignments.get(variable))
        {
          Ok(self.evaluate_assignment(assignment)?.to_owned())
        } else {
          Err(RuntimeError::Internal {
            message: format!("attempted to evaluate undefined variable `{}`", variable),
          })
        }
      },
      Expression::Call { thunk } => {
        use Thunk::*;

        let context = FunctionContext {
          dotenv:               self.dotenv,
          invocation_directory: &self.config.invocation_directory,
          search:               self.search,
        };

        match thunk {
          Nullary { name, function, .. } =>
            function(&context).map_err(|message| RuntimeError::FunctionCall {
              function: *name,
              message,
            }),
          Unary {
            name,
            function,
            arg,
            ..
          } => function(&context, &self.evaluate_expression(arg)?).map_err(|message| {
            RuntimeError::FunctionCall {
              function: *name,
              message,
            }
          }),
          Binary {
            name,
            function,
            args: [a, b],
            ..
          } => function(
            &context,
            &self.evaluate_expression(a)?,
            &self.evaluate_expression(b)?,
          )
          .map_err(|message| RuntimeError::FunctionCall {
            function: *name,
            message,
          }),
        }
      },
      Expression::StringLiteral { string_literal } => Ok(string_literal.cooked.to_string()),
      Expression::Backtick { contents, token } =>
        if self.config.dry_run {
          Ok(format!("`{}`", contents))
        } else {
          Ok(self.run_backtick(contents, token)?)
        },
      Expression::Concatination { lhs, rhs } =>
        Ok(self.evaluate_expression(lhs)? + &self.evaluate_expression(rhs)?),
      Expression::Conditional {
        lhs,
        rhs,
        then,
        otherwise,
        inverted,
      } => {
        // TODO: test that branch not taken isn't evaluated
        // i.e. back ticks aren't run
        let lhs = self.evaluate_expression(lhs)?;
        let rhs = self.evaluate_expression(rhs)?;
        let condition = if *inverted { lhs != rhs } else { lhs == rhs };
        if condition {
          self.evaluate_expression(then)
        } else {
          self.evaluate_expression(otherwise)
        }
      },
      Expression::Group { contents } => self.evaluate_expression(contents),
    }
  }

  fn run_backtick(&self, raw: &str, token: &Token<'src>) -> RunResult<'src, String> {
    let mut cmd = self.settings.shell_command(self.config);

    cmd.arg(raw);

    cmd.current_dir(&self.search.working_directory);

    cmd.export(self.dotenv, &self.scope);

    cmd.stdin(process::Stdio::inherit());

    cmd.stderr(if self.config.verbosity.quiet() {
      process::Stdio::null()
    } else {
      process::Stdio::inherit()
    });

    InterruptHandler::guard(|| {
      output(cmd).map_err(|output_error| RuntimeError::Backtick {
        token: *token,
        output_error,
      })
    })
  }

  pub(crate) fn evaluate_line(
    &mut self,
    line: &Line<'src>,
    continued: bool,
  ) -> RunResult<'src, String> {
    let mut evaluated = String::new();
    for (i, fragment) in line.fragments.iter().enumerate() {
      match fragment {
        Fragment::Text { token } =>
          if i == 0 && continued {
            evaluated += token.lexeme().trim_start();
          } else {
            evaluated += token.lexeme();
          },
        Fragment::Interpolation { expression } => {
          evaluated += &self.evaluate_expression(expression)?;
        },
      }
    }
    Ok(evaluated)
  }

  pub(crate) fn evaluate_parameters(
    config: &'run Config,
    dotenv: &'run BTreeMap<String, String>,
    parameters: &[Parameter<'src>],
    arguments: &[&str],
    scope: &'run Scope<'src, 'run>,
    settings: &'run Settings,
    search: &'run Search,
  ) -> RunResult<'src, Scope<'src, 'run>> {
    let mut evaluator = Evaluator {
      assignments: None,
      scope: Scope::child(scope),
      search,
      settings,
      dotenv,
      config,
    };

    let mut scope = Scope::child(scope);

    let mut rest = arguments;
    for parameter in parameters {
      let value = if rest.is_empty() {
        if let Some(ref default) = parameter.default {
          evaluator.evaluate_expression(default)?
        } else if parameter.kind == ParameterKind::Star {
          String::new()
        } else {
          return Err(RuntimeError::Internal {
            message: "missing parameter without default".to_string(),
          });
        }
      } else if parameter.kind.is_variadic() {
        let value = rest.to_vec().join(" ");
        rest = &[];
        value
      } else {
        let value = rest[0].to_owned();
        rest = &rest[1..];
        value
      };
      scope.bind(false, parameter.name, value);
    }

    Ok(scope)
  }

  pub(crate) fn recipe_evaluator(
    config: &'run Config,
    dotenv: &'run BTreeMap<String, String>,
    scope: &'run Scope<'src, 'run>,
    settings: &'run Settings,
    search: &'run Search,
  ) -> Evaluator<'src, 'run> {
    Evaluator {
      assignments: None,
      scope: Scope::child(scope),
      search,
      settings,
      dotenv,
      config,
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
    error: RuntimeError::Backtick {
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
      export exported_variable = "A"
      b = `echo $exported_variable`

      recipe:
        echo {{b}}
    "#,
    args: ["--quiet", "recipe"],
    error: RuntimeError::Backtick {
        token,
        output_error: OutputError::Code(_),
    },
    check: {
      assert_eq!(token.lexeme(), "`echo $exported_variable`");
    }
  }
}
