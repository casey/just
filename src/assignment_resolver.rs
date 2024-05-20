use {super::*, CompileErrorKind::*};

pub(crate) struct AssignmentResolver<'src: 'run, 'run> {
  assignments: &'run Table<'src, Assignment<'src>>,
  stack: Vec<&'src str>,
  evaluated: BTreeSet<&'src str>,
}

impl<'src: 'run, 'run> AssignmentResolver<'src, 'run> {
  pub(crate) fn resolve_assignments(
    assignments: &'run Table<'src, Assignment<'src>>,
  ) -> CompileResult<'src> {
    let mut resolver = Self {
      stack: Vec::new(),
      evaluated: BTreeSet::new(),
      assignments,
    };

    for name in assignments.keys() {
      resolver.resolve_assignment(name)?;
    }

    Ok(())
  }

  fn resolve_assignment(&mut self, name: &'src str) -> CompileResult<'src> {
    if self.evaluated.contains(name) {
      return Ok(());
    }

    self.stack.push(name);

    if let Some(assignment) = self.assignments.get(name) {
      self.resolve_expression(&assignment.value)?;
      self.evaluated.insert(name);
    } else {
      let message = format!("attempted to resolve unknown assignment `{name}`");
      let token = Token {
        src: "",
        offset: 0,
        line: 0,
        column: 0,
        length: 0,
        kind: TokenKind::Unspecified,
        path: "".as_ref(),
      };
      return Err(CompileError::new(token, Internal { message }));
    }

    self.stack.pop();

    Ok(())
  }

  fn resolve_expression(&mut self, expression: &Expression<'src>) -> CompileResult<'src> {
    match expression {
      Expression::Assert {
        condition: Condition {
          lhs,
          rhs,
          operator: _,
        },
        error,
      } => {
        self.resolve_expression(lhs)?;
        self.resolve_expression(rhs)?;
        self.resolve_expression(error)
      }
      Expression::Call { thunk } => match thunk {
        Thunk::Nullary { .. } => Ok(()),
        Thunk::Unary { arg, .. } => self.resolve_expression(arg),
        Thunk::UnaryOpt { args: (a, b), .. } => {
          self.resolve_expression(a)?;
          if let Some(b) = b.as_ref() {
            self.resolve_expression(b)?;
          }
          Ok(())
        }
        Thunk::UnaryPlus {
          args: (a, rest), ..
        } => {
          self.resolve_expression(a)?;
          for arg in rest {
            self.resolve_expression(arg)?;
          }
          Ok(())
        }
        Thunk::Binary { args: [a, b], .. } => {
          self.resolve_expression(a)?;
          self.resolve_expression(b)
        }
        Thunk::BinaryPlus {
          args: ([a, b], rest),
          ..
        } => {
          self.resolve_expression(a)?;
          self.resolve_expression(b)?;
          for arg in rest {
            self.resolve_expression(arg)?;
          }
          Ok(())
        }
        Thunk::Ternary {
          args: [a, b, c], ..
        } => {
          self.resolve_expression(a)?;
          self.resolve_expression(b)?;
          self.resolve_expression(c)
        }
      },
      Expression::Concatenation { lhs, rhs } => {
        self.resolve_expression(lhs)?;
        self.resolve_expression(rhs)
      }
      Expression::Conditional {
        condition: Condition {
          lhs,
          rhs,
          operator: _,
        },
        then,
        otherwise,
        ..
      } => {
        self.resolve_expression(lhs)?;
        self.resolve_expression(rhs)?;
        self.resolve_expression(then)?;
        self.resolve_expression(otherwise)
      }
      Expression::Group { contents } => self.resolve_expression(contents),
      Expression::Join { lhs, rhs } => {
        if let Some(lhs) = lhs {
          self.resolve_expression(lhs)?;
        }
        self.resolve_expression(rhs)
      }
      Expression::StringLiteral { .. } | Expression::Backtick { .. } => Ok(()),
      Expression::Variable { name } => {
        let variable = name.lexeme();
        if self.evaluated.contains(variable) || constants().contains_key(variable) {
          Ok(())
        } else if self.stack.contains(&variable) {
          self.stack.push(variable);
          Err(
            self.assignments[variable]
              .name
              .error(CircularVariableDependency {
                variable,
                circle: self.stack.clone(),
              }),
          )
        } else if self.assignments.contains_key(variable) {
          self.resolve_assignment(variable)
        } else {
          Err(name.token.error(UndefinedVariable { variable }))
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  analysis_error! {
    name:   circular_variable_dependency,
    input:   "a := b\nb := a",
    offset:  0,
    line:   0,
    column: 0,
    width:  1,
    kind:   CircularVariableDependency{variable: "a", circle: vec!["a", "b", "a"]},
  }

  analysis_error! {
    name:   self_variable_dependency,
    input:  "a := a",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   CircularVariableDependency{variable: "a", circle: vec!["a", "a"]},
  }

  analysis_error! {
    name:   unknown_expression_variable,
    input:  "x := yy",
    offset: 5,
    line:   0,
    column: 5,
    width:  2,
    kind:   UndefinedVariable{variable: "yy"},
  }

  analysis_error! {
    name:   unknown_function_parameter,
    input:  "x := env_var(yy)",
    offset:  13,
    line:   0,
    column: 13,
    width:  2,
    kind:   UndefinedVariable{variable: "yy"},
  }

  analysis_error! {
    name:   unknown_function_parameter_binary_first,
    input:  "x := env_var_or_default(yy, 'foo')",
    offset:  24,
    line:   0,
    column: 24,
    width:  2,
    kind:   UndefinedVariable{variable: "yy"},
  }

  analysis_error! {
    name:   unknown_function_parameter_binary_second,
    input:  "x := env_var_or_default('foo', yy)",
    offset:  31,
    line:   0,
    column: 31,
    width:  2,
    kind:   UndefinedVariable{variable: "yy"},
  }
}
