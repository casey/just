use crate::common::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum State<'a> {
  Normal,
  Indented { indentation: &'a str },
  Text,
  Interpolation { interpolation_start: Position },
}
