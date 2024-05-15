use super::*;

pub(crate) struct Evaluator<'src: 'run, 'run> {
  assignments: Option<&'run Table<'src, Assignment<'src>>>,
  config: &'run Config,
  dotenv: &'run BTreeMap<String, String>,
  scope: Scope<'src, 'run>,
  settings: &'run Settings<'run>,
  search: &'run Search,
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
    let mut evaluator = Self {
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
          Err(Error::Internal {
            message: format!("attempted to evaluate undefined variable `{variable}`"),
          })
        }
      }
      Expression::Call { thunk } => {
        use Thunk::*;

        let context = FunctionContext {
          dotenv: self.dotenv,
          invocation_directory: &self.config.invocation_directory,
          search: self.search,
        };

        match thunk {
          Nullary { name, function, .. } => {
            function(&context).map_err(|message| Error::FunctionCall {
              function: *name,
              message,
            })
          }
          Unary {
            name,
            function,
            arg,
            ..
          } => function(&context, &self.evaluate_expression(arg)?).map_err(|message| {
            Error::FunctionCall {
              function: *name,
              message,
            }
          }),
          UnaryOpt {
            name,
            function,
            args: (a, b),
            ..
          } => {
            let a = self.evaluate_expression(a)?;
            let b = match b.as_ref() {
              Some(b) => Some(self.evaluate_expression(b)?),
              None => None,
            };

            function(&context, &a, b.as_deref()).map_err(|message| Error::FunctionCall {
              function: *name,
              message,
            })
          }
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
          .map_err(|message| Error::FunctionCall {
            function: *name,
            message,
          }),
          BinaryPlus {
            name,
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

            function(&context, &a, &b, &rest_evaluated).map_err(|message| Error::FunctionCall {
              function: *name,
              message,
            })
          }
          Ternary {
            name,
            function,
            args: [a, b, c],
            ..
          } => function(
            &context,
            &self.evaluate_expression(a)?,
            &self.evaluate_expression(b)?,
            &self.evaluate_expression(c)?,
          )
          .map_err(|message| Error::FunctionCall {
            function: *name,
            message,
          }),
        }
      }
      Expression::StringLiteral { string_literal } => Ok(string_literal.cooked.clone()),
      Expression::Backtick { contents, token } => {
        if self.config.dry_run {
          Ok(format!("`{contents}`"))
        } else {
          Ok(self.run_backtick(contents, token)?)
        }
      }
      Expression::Concatenation { lhs, rhs } => {
        Ok(self.evaluate_expression(lhs)? + &self.evaluate_expression(rhs)?)
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
      Expression::Group { contents } => self.evaluate_expression(contents),
      Expression::Join { lhs: None, rhs } => Ok("/".to_string() + &self.evaluate_expression(rhs)?),
      Expression::Join {
        lhs: Some(lhs),
        rhs,
      } => Ok(self.evaluate_expression(lhs)? + "/" + &self.evaluate_expression(rhs)?),
      Expression::Assert { condition, error } => {
        if self.evaluate_condition(condition)? {
          Ok(String::new())
        } else {
          Err(Error::Assert {
            message: self.evaluate_expression(error)?,
          })
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
    };
    Ok(condition)
  }

  fn run_backtick(&self, raw: &str, token: &Token<'src>) -> RunResult<'src, String> {
    let mut cmd = self.settings.shell_command(self.config);

    cmd.arg(raw);

    cmd.current_dir(&self.search.working_directory);

    cmd.export(self.settings, self.dotenv, &self.scope);

    cmd.stdin(Stdio::inherit());

    cmd.stderr(if self.config.verbosity.quiet() {
      Stdio::null()
    } else {
      Stdio::inherit()
    });

    InterruptHandler::guard(|| {
      output(cmd).map_err(|output_error| Error::Backtick {
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
        Fragment::Text { token } => {
          let lexeme = token.lexeme().replace("{{{{", "{{");

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
    config: &'run Config,
    dotenv: &'run BTreeMap<String, String>,
    parameters: &[Parameter<'src>],
    arguments: &[String],
    scope: &'run Scope<'src, 'run>,
    settings: &'run Settings,
    search: &'run Search,
  ) -> RunResult<'src, (Scope<'src, 'run>, Vec<String>)> {
    let mut evaluator = Self {
      assignments: None,
      scope: scope.child(),
      search,
      settings,
      dotenv,
      config,
    };

    let mut scope = scope.child();

    let mut positional = Vec::new();

    let mut rest = arguments;
    for parameter in parameters {
      let value = if rest.is_empty() {
        if let Some(ref default) = parameter.default {
          let value = evaluator.evaluate_expression(default)?;
          positional.push(value.clone());
          value
        } else if parameter.kind == ParameterKind::Star {
          String::new()
        } else {
          return Err(Error::Internal {
            message: "missing parameter without default".to_owned(),
          });
        }
      } else if parameter.kind.is_variadic() {
        for value in rest {
          positional.push(value.clone());
        }
        let value = rest.to_vec().join(" ");
        rest = &[];
        value
      } else {
        let value = rest[0].clone();
        positional.push(value.clone());
        rest = &rest[1..];
        value
      };
      scope.bind(parameter.export, parameter.name, value);
    }

    Ok((scope, positional))
  }

  pub(crate) fn recipe_evaluator(
    config: &'run Config,
    dotenv: &'run BTreeMap<String, String>,
    scope: &'run Scope<'src, 'run>,
    settings: &'run Settings,
    search: &'run Search,
  ) -> Evaluator<'src, 'run> {
    Self {
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
