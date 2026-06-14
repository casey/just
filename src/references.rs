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
        Expression::And { lhs, rhs } | Expression::Or { lhs, rhs } => {
          self.stack.push(lhs);
          self.stack.push(rhs);
        }
        Expression::Assert {
          condition, error, ..
        } => {
          self.stack.push(error);
          self.stack.push(condition);
        }
        Expression::Backtick { .. } | Expression::StringLiteral { .. } => {}
        Expression::Call { name, arguments } => {
          for arg in arguments.iter().rev() {
            self.stack.push(arg);
          }
          return Some(Reference::Call {
            name: *name,
            arguments: arguments.len(),
          });
        }
        Expression::Comparison { lhs, rhs, .. } | Expression::Concatenation { lhs, rhs } => {
          self.stack.push(rhs);
          self.stack.push(lhs);
        }
        Expression::Conditional {
          condition,
          then,
          otherwise,
        } => {
          self.stack.push(otherwise);
          self.stack.push(then);
          self.stack.push(condition);
        }
        Expression::FormatString { expressions, .. } => {
          for (expression, _string) in expressions {
            self.stack.push(expression);
          }
        }
        Expression::Group { contents } => {
          self.stack.push(contents);
        }
        Expression::Join { lhs, rhs } => {
          self.stack.push(rhs);
          if let Some(lhs) = lhs {
            self.stack.push(lhs);
          }
        }
        Expression::List { elements } => {
          for element in elements.iter().rev() {
            self.stack.push(element);
          }
        }
        Expression::Not { operand } => {
          self.stack.push(operand);
        }
        Expression::Variable { name, .. } => return Some(Reference::Variable(*name)),
      }
    }
  }
}
