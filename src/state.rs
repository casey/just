#[derive(Copy, Clone, PartialEq, Debug)]
pub enum State<'a> {
  Start,
  Indent { indentation: &'a str },
  Text,
  Interpolation,
}
