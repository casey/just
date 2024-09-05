use super::*;

#[derive(Debug, Clone)]
pub(crate) enum AttributeArgument<'src> {
  StringLiteral(StringLiteral<'src>),
  Name(Name<'src>),
}

impl<'src> From<StringLiteral<'src>> for AttributeArgument<'src> {
  fn from(value: StringLiteral<'src>) -> Self {
    Self::StringLiteral(value)
  }
}

impl<'src> From<Name<'src>> for AttributeArgument<'src> {
  fn from(value: Name<'src>) -> Self {
    Self::Name(value)
  }
}
