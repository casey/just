use {super::*, CompileErrorKind::*};

pub(crate) struct AssignmentResolver<'src: 'run, 'run> {
  assignments: &'run Table<'src, Assignment<'src>>,
  evaluated: BTreeSet<&'src str>,
  functions: &'run Table<'src, FunctionDefinition<'src>>,
  stack: Vec<&'src str>,
}

impl<'src: 'run, 'run> AssignmentResolver<'src, 'run> {
  pub(crate) fn resolve_assignments(
    assignments: &'run Table<'src, Assignment<'src>>,
    functions: &'run Table<'src, FunctionDefinition<'src>>,
  ) -> CompileResult<'src> {
    let mut resolver = Self {
      assignments,
      evaluated: BTreeSet::new(),
      functions,
      stack: Vec::new(),
    };

    for assignment in assignments.values() {
      resolver.resolve_assignment(assignment)?;
    }

    for function in functions.values() {
      for reference in function.body.references() {
        resolver.resolve_reference(Some(&function.parameters), reference)?;
      }
    }

    Ok(())
  }

  fn resolve_assignment(&mut self, assignment: &Assignment<'src>) -> CompileResult<'src> {
    let name = assignment.name.lexeme();

    if self.evaluated.contains(name) {
      return Ok(());
    }

    self.stack.push(name);

    for reference in assignment.value.references() {
      self.resolve_reference(None, reference)?;
    }

    self.evaluated.insert(name);

    self.stack.pop();

    Ok(())
  }

  fn resolve_reference(
    &mut self,
    parameters: Option<&[(Name<'src>, Number)]>,
    reference: Reference<'src>,
  ) -> CompileResult<'src> {
    match reference {
      Reference::Call { name, arguments } => {
        Analyzer::resolve_call(self.functions, name, arguments)?;
        self.resolve_function_variables(name.lexeme())
      }
      Reference::Variable(name) => self.resolve_variable(parameters, name),
    }
  }

  fn resolve_function_variables(&mut self, root: &'src str) -> CompileResult<'src> {
    let functions = self.functions;

    let mut visited = BTreeSet::new();
    let mut queue = vec![root];

    while let Some(name) = queue.pop() {
      if !visited.insert(name) {
        continue;
      }

      let Some(function) = functions.get(name) else {
        continue;
      };

      for reference in function.body.references() {
        match reference {
          Reference::Call { name, .. } => queue.push(name.lexeme()),
          Reference::Variable(variable) => {
            self.resolve_variable(Some(&function.parameters), variable)?;
          }
        }
      }
    }

    Ok(())
  }

  fn resolve_variable(
    &mut self,
    parameters: Option<&[(Name<'src>, Number)]>,
    variable: Name<'src>,
  ) -> CompileResult<'src> {
    let name = variable.lexeme();

    if let Some(parameters) = parameters
      && parameters
        .iter()
        .any(|(parameter, _number)| parameter.lexeme() == name)
    {
      return Ok(());
    }

    if self.evaluated.contains(name) || constants().contains_key(name) {
      return Ok(());
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
    } else if let Some(assignment) = self.assignments.get(name) {
      self.resolve_assignment(assignment)?;
    } else {
      return Err(variable.error(UndefinedVariable { variable: name }));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  analysis_error! {
    name:   circular_variable_dependency,
    input:  "a := b\nb := a",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   CircularVariableDependency { variable: "a", circle: vec!["a", "b", "a"] },
  }

  analysis_error! {
    name:   self_variable_dependency,
    input:  "a := a",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   CircularVariableDependency { variable: "a", circle: vec!["a", "a"] },
  }

  analysis_error! {
    name:   circular_function_variable_dependency,
    input:  "a := f()\nf() := a",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   CircularVariableDependency { variable: "a", circle: vec!["a", "a"] },
  }

  analysis_error! {
    name:   circular_transitive_function_variable_dependency,
    input:  "a := f()\nf() := g()\ng() := a",
    offset: 0,
    line:   0,
    column: 0,
    width:  1,
    kind:   CircularVariableDependency { variable: "a", circle: vec!["a", "a"] },
  }

  #[test]
  fn function_parameters_shadow_variables() {
    testing::compile("a := f('x')\nf(a) := a");
  }

  analysis_error! {
    name:   unknown_expression_variable,
    input:  "x := yy",
    offset: 5,
    line:   0,
    column: 5,
    width:  2,
    kind:   UndefinedVariable { variable: "yy" },
  }

  analysis_error! {
    name:   undefined_function_parameter,
    input:  "x := env_var(yy)",
    offset: 13,
    line:   0,
    column: 13,
    width:  2,
    kind:   UndefinedVariable { variable: "yy" },
  }

  analysis_error! {
    name:   undefined_function_parameter_binary_first,
    input:  "x := env_var_or_default(yy, 'foo')",
    offset: 24,
    line:   0,
    column: 24,
    width:  2,
    kind:   UndefinedVariable { variable: "yy" },
  }

  analysis_error! {
    name:   undefined_function_parameter_binary_second,
    input:  "x := env_var_or_default('foo', yy)",
    offset: 31,
    line:   0,
    column: 31,
    width:  2,
    kind:   UndefinedVariable { variable: "yy" },
  }
}
