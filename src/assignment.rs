use super::*;

/// An assignment, e.g `foo := bar`
pub(crate) type Assignment<'src> = Binding<'src, Expression<'src>>;

impl<'src> Table<'src, Assignment<'src>> {
  pub(crate) fn assignment(&self, number: Number) -> Option<&Assignment<'src>> {
    self.values().find(|assignment| assignment.number == number)
  }
}

impl Display for Assignment<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.eager {
      write!(f, "eager ")?;
    }

    if self.export {
      write!(f, "export ")?;
    }

    write!(f, "{} := {}", self.name, self.value)
  }
}
