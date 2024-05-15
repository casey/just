use super::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct OrderedList<'src> {
  pub(crate) list: Vec<StringLiteral<'src>>,
}

impl<'src> Display for OrderedList<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(f, "[")?;
    write!(
      f,
      "{}",
      self
        .list
        .iter()
        .map(|v| v.cooked.as_ref())
        .collect::<Vec<&str>>()
        .join(", ")
    )?;
    write!(f, "]")
  }
}
