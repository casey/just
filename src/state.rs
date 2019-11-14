use crate::common::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum State<'src> {
  Normal,
  Indented { indentation: &'src str },
  Text,
  Interpolation { interpolation_start: Token<'src> },
}
