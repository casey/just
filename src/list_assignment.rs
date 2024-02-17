use super::*;

/// A list assignment, e.g `foo := bar baz`
pub(crate) type ListAssignment<'src> = ListBinding<'src, Vec<Expression<'src>>>;

impl<'src> Display for ListAssignment<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    if self.export {
      write!(f, "export ")?;
    }
    let mut out = String::new();
    for e in &self.value {
      out.push_str(e.to_string().as_str());
      out.push(' ');
    }
    write!(f, "{} := {}", self.name, out)
  }
}
