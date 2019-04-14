#[derive(PartialEq)]
pub enum State<'a> {
  Start,
  Indent(&'a str),
  Text,
  Interpolation,
}
