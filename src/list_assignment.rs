use super::*;

/// A list assignment, e.g `foo := bar baz`
pub(crate) type ListAssignment<'src> = ListBinding<'src, Vec<Expression<'src>>>;

impl<'src> Display for ListAssignment<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    if self.export {
      write!(f, "export ")?;
    }
    let out = &self
      .value
      .iter()
      .map(std::string::ToString::to_string)
      .collect::<Vec<_>>()
      .join(" ");

    write!(f, "{} := {}", self.name, out)
  }
}
