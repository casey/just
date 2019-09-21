use crate::common::*;

#[derive(PartialEq, Debug)]
pub(crate) enum Expression<'a> {
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
    cooked_string: StringLiteral<'a>,
  },
  Variable {
    name: &'a str,
    token: Token<'a>,
  },
  Group {
    expression: Box<Expression<'a>>,
  },
}

impl<'a> Expression<'a> {
  pub(crate) fn variables(&'a self) -> Variables<'a> {
    Variables::new(self)
  }

  pub(crate) fn functions(&'a self) -> Functions<'a> {
    Functions::new(self)
  }
}

impl<'a> Display for Expression<'a> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match *self {
      Expression::Backtick { raw, .. } => write!(f, "`{}`", raw)?,
      Expression::Concatination { ref lhs, ref rhs } => write!(f, "{} + {}", lhs, rhs)?,
      Expression::String { ref cooked_string } => write!(f, "{}", cooked_string)?,
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
      Expression::Group { ref expression } => write!(f, "({})", expression)?,
    }
    Ok(())
  }
}
