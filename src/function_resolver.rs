use super::*;

pub(crate) struct FunctionResolver<'src: 'run, 'run> {
  assignments: &'run Table<'src, Assignment<'src>>,
  functions: &'run Table<'src, UserFunction<'src>>,
}

impl<'src: 'run, 'run> FunctionResolver<'src, 'run> {
  pub(crate) fn resolve_functions(
    assignments: &'run Table<'src, Assignment<'src>>,
    functions: &'run Table<'src, UserFunction<'src>>,
  ) -> CompileResult<'src> {
    let resolver = Self {
      assignments,
      functions,
    };

    for function in functions.values() {
      resolver.resolve_function(function)?;
    }

    Ok(())
  }

  pub(crate) fn resolve_calls(
    functions: &Table<'src, UserFunction<'src>>,
    expression: &Expression<'src>,
  ) -> CompileResult<'src> {
    match expression {
      Expression::And { lhs, rhs }
      | Expression::Concatenation { lhs, rhs }
      | Expression::Or { lhs, rhs } => {
        Self::resolve_calls(functions, lhs)?;
        Self::resolve_calls(functions, rhs)?;
      }
      Expression::Assert {
        condition, error, ..
      } => {
        Self::resolve_calls(functions, &condition.lhs)?;
        Self::resolve_calls(functions, &condition.rhs)?;
        Self::resolve_calls(functions, error)?;
      }
      Expression::Backtick { .. }
      | Expression::StringLiteral { .. }
      | Expression::Variable { .. } => {}
      Expression::Call { name, arguments } => {
        for arg in arguments {
          Self::resolve_calls(functions, arg)?;
        }

        if let Some(user_fn) = functions.get(name.lexeme()) {
          if user_fn.parameters.len() != arguments.len() {
            return Err(name.error(CompileErrorKind::FunctionArgumentCountMismatch {
              function: name.lexeme(),
              found: arguments.len(),
              expected: user_fn.parameters.len()..=user_fn.parameters.len(),
            }));
          }
        } else if let Some(builtin) = function::get(name.lexeme()) {
          if !builtin.argc().contains(&arguments.len()) {
            return Err(name.error(CompileErrorKind::FunctionArgumentCountMismatch {
              function: name.lexeme(),
              found: arguments.len(),
              expected: builtin.argc(),
            }));
          }
        } else {
          return Err(name.error(CompileErrorKind::UnknownFunction {
            function: name.lexeme(),
          }));
        }
      }
      Expression::Conditional {
        condition,
        then,
        otherwise,
      } => {
        Self::resolve_calls(functions, &condition.lhs)?;
        Self::resolve_calls(functions, &condition.rhs)?;
        Self::resolve_calls(functions, then)?;
        Self::resolve_calls(functions, otherwise)?;
      }
      Expression::FormatString { expressions, .. } => {
        for (expression, _) in expressions {
          Self::resolve_calls(functions, expression)?;
        }
      }
      Expression::Group { contents } => {
        Self::resolve_calls(functions, contents)?;
      }
      Expression::Join { lhs, rhs } => {
        if let Some(lhs) = lhs {
          Self::resolve_calls(functions, lhs)?;
        }
        Self::resolve_calls(functions, rhs)?;
      }
    }

    Ok(())
  }

  fn resolve_function(&self, function: &UserFunction<'src>) -> CompileResult<'src> {
    self.resolve_expression(&function.body, &function.parameters)
  }

  fn resolve_expression(
    &self,
    expression: &Expression<'src>,
    parameters: &[Name<'src>],
  ) -> CompileResult<'src> {
    match expression {
      Expression::And { lhs, rhs }
      | Expression::Concatenation { lhs, rhs }
      | Expression::Or { lhs, rhs } => {
        self.resolve_expression(lhs, parameters)?;
        self.resolve_expression(rhs, parameters)?;
      }
      Expression::Assert {
        condition, error, ..
      } => {
        self.resolve_expression(&condition.lhs, parameters)?;
        self.resolve_expression(&condition.rhs, parameters)?;
        self.resolve_expression(error, parameters)?;
      }
      Expression::Backtick { .. } | Expression::StringLiteral { .. } => {}
      Expression::Call { name, arguments } => {
        for arg in arguments {
          self.resolve_expression(arg, parameters)?;
        }
        self.resolve_call(name, arguments.len())?;
      }
      Expression::Conditional {
        condition,
        then,
        otherwise,
      } => {
        self.resolve_expression(&condition.lhs, parameters)?;
        self.resolve_expression(&condition.rhs, parameters)?;
        self.resolve_expression(then, parameters)?;
        self.resolve_expression(otherwise, parameters)?;
      }
      Expression::FormatString { expressions, .. } => {
        for (expression, _) in expressions {
          self.resolve_expression(expression, parameters)?;
        }
      }
      Expression::Group { contents } => {
        self.resolve_expression(contents, parameters)?;
      }
      Expression::Join { lhs, rhs } => {
        if let Some(lhs) = lhs {
          self.resolve_expression(lhs, parameters)?;
        }
        self.resolve_expression(rhs, parameters)?;
      }
      Expression::Variable { name } => {
        self.resolve_variable(name, parameters)?;
      }
    }

    Ok(())
  }

  fn resolve_variable(
    &self,
    variable: &Name<'src>,
    parameters: &[Name<'src>],
  ) -> CompileResult<'src> {
    let name = variable.lexeme();

    if parameters.iter().any(|p| p.lexeme() == name) {
      return Ok(());
    }

    if self.assignments.contains_key(name) {
      return Ok(());
    }

    if constants().contains_key(name) {
      return Ok(());
    }

    Err(variable.error(CompileErrorKind::UndefinedVariable { variable: name }))
  }

  fn resolve_call(&self, name: &Name<'src>, argument_count: usize) -> CompileResult<'src> {
    if let Some(user_fn) = self.functions.get(name.lexeme()) {
      if user_fn.parameters.len() != argument_count {
        return Err(name.error(CompileErrorKind::FunctionArgumentCountMismatch {
          function: name.lexeme(),
          found: argument_count,
          expected: user_fn.parameters.len()..=user_fn.parameters.len(),
        }));
      }
      return Ok(());
    }

    if let Some(builtin) = function::get(name.lexeme()) {
      if !builtin.argc().contains(&argument_count) {
        return Err(name.error(CompileErrorKind::FunctionArgumentCountMismatch {
          function: name.lexeme(),
          found: argument_count,
          expected: builtin.argc(),
        }));
      }
      return Ok(());
    }

    Err(name.error(CompileErrorKind::UnknownFunction {
      function: name.lexeme(),
    }))
  }
}
