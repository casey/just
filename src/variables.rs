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
    match self.stack.pop() {
      None
      | Some(Expression::StringLiteral { .. })
      | Some(Expression::Backtick { .. })
      | Some(Expression::Call { .. }) => None,
      Some(Expression::Conditional {
        condition,
        then,
        otherwise,
        ..
      }) => {
        self.stack.push(&condition.lhs);
        self.stack.push(&condition.rhs);
        self.stack.push(then);
        self.stack.push(otherwise);
        self.next()
      },
      Some(Expression::Variable { name, .. }) => Some(name.token()),
      Some(Expression::Concatination { lhs, rhs }) => {
        self.stack.push(lhs);
        self.stack.push(rhs);
        self.next()
      },
      Some(Expression::Group { contents }) => {
        self.stack.push(contents);
        self.next()
      },
    }
  }
}
