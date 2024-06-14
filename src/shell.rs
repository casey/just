use super::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Shell<'src> {
  pub(crate) arguments: Vec<StringLiteral<'src>>,
  pub(crate) command: StringLiteral<'src>,
}

impl<'src> Display for Shell<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "[{}", self.command)?;

    for argument in &self.arguments {
      write!(f, ", {argument}")?;
    }

    write!(f, "]")
  }
}
