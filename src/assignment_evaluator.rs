use common::*;

use brev;
use DEFAULT_SHELL;

pub fn evaluate_assignments<'a>(
  assignments: &Map<&'a str, Expression<'a>>,
  overrides:   &Map<&str, &str>,
  quiet:       bool,
) -> Result<Map<&'a str, String>, RuntimeError<'a>> {
  let mut evaluator = Evaluator {
    assignments: assignments,
    evaluated:   empty(),
    exports:     &empty(),
    overrides:   overrides,
    quiet:       quiet,
    scope:       &empty(),
  };

  for name in assignments.keys() {
    evaluator.evaluate_assignment(name)?;
  }

  Ok(evaluator.evaluated)
}

fn run_backtick<'a>(
  raw:     &str,
  token:   &Token<'a>,
  scope:   &Map<&'a str, String>,
  exports: &Set<&'a str>,
  quiet:   bool,
) -> Result<String, RuntimeError<'a>> {
  let mut cmd = process::Command::new(DEFAULT_SHELL);

  cmd.export_environment_variables(scope, exports)?;

  cmd.arg("-cu")
     .arg(raw);

  cmd.stderr(if quiet {
    process::Stdio::null()
  } else {
    process::Stdio::inherit()
  });

  brev::output(cmd).map_err(|output_error| RuntimeError::Backtick{token: token.clone(), output_error})
}

pub struct Evaluator<'a: 'b, 'b> {
  pub assignments: &'b Map<&'a str, Expression<'a>>,
  pub evaluated:   Map<&'a str, String>,
  pub exports:     &'b Set<&'a str>,
  pub overrides:   &'b Map<&'b str, &'b str>,
  pub quiet:       bool,
  pub scope:       &'b Map<&'a str, String>,
}

impl<'a, 'b> Evaluator<'a, 'b> {
  pub fn evaluate_line(
    &mut self,
    line:      &[Fragment<'a>],
    arguments: &Map<&str, Cow<str>>
  ) -> Result<String, RuntimeError<'a>> {
    let mut evaluated = String::new();
    for fragment in line {
      match *fragment {
        Fragment::Text{ref text} => evaluated += text.lexeme,
        Fragment::Expression{ref expression} => {
          evaluated += &self.evaluate_expression(expression, arguments)?;
        }
      }
    }
    Ok(evaluated)
  }

  fn evaluate_assignment(&mut self, name: &'a str) -> Result<(), RuntimeError<'a>> {
    if self.evaluated.contains_key(name) {
      return Ok(());
    }

    if let Some(expression) = self.assignments.get(name) {
      if let Some(value) = self.overrides.get(name) {
        self.evaluated.insert(name, value.to_string());
      } else {
        let value = self.evaluate_expression(expression, &empty())?;
        self.evaluated.insert(name, value);
      }
    } else {
      return Err(RuntimeError::InternalError {
        message: format!("attempted to evaluated unknown assignment {}", name)
      });
    }

    Ok(())
  }

  fn evaluate_expression(
    &mut self,
    expression: &Expression<'a>,
    arguments: &Map<&str, Cow<str>>
  ) -> Result<String, RuntimeError<'a>> {
    Ok(match *expression {
      Expression::Variable{name, ..} => {
        if self.evaluated.contains_key(name) {
          self.evaluated[name].clone()
        } else if self.scope.contains_key(name) {
          self.scope[name].clone()
        } else if self.assignments.contains_key(name) {
          self.evaluate_assignment(name)?;
          self.evaluated[name].clone()
        } else if arguments.contains_key(name) {
          arguments[name].to_string()
        } else {
          return Err(RuntimeError::InternalError {
            message: format!("attempted to evaluate undefined variable `{}`", name)
          });
        }
      }
      Expression::String{ref cooked_string} => cooked_string.cooked.clone(),
      Expression::Backtick{raw, ref token} => {
        run_backtick(raw, token, self.scope, self.exports, self.quiet)?
      }
      Expression::Concatination{ref lhs, ref rhs} => {
        self.evaluate_expression(lhs, arguments)?
          +
        &self.evaluate_expression(rhs, arguments)?
      }
    })
  }
}


#[cfg(test)]
mod test {
  use super::*;
  use brev::OutputError;
  use testing::parse_success;
  use Configuration;

#[test]
fn backtick_code() {
  match parse_success("a:\n echo {{`f() { return 100; }; f`}}")
        .run(&["a"], &Default::default()).unwrap_err() {
    RuntimeError::Backtick{token, output_error: OutputError::Code(code)} => {
      assert_eq!(code, 100);
      assert_eq!(token.lexeme, "`f() { return 100; }; f`");
    },
    other => panic!("expected a code run error, but got: {}", other),
  }
}

#[test]
fn export_assignment_backtick() {
  let text = r#"
export exported_variable = "A"
b = `echo $exported_variable`

recipe:
  echo {{b}}
"#;

  let options = Configuration {
    quiet: true,
    ..Default::default()
  };

  match parse_success(text).run(&["recipe"], &options).unwrap_err() {
    RuntimeError::Backtick{token, output_error: OutputError::Code(_)} => {
      assert_eq!(token.lexeme, "`echo $exported_variable`");
    },
    other => panic!("expected a backtick code errror, but got: {}", other),
  }
}

}
