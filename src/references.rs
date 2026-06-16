use super::*;

pub(crate) struct References<'expression, 'src> {
  stack: Vec<&'expression Expression<'src>>,
}

impl<'expression, 'src> References<'expression, 'src> {
  pub(crate) fn new(root: &'expression Expression<'src>) -> Self {
    Self { stack: vec![root] }
  }
}

impl<'src> Iterator for References<'_, 'src> {
  type Item = Reference<'src>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.stack.pop()? {
        Expression::Assert {
          condition, error, ..
        } => {
          self.stack.push(error);
          self.stack.push(condition);
        }
        Expression::Backtick { .. } | Expression::StringLiteral { .. } => {}
        Expression::Binary { lhs, rhs, .. } => {
          self.stack.push(rhs);
          self.stack.push(lhs);
        }
        Expression::Call { name, arguments } => {
          for arg in arguments.iter().rev() {
            self.stack.push(arg);
          }
          return Some(Reference::Call {
            name: *name,
            arguments: arguments.len(),
          });
        }
        Expression::Conditional {
          condition,
          then,
          otherwise,
        } => {
          self.stack.push(then);
          self.stack.push(condition);
          if let Some(otherwise) = otherwise {
            self.stack.push(otherwise);
          }
        }
        Expression::FormatString { expressions, .. } => {
          for (expression, _string) in expressions {
            self.stack.push(expression);
          }
        }
        Expression::Group { contents } => {
          self.stack.push(contents);
        }
        Expression::List { elements, .. } => {
          for element in elements.iter().rev() {
            self.stack.push(element);
          }
        }
        Expression::Unary { operand, .. } => {
          self.stack.push(operand);
        }
        Expression::Variable { name, .. } => return Some(Reference::Variable(*name)),
      }
    }
  }
}
