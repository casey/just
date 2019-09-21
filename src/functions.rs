use crate::common::*;

pub(crate) struct Functions<'a> {
  stack: Vec<&'a Expression<'a>>,
}

impl<'a> Functions<'a> {
  pub(crate) fn new(root: &'a Expression<'a>) -> Functions<'a> {
    Functions { stack: vec![root] }
  }
}

impl<'a> Iterator for Functions<'a> {
  type Item = (&'a Token<'a>, usize);

  fn next(&mut self) -> Option<Self::Item> {
    match self.stack.pop() {
      None
      | Some(Expression::String { .. })
      | Some(Expression::Backtick { .. })
      | Some(Expression::Variable { .. }) => None,
      Some(Expression::Call {
        token, arguments, ..
      }) => Some((token, arguments.len())),
      Some(Expression::Concatination { lhs, rhs }) => {
        self.stack.push(lhs);
        self.stack.push(rhs);
        self.next()
      }
      Some(Expression::Group { expression }) => {
        self.stack.push(expression);
        self.next()
      }
    }
  }
}
