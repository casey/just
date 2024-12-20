use super::*;

pub(crate) struct Variables<'expression, 'src> {
  stack: Vec<&'expression Expression<'src>>,
}

impl<'expression, 'src> Variables<'expression, 'src> {
  pub(crate) fn new(root: &'expression Expression<'src>) -> Self {
    Self { stack: vec![root] }
  }
}

impl<'src> Iterator for Variables<'_, 'src> {
  type Item = Token<'src>;

  fn next(&mut self) -> Option<Token<'src>> {
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
        } => {
          self.stack.push(error);
          self.stack.push(rhs);
          self.stack.push(lhs);
        }
        Expression::Backtick { .. } | Expression::StringLiteral { .. } => {}
        Expression::Call { thunk } => match thunk {
          Thunk::Nullary { .. } => {}
          Thunk::Unary { arg, .. } => self.stack.push(arg),
          Thunk::UnaryOpt {
            args: (a, opt_b), ..
          } => {
            self.stack.push(a);
            if let Some(b) = opt_b.as_ref() {
              self.stack.push(b);
            }
          }
          Thunk::UnaryPlus {
            args: (a, rest), ..
          } => {
            let first: &[&Expression] = &[a];
            for arg in first.iter().copied().chain(rest).rev() {
              self.stack.push(arg);
            }
          }
          Thunk::Binary { args, .. } => {
            for arg in args.iter().rev() {
              self.stack.push(arg);
            }
          }
          Thunk::BinaryPlus {
            args: ([a, b], rest),
            ..
          } => {
            let first: &[&Expression] = &[a, b];
            for arg in first.iter().copied().chain(rest).rev() {
              self.stack.push(arg);
            }
          }
          Thunk::Ternary { args, .. } => {
            for arg in args.iter().rev() {
              self.stack.push(arg);
            }
          }
        },
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
        Expression::Group { contents } => {
          self.stack.push(contents);
        }
        Expression::Join { lhs, rhs } => {
          self.stack.push(rhs);
          if let Some(lhs) = lhs {
            self.stack.push(lhs);
          }
        }
        Expression::Variable { name, .. } => return Some(name.token),
      }
    }
  }
}
