use crate::common::*;

pub(crate) struct Functions<'expression, 'src> {
  stack: Vec<&'expression Expression<'src>>,
}

impl<'expression, 'src> Functions<'expression, 'src> {
  pub(crate) fn new(root: &'expression Expression<'src>) -> Functions<'expression, 'src> {
    Functions { stack: vec![root] }
  }
}

impl<'expression, 'src> Iterator for Functions<'expression, 'src> {
  type Item = (Token<'src>, usize);

  fn next(&mut self) -> Option<Self::Item> {
    match self.stack.pop() {
      None
      | Some(Expression::StringLiteral { .. })
      | Some(Expression::Backtick { .. })
      | Some(Expression::Variable { .. }) => None,
      Some(Expression::Call {
        function,
        arguments,
        ..
      }) => Some((function.token(), arguments.len())),
      Some(Expression::Concatination { lhs, rhs }) => {
        self.stack.push(lhs);
        self.stack.push(rhs);
        self.next()
      }
      Some(Expression::Group { contents }) => {
        self.stack.push(contents);
        self.next()
      }
    }
  }
}
