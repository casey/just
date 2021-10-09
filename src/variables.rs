use crate::common::*;

pub(crate) struct Variables<'expression, 'src> {
  stack: Vec<&'expression Expression<'src>>,
}

impl<'expression, 'src> Variables<'expression, 'src> {
  pub(crate) fn new(root: &'expression Expression<'src>) -> Variables<'expression, 'src> {
    Variables { stack: vec![root] }
  }
}

impl<'expression, 'src> Iterator for Variables<'expression, 'src> {
  type Item = Token<'src>;

  fn next(&mut self) -> Option<Token<'src>> {
    loop {
      match self.stack.pop()? {
        Expression::StringLiteral { .. } | Expression::Backtick { .. } => {}
        Expression::Call { thunk } => match thunk {
          Thunk::Nullary { .. } => {}
          Thunk::Unary { arg, .. } => self.stack.push(arg),
          Thunk::Binary { args, .. } => {
            for arg in args.iter().rev() {
              self.stack.push(arg);
            }
          }
          Thunk::Ternary { args, .. } => {
            for arg in args.iter().rev() {
              self.stack.push(arg);
            }
          }
        },
        Expression::Conditional {
          lhs,
          rhs,
          then,
          otherwise,
          ..
        } => {
          self.stack.push(lhs);
          self.stack.push(rhs);
          self.stack.push(then);
          self.stack.push(otherwise);
        }
        Expression::Variable { name, .. } => return Some(name.token()),
        Expression::Concatination { lhs, rhs } => {
          self.stack.push(lhs);
          self.stack.push(rhs);
        }
        Expression::Group { contents } => {
          self.stack.push(contents);
        }
      }
    }
  }
}
