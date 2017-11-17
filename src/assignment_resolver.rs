use common::*;

pub fn resolve_assignments<'a>(
  assignments:       &Map<&'a str, Expression<'a>>,
  assignment_tokens: &Map<&'a str, Token<'a>>,
) -> Result<(), CompilationError<'a>> {

  let mut resolver = AssignmentResolver {
    assignments:       assignments,
    assignment_tokens: assignment_tokens,
    stack:             empty(),
    seen:              empty(),
    evaluated:         empty(),
  };

  for name in assignments.keys() {
    resolver.resolve_assignment(name)?;
  }

  Ok(())
}

struct AssignmentResolver<'a: 'b, 'b> {
  assignments:       &'b Map<&'a str, Expression<'a>>,
  assignment_tokens: &'b Map<&'a str, Token<'a>>,
  stack:             Vec<&'a str>,
  seen:              Set<&'a str>,
  evaluated:         Set<&'a str>,
}

impl<'a: 'b, 'b> AssignmentResolver<'a, 'b> {
  fn resolve_assignment(&mut self, name: &'a str) -> Result<(), CompilationError<'a>> {
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
        text:   "",
        index:  0,
        line:   0,
        column: 0,
        width:  None,
        kind:   CompilationErrorKind::InternalError{message}
      });
    }
    Ok(())
  }

  fn resolve_expression(&mut self, expression: &Expression<'a>) -> Result<(), CompilationError<'a>> {
    match *expression {
      Expression::Variable{name, ref token} => {
        if self.evaluated.contains(name) {
          return Ok(());
        } else if self.seen.contains(name) {
          let token = &self.assignment_tokens[name];
          self.stack.push(name);
          return Err(token.error(CompilationErrorKind::CircularVariableDependency {
            variable: name,
            circle:   self.stack.clone(),
          }));
        } else if self.assignments.contains_key(name) {
          self.resolve_assignment(name)?;
        } else {
          return Err(token.error(CompilationErrorKind::UndefinedVariable{variable: name}));
        }
      }
      Expression::Concatination{ref lhs, ref rhs} => {
        self.resolve_expression(lhs)?;
        self.resolve_expression(rhs)?;
      }
      Expression::String{..} | Expression::Backtick{..} => {}
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use testing::parse_error;
  use super::*;

#[test]
fn circular_variable_dependency() {
  let text = "a = b\nb = a";
  parse_error(text, CompilationError {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  Some(1),
    kind:   CompilationErrorKind::CircularVariableDependency{variable: "a", circle: vec!["a", "b", "a"]}
  });
}


#[test]
fn self_variable_dependency() {
  let text = "a = a";
  parse_error(text, CompilationError {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  Some(1),
    kind:   CompilationErrorKind::CircularVariableDependency{variable: "a", circle: vec!["a", "a"]}
  });
}
#[test]
fn unknown_expression_variable() {
  let text = "x = yy";
  parse_error(text, CompilationError {
    text:   text,
    index:  4,
    line:   0,
    column: 4,
    width:  Some(2),
    kind:   CompilationErrorKind::UndefinedVariable{variable: "yy"},
  });
}

}
