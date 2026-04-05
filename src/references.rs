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
          condition:
            Condition {
              lhs,
              rhs,
              operator: _,
            },
          error,
          ..
        } => {
          self.stack.push(error);
          self.stack.push(rhs);
          self.stack.push(lhs);
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
        Expression::Concatenation { lhs, rhs } => {
          self.stack.push(rhs);
          self.stack.push(lhs);
        }
        Expression::Conditional {
          condition:
            Condition {
              lhs,
              rhs,
              operator: _,
            },
          then,
          otherwise,
        } => {
          self.stack.push(otherwise);
          self.stack.push(then);
          self.stack.push(rhs);
          self.stack.push(lhs);
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
        Expression::Variable { name, .. } => return Some(Reference::Variable(*name)),
      }
    }
  }
}
