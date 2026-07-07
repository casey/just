use {super::*, CompileErrorKind::*};

pub(crate) struct VariableResolver<'src: 'run, 'run> {
  assignments: &'run Table<'src, Assignment<'src>>,
  evaluated: BTreeSet<&'src str>,
  functions: &'run Table<'src, FunctionDefinition<'src>>,
  stack: Vec<&'src str>,
}

impl<'src: 'run, 'run> VariableResolver<'src, 'run> {
  pub(crate) fn new(
    assignments: &'run Table<'src, Assignment<'src>>,
    functions: &'run Table<'src, FunctionDefinition<'src>>,
  ) -> CompileResult<'src, Self> {
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
      let context = ExpressionContext::from(function.parameters.as_slice());
      for reference in function.body.references() {
        resolver.resolve_reference(&context, reference)?;
      }
    }

    Ok(resolver)
  }

  pub(crate) fn resolve_expression(
    &mut self,
    expression: &Expression<'src>,
    context: &ExpressionContext<'src>,
    references: &mut HashSet<Number>,
  ) -> CompileResult<'src> {
    for reference in expression.references() {
      match reference {
        Reference::Call { name, arguments } => self.resolve_call(name, arguments)?,
        Reference::Variable(variable) => {
          self.resolve_variable(context, variable, Some(&mut *references))?;
        }
      }
    }

    Ok(())
  }

  pub(crate) fn collect_references(
    &self,
    expression: &Expression<'src>,
    references: &mut HashSet<Number>,
  ) {
    for reference in expression.references() {
      if let Reference::Variable(variable) = reference
        && let Some(assignment) = self.assignments.get(variable.lexeme())
      {
        references.insert(assignment.number);
      }
    }
  }

  pub(crate) fn resolve_call(&self, name: Name<'src>, arguments: usize) -> CompileResult<'src> {
    let function = name.lexeme();

    let expected = if let Some(function) = self.functions.get(function) {
      function.parameters.len()..=function.parameters.len()
    } else if let Some(function) = function::get(function) {
      function.expected_arguments()
    } else {
      return Err(name.error(UndefinedFunction { function }));
    };

    if !expected.contains(&arguments) {
      return Err(name.error(FunctionArgumentCountMismatch {
        arguments,
        expected,
        function,
      }));
    }

    Ok(())
  }

  fn resolve_assignment(&mut self, assignment: &Assignment<'src>) -> CompileResult<'src> {
    let name = assignment.name.lexeme();

    if self.evaluated.contains(name) {
      return Ok(());
    }

    self.stack.push(name);

    let context = ExpressionContext::new();

    for reference in assignment.value.references() {
      self.resolve_reference(&context, reference)?;
    }

    self.evaluated.insert(name);

    self.stack.pop();

    Ok(())
  }

  fn resolve_reference(
    &mut self,
    context: &ExpressionContext<'src>,
    reference: Reference<'src>,
  ) -> CompileResult<'src> {
    match reference {
      Reference::Call { name, arguments } => {
        self.resolve_call(name, arguments)?;
        self.resolve_function_variables(name.lexeme())
      }
      Reference::Variable(name) => self.resolve_variable(context, name, None),
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

      let context = ExpressionContext::from(function.parameters.as_slice());

      for reference in function.body.references() {
        match reference {
          Reference::Call { name, .. } => queue.push(name.lexeme()),
          Reference::Variable(variable) => {
            self.resolve_variable(&context, variable, None)?;
          }
        }
      }
    }

    Ok(())
  }

  fn resolve_variable(
    &mut self,
    context: &ExpressionContext<'src>,
    variable: Name<'src>,
    references: Option<&mut HashSet<Number>>,
  ) -> CompileResult<'src> {
    let name = variable.lexeme();

    if context.shadows(name) {
      return Ok(());
    }

    if self.evaluated.contains(name) {
      if let Some(references) = references {
        references.insert(self.assignments[name].number);
      }
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
    }

    if let Some(assignment) = self.assignments.get(name) {
      return self.resolve_assignment(assignment);
    }

    if constants().contains_key(name) {
      return Ok(());
    }

    Err(variable.error(UndefinedVariable { variable: name }))
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

  analysis_error! {
    name:   constant_shadowing_self_variable_dependency,
    input:  "HEX := HEX",
    offset: 0,
    line:   0,
    column: 0,
    width:  3,
    kind:   CircularVariableDependency { variable: "HEX", circle: vec!["HEX", "HEX"] },
  }

  analysis_error! {
    name:   constant_shadowing_circular_variable_dependency,
    input:  "x := HEX\nHEX := x",
    offset: 9,
    line:   1,
    column: 0,
    width:  3,
    kind:   CircularVariableDependency { variable: "HEX", circle: vec!["HEX", "x", "HEX"] },
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
