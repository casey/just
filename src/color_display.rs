use crate::common::*;

pub(crate) trait ColorDisplay {
  fn color_display<'a>(&'a self, color: Color) -> Wrapper<'a>
  where
    Self: Sized,
  {
    Wrapper(self, color)
  }

  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result;
}

pub(crate) struct Wrapper<'a>(&'a dyn ColorDisplay, Color);

impl<'a> Display for Wrapper<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.0.fmt(f, self.1)
  }
}
