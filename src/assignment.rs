use super::*;
use crate::binding::DepthAwareBinding;
/// An assignment, e.g `foo := bar`
pub(crate) type Assignment<'src> = DepthAwareBinding<'src, Expression<'src>>;

impl<'src> Display for Assignment<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    if self.export {
      write!(f, "export ")?;
    }
    write!(f, "{} := {}", self.name, self.value)
  }
}
