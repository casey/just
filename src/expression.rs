use crate::common::*;

#[derive(PartialEq, Debug)]
pub enum Expression<'a> {
  Backtick {
    raw: &'a str,
    token: Token<'a>,
  },
  Call {
    name: &'a str,
    token: Token<'a>,
    arguments: Vec<Expression<'a>>,
  },
  Concatination {
    lhs: Box<Expression<'a>>,
    rhs: Box<Expression<'a>>,
  },
  String {
    cooked_string: CookedString<'a>,
  },
  Variable {
    name: &'a str,
    token: Token<'a>,
  },
}

impl<'a> Expression<'a> {
  pub fn variables(&'a self) -> Variables<'a> {
    Variables { stack: vec![self] }
  }

  pub fn functions(&'a self) -> Functions<'a> {
    Functions { stack: vec![self] }
  }
}

impl<'a> Display for Expression<'a> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match *self {
      Expression::Backtick { raw, .. } => write!(f, "`{}`", raw)?,
      Expression::Concatination { ref lhs, ref rhs } => write!(f, "{} + {}", lhs, rhs)?,
      Expression::String { ref cooked_string } => write!(f, "\"{}\"", cooked_string.raw)?,
      Expression::Variable { name, .. } => write!(f, "{}", name)?,
      Expression::Call {
        name,
        ref arguments,
        ..
      } => {
        write!(f, "{}(", name)?;
        for (i, argument) in arguments.iter().enumerate() {
          if i > 0 {
            write!(f, ", {}", argument)?;
          } else {
            write!(f, "{}", argument)?;
          }
        }
        write!(f, ")")?;
      }
    }
    Ok(())
  }
}

pub struct Variables<'a> {
  stack: Vec<&'a Expression<'a>>,
}

impl<'a> Iterator for Variables<'a> {
  type Item = &'a Token<'a>;

  fn next(&mut self) -> Option<&'a Token<'a>> {
    match self.stack.pop() {
      None
      | Some(&Expression::String { .. })
      | Some(&Expression::Backtick { .. })
      | Some(&Expression::Call { .. }) => None,
      Some(&Expression::Variable { ref token, .. }) => Some(token),
      Some(&Expression::Concatination { ref lhs, ref rhs }) => {
        self.stack.push(lhs);
        self.stack.push(rhs);
        self.next()
      }
    }
  }
}

pub struct Functions<'a> {
  stack: Vec<&'a Expression<'a>>,
}

impl<'a> Iterator for Functions<'a> {
  type Item = (&'a Token<'a>, usize);

  fn next(&mut self) -> Option<Self::Item> {
    match self.stack.pop() {
      None
      | Some(&Expression::String { .. })
      | Some(&Expression::Backtick { .. })
      | Some(&Expression::Variable { .. }) => None,
      Some(&Expression::Call {
        ref token,
        ref arguments,
        ..
      }) => Some((token, arguments.len())),
      Some(&Expression::Concatination { ref lhs, ref rhs }) => {
        self.stack.push(lhs);
        self.stack.push(rhs);
        self.next()
      }
    }
  }
}
