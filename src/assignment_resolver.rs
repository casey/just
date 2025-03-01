use {super::*, CompileErrorKind::*};

pub(crate) struct AssignmentResolver<'src: 'run, 'run> {
  assignments: &'run Table<'src, Assignment<'src>>,
  evaluated: BTreeSet<&'src str>,
  stack: Vec<&'src str>,
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
      for variable in assignment.value.variables() {
        let name = variable.lexeme();

        if self.evaluated.contains(name) || constants().contains_key(name) {
          continue;
        }

        if self.stack.contains(&name) {
          self.stack.push(name);
          return Err(
            self.assignments[name]
              .name
              .error(CircularVariableDependency {
                variable: name,
                circle: self.stack.clone(),
              }),
          );
        } else if self.assignments.contains_key(name) {
          self.resolve_assignment(name)?;
        } else {
          return Err(variable.error(UndefinedVariable { variable: name }));
        }
      }
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
