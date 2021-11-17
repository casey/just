use crate::common::*;

/// A single function parameter
#[derive(PartialEq, Debug, Clone, Serialize)]
pub(crate) struct Parameter<'src> {
  /// An optional default expression
  pub(crate) default: Option<Expression<'src>>,
  /// Export parameter as environment variable
  pub(crate) export: bool,
  /// The kind of parameter
  pub(crate) kind: ParameterKind,
  /// The parameter name
  pub(crate) name: Name<'src>,
}

impl<'src> ColorDisplay for Parameter<'src> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> Result<(), fmt::Error> {
    if self.export {
      write!(f, "$")?;
    }
    if let Some(prefix) = self.kind.prefix() {
      write!(f, "{}", color.annotation().paint(prefix))?;
    }
    write!(f, "{}", color.parameter().paint(self.name.lexeme()))?;
    if let Some(ref default) = self.default {
      write!(f, "={}", color.string().paint(&default.to_string()))?;
    }
    Ok(())
  }
}
