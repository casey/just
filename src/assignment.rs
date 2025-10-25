use super::*;

/// An assignment, e.g `foo := bar`
pub(crate) type Assignment<'src> = Binding<'src, Expression<'src>>;

impl Display for Assignment<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.private {
      writeln!(f, "[private]")?;
    }

    if self.lazy {
      write!(f, "lazy ")?;
    }

    if self.export {
      write!(f, "export ")?;
    }

    write!(f, "{} := {}", self.name, self.value)
  }
}
