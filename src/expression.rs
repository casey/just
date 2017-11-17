use common::*;

#[derive(PartialEq, Debug)]
pub enum Expression<'a> {
  Variable{name: &'a str, token: Token<'a>},
  String{cooked_string: CookedString<'a>},
  Backtick{raw: &'a str, token: Token<'a>},
  Concatination{lhs: Box<Expression<'a>>, rhs: Box<Expression<'a>>},
}

impl<'a> Expression<'a> {
  pub fn variables(&'a self) -> Variables<'a> {
    Variables {
      stack: vec![self],
    }
  }
}

impl<'a> Display for Expression<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Expression::Backtick     {raw, ..          } => write!(f, "`{}`", raw)?,
      Expression::Concatination{ref lhs, ref rhs } => write!(f, "{} + {}", lhs, rhs)?,
      Expression::String       {ref cooked_string} => write!(f, "\"{}\"", cooked_string.raw)?,
      Expression::Variable     {name, ..         } => write!(f, "{}", name)?,
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
      None | Some(&Expression::String{..}) | Some(&Expression::Backtick{..}) => None,
      Some(&Expression::Variable{ref token,..})          => Some(token),
      Some(&Expression::Concatination{ref lhs, ref rhs}) => {
        self.stack.push(lhs);
        self.stack.push(rhs);
        self.next()
      }
    }
  }
}
