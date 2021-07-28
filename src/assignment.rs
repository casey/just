use crate::common::*;

/// An assignment, e.g `foo := bar`
pub(crate) type Assignment<'src> = Binding<'src, Expression<'src>, Condition<'src>>;

impl<'src> Display for Assignment<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    if self.export {
      write!(f, "export ")?;
    }

    write!(f, "{} := {}", self.name, self.value)?;

    if let Some(condition) = &self.condition {
      write!(f, " if {}", condition)?;
    }

    Ok(())
  }
}
