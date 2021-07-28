use crate::common::*;

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Condition<'src> {
  pub(crate) lhs:      Expression<'src>,
  pub(crate) rhs:      Expression<'src>,
  pub(crate) inverted: bool,
}

impl<'src> Display for Condition<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", self.lhs)?;
    if self.inverted {
      write!(f, " != ")?;
    } else {
      write!(f, " == ")?;
    }
    write!(f, "{}", self.rhs)?;
    Ok(())
  }
}
