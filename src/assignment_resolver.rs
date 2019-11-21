use crate::common::*;

use CompilationErrorKind::*;

pub(crate) struct AssignmentResolver<'a: 'b, 'b> {
  assignments: &'b Table<'a, Assignment<'a>>,
  stack: Vec<&'a str>,
  seen: BTreeSet<&'a str>,
  evaluated: BTreeSet<&'a str>,
}

impl<'a: 'b, 'b> AssignmentResolver<'a, 'b> {
  pub(crate) fn resolve_assignments(
    assignments: &Table<'a, Assignment<'a>>,
  ) -> CompilationResult<'a, ()> {
    let mut resolver = AssignmentResolver {
      stack: empty(),
      seen: empty(),
      evaluated: empty(),
      assignments,
    };

    for name in assignments.keys() {
      resolver.resolve_assignment(name)?;
    }

    Ok(())
  }

  fn resolve_assignment(&mut self, name: &'a str) -> CompilationResult<'a, ()> {
    if self.evaluated.contains(name) {
      return Ok(());
    }

    self.seen.insert(name);
    self.stack.push(name);

    if let Some(assignment) = self.assignments.get(name) {
      self.resolve_expression(&assignment.expression)?;
      self.evaluated.insert(name);
    } else {
      let message = format!("attempted to resolve unknown assignment `{}`", name);
      let token = Token {
        src: "",
        offset: 0,
        line: 0,
        column: 0,
        length: 0,
        kind: TokenKind::Unspecified,
      };
      return Err(CompilationError {
        kind: Internal { message },
        token,
      });
    }
    Ok(())
  }

  fn resolve_expression(&mut self, expression: &Expression<'a>) -> CompilationResult<'a, ()> {
    match expression {
      Expression::Variable { name } => {
        let variable = name.lexeme();
        if self.evaluated.contains(variable) {
          Ok(())
        } else if self.seen.contains(variable) {
          let token = self.assignments[variable].name.token();
          self.stack.push(variable);
          Err(token.error(CircularVariableDependency {
            variable,
            circle: self.stack.clone(),
          }))
        } else if self.assignments.contains_key(variable) {
          self.resolve_assignment(variable)
        } else {
          Err(name.token().error(UndefinedVariable { variable }))
        }
      }
      Expression::Call { thunk } => match thunk {
        Thunk::Nullary { .. } => Ok(()),
        Thunk::Unary { arg, .. } => self.resolve_expression(arg),
        Thunk::Binary { args: [a, b], .. } => {
          self.resolve_expression(a)?;
          self.resolve_expression(b)
        }
      },
      Expression::Concatination { lhs, rhs } => {
        self.resolve_expression(lhs)?;
        self.resolve_expression(rhs)
      }
      Expression::StringLiteral { .. } | Expression::Backtick { .. } => Ok(()),
      Expression::Group { contents } => self.resolve_expression(contents),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  analysis_error! {
    name:   circular_variable_dependency,
    input:   "a = b\nb = a",
    offset:  0,
    line:   0,
    column: 0,
    width:  1,
    kind:   CircularVariableDependency{variable: "a", circle: vec!["a", "b", "a"]},
  }

  analysis_error! {
    name:   self_variable_dependency,
    input:  "a = a",
    offset:  0,
    line:   0,
    column: 0,
    width:  1,
    kind:   CircularVariableDependency{variable: "a", circle: vec!["a", "a"]},
  }

  analysis_error! {
    name:   unknown_expression_variable,
    input:  "x = yy",
    offset:  4,
    line:   0,
    column: 4,
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
