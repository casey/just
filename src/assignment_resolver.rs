use common::*;

use CompilationErrorKind::*;

pub struct AssignmentResolver<'a: 'b, 'b> {
  assignments: &'b Map<&'a str, Expression<'a>>,
  assignment_tokens: &'b Map<&'a str, Token<'a>>,
  stack: Vec<&'a str>,
  seen: Set<&'a str>,
  evaluated: Set<&'a str>,
}

impl<'a: 'b, 'b> AssignmentResolver<'a, 'b> {
  pub fn resolve_assignments(
    assignments: &Map<&'a str, Expression<'a>>,
    assignment_tokens: &Map<&'a str, Token<'a>>,
  ) -> CompilationResult<'a, ()> {
    let mut resolver = AssignmentResolver {
      stack: empty(),
      seen: empty(),
      evaluated: empty(),
      assignments,
      assignment_tokens,
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

    if let Some(expression) = self.assignments.get(name) {
      self.resolve_expression(expression)?;
      self.evaluated.insert(name);
    } else {
      let message = format!("attempted to resolve unknown assignment `{}`", name);
      return Err(CompilationError {
        text: "",
        index: 0,
        line: 0,
        column: 0,
        width: None,
        kind: Internal { message },
      });
    }
    Ok(())
  }

  fn resolve_expression(&mut self, expression: &Expression<'a>) -> CompilationResult<'a, ()> {
    match *expression {
      Expression::Variable { name, ref token } => {
        if self.evaluated.contains(name) {
          return Ok(());
        } else if self.seen.contains(name) {
          let token = &self.assignment_tokens[name];
          self.stack.push(name);
          return Err(token.error(CircularVariableDependency {
            variable: name,
            circle: self.stack.clone(),
          }));
        } else if self.assignments.contains_key(name) {
          self.resolve_assignment(name)?;
        } else {
          return Err(token.error(UndefinedVariable { variable: name }));
        }
      }
      Expression::Call {
        ref token,
        ref arguments,
        ..
      } => resolve_function(token, arguments.len())?,
      Expression::Concatination { ref lhs, ref rhs } => {
        self.resolve_expression(lhs)?;
        self.resolve_expression(rhs)?;
      }
      Expression::String { .. } | Expression::Backtick { .. } => {}
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  compilation_error_test! {
    name:   circular_variable_dependency,
    input:   "a = b\nb = a",
    index:  0,
    line:   0,
    column: 0,
    width:  Some(1),
    kind:   CircularVariableDependency{variable: "a", circle: vec!["a", "b", "a"]},
  }

  compilation_error_test! {
    name:   self_variable_dependency,
    input:  "a = a",
    index:  0,
    line:   0,
    column: 0,
    width:  Some(1),
    kind:   CircularVariableDependency{variable: "a", circle: vec!["a", "a"]},
  }

  compilation_error_test! {
    name:   unknown_expression_variable,
    input:  "x = yy",
    index:  4,
    line:   0,
    column: 4,
    width:  Some(2),
    kind:   UndefinedVariable{variable: "yy"},
  }

  compilation_error_test! {
    name:   unknown_function,
    input:  "a = foo()",
    index:  4,
    line:   0,
    column: 4,
    width:  Some(3),
    kind:   UnknownFunction{function: "foo"},
  }

}
