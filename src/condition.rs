use super::*;

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Condition<'src> {
  pub(crate) lhs: Box<Expression<'src>>,
  pub(crate) rhs: Box<Expression<'src>>,
  pub(crate) operator: ConditionalOperator,
}

impl<'src> Display for Condition<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} {} {}", self.lhs, self.operator, self.rhs)
  }
}

impl<'src> Serialize for Condition<'src> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut seq = serializer.serialize_seq(None)?;
    seq.serialize_element(&self.operator.to_string())?;
    seq.serialize_element(&self.lhs)?;
    seq.serialize_element(&self.rhs)?;
    seq.end()
  }
}
