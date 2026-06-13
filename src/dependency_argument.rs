use super::*;

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct DependencyArgument<'src> {
  pub(crate) expression: Expression<'src>,
  pub(crate) starred: bool,
}

impl Display for DependencyArgument<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.starred {
      write!(f, "*")?;
    }
    write!(f, "{}", self.expression)
  }
}
