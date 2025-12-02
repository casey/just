use super::*;

#[derive(Debug, Clone)]
pub(crate) enum StringLiteralOrArray<'src> {
  Single(StringLiteral<'src>),
  Multiple(Vec<StringLiteral<'src>>),
}

impl<'src> StringLiteralOrArray<'src> {
  pub(crate) fn cooked(&self) -> Vec<String> {
    match self {
      Self::Single(lit) => vec![lit.cooked.clone()],
      Self::Multiple(lits) => lits.iter().map(|lit| lit.cooked.clone()).collect(),
    }
  }
}

impl Display for StringLiteralOrArray<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Single(lit) => write!(f, "{lit}"),
      Self::Multiple(lits) => {
        write!(f, "[")?;
        for (i, lit) in lits.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{lit}")?;
        }
        write!(f, "]")
      }
    }
  }
}
