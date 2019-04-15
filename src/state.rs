#[derive(Copy, Clone, PartialEq, Debug)]
pub enum State<'a> {
  Normal,
  Indented { indentation: &'a str },
  Text,
  Interpolation,
}
