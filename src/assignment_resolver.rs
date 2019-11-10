use crate::common::*;

use CompilationErrorKind::*;

pub(crate) struct AssignmentResolver<'a: 'b, 'b> {
  assignments: &'b BTreeMap<&'a str, Assignment<'a>>,
  stack: Vec<&'a str>,
  seen: BTreeSet<&'a str>,
  evaluated: BTreeSet<&'a str>,
}

impl<'a: 'b, 'b> AssignmentResolver<'a, 'b> {
  pub(crate) fn resolve_assignments(
    assignments: &BTreeMap<&'a str, Assignment<'a>>,
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
      return Err(CompilationError {
        src: "",
        offset: 0,
        line: 0,
        column: 0,
        width: 0,
        kind: Internal { message },
      });
    }
    Ok(())
  }

  fn resolve_expression(&mut self, expression: &Expression<'a>) -> CompilationResult<'a, ()> {
    match expression {
      Expression::Variable { name } => {
        let variable = name.lexeme();
        if self.evaluated.contains(variable) {
          return Ok(());
        } else if self.seen.contains(variable) {
          let token = self.assignments[variable].name.token();
          self.stack.push(variable);
          return Err(token.error(CircularVariableDependency {
            variable,
            circle: self.stack.clone(),
          }));
        } else if self.assignments.contains_key(variable) {
          self.resolve_assignment(variable)?;
        } else {
          return Err(name.token().error(UndefinedVariable { variable }));
        }
      }
      Expression::Call {
        function,
        arguments,
      } => Function::resolve(&function.token(), arguments.len())?,
      Expression::Concatination { lhs, rhs } => {
        self.resolve_expression(lhs)?;
        self.resolve_expression(rhs)?;
      }
      Expression::StringLiteral { .. } | Expression::Backtick { .. } => {}
      Expression::Group { contents } => self.resolve_expression(contents)?,
    }
    Ok(())
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
    name:   unknown_function,
    input:  "a = foo()",
    offset:  4,
    line:   0,
    column: 4,
    width:  3,
    kind:   UnknownFunction{function: "foo"},
  }
}
