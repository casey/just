use crate::common::*;

pub(crate) struct AssignmentEvaluator<'a: 'b, 'b> {
  pub(crate) assignments: &'b Table<'a, Assignment<'a>>,
  pub(crate) config: &'a Config,
  pub(crate) dotenv: &'b BTreeMap<String, String>,
  pub(crate) evaluated: BTreeMap<&'a str, (bool, String)>,
  pub(crate) scope: &'b BTreeMap<&'a str, (bool, String)>,
  pub(crate) working_directory: &'b Path,
  pub(crate) overrides: &'b BTreeMap<String, String>,
  pub(crate) settings: &'b Settings<'b>,
}

impl<'a, 'b> AssignmentEvaluator<'a, 'b> {
  pub(crate) fn evaluate_assignments(
    config: &'a Config,
    working_directory: &'b Path,
    dotenv: &'b BTreeMap<String, String>,
    assignments: &Table<'a, Assignment<'a>>,
    overrides: &BTreeMap<String, String>,
    settings: &'b Settings<'b>,
  ) -> RunResult<'a, BTreeMap<&'a str, (bool, String)>> {
    let mut evaluator = AssignmentEvaluator {
      evaluated: empty(),
      scope: &empty(),
      settings,
      overrides,
      config,
      assignments,
      working_directory,
      dotenv,
    };

    for name in assignments.keys() {
      evaluator.evaluate_assignment(name)?;
    }

    Ok(evaluator.evaluated)
  }

  pub(crate) fn evaluate_line(
    &mut self,
    line: &[Fragment<'a>],
    arguments: &BTreeMap<&'a str, Cow<str>>,
  ) -> RunResult<'a, String> {
    let mut evaluated = String::new();
    for fragment in line {
      match fragment {
        Fragment::Text { token } => evaluated += token.lexeme(),
        Fragment::Interpolation { expression } => {
          evaluated += &self.evaluate_expression(expression, arguments)?;
        }
      }
    }
    Ok(evaluated)
  }

  fn evaluate_assignment(&mut self, name: &'a str) -> RunResult<'a, ()> {
    if self.evaluated.contains_key(name) {
      return Ok(());
    }

    if let Some(assignment) = self.assignments.get(name) {
      if let Some(value) = self.overrides.get(name) {
        self
          .evaluated
          .insert(name, (assignment.export, value.to_string()));
      } else {
        let value = self.evaluate_expression(&assignment.expression, &empty())?;
        self.evaluated.insert(name, (assignment.export, value));
      }
    } else {
      return Err(RuntimeError::Internal {
        message: format!("attempted to evaluated unknown assignment {}", name),
      });
    }

    Ok(())
  }

  pub(crate) fn evaluate_expression(
    &mut self,
    expression: &Expression<'a>,
    arguments: &BTreeMap<&'a str, Cow<str>>,
  ) -> RunResult<'a, String> {
    match expression {
      Expression::Variable { name, .. } => {
        let variable = name.lexeme();
        if self.evaluated.contains_key(variable) {
          Ok(self.evaluated[variable].1.clone())
        } else if self.scope.contains_key(variable) {
          Ok(self.scope[variable].1.clone())
        } else if self.assignments.contains_key(variable) {
          self.evaluate_assignment(variable)?;
          Ok(self.evaluated[variable].1.clone())
        } else if arguments.contains_key(variable) {
          Ok(arguments[variable].to_string())
        } else {
          Err(RuntimeError::Internal {
            message: format!("attempted to evaluate undefined variable `{}`", variable),
          })
        }
      }
      Expression::Call {
        function,
        arguments: call_arguments,
      } => {
        let call_arguments = call_arguments
          .iter()
          .map(|argument| self.evaluate_expression(argument, arguments))
          .collect::<Result<Vec<String>, RuntimeError>>()?;
        let context = FunctionContext {
          invocation_directory: &self.config.invocation_directory,
          working_directory: &self.working_directory,
          dotenv: self.dotenv,
        };
        Function::evaluate(*function, &context, &call_arguments)
      }
      Expression::StringLiteral { string_literal } => Ok(string_literal.cooked.to_string()),
      Expression::Backtick { contents, token } => {
        if self.config.dry_run {
          Ok(format!("`{}`", contents))
        } else {
          Ok(self.run_backtick(self.dotenv, contents, token)?)
        }
      }
      Expression::Concatination { lhs, rhs } => {
        Ok(self.evaluate_expression(lhs, arguments)? + &self.evaluate_expression(rhs, arguments)?)
      }
      Expression::Group { contents } => self.evaluate_expression(contents, arguments),
    }
  }

  fn run_backtick(
    &self,
    dotenv: &BTreeMap<String, String>,
    raw: &str,
    token: &Token<'a>,
  ) -> RunResult<'a, String> {
    let mut cmd = self.settings.shell_command(&self.config.shell);

    cmd.arg(raw);

    cmd.current_dir(self.working_directory);

    cmd.export_environment_variables(self.scope, dotenv)?;

    cmd.stdin(process::Stdio::inherit());

    cmd.stderr(if self.config.quiet {
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
