use super::*;

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
  pub(crate) one_of: Option<OneOf<'src>>,
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub(crate) struct OneOf<'src> {
  pub(crate) enum_name: Name<'src>,
}

impl Parameter<'_> {
  pub(crate) fn is_required(&self) -> bool {
    self.default.is_none() && self.kind != ParameterKind::Star
  }
}

impl ColorDisplay for Parameter<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    if let Some(prefix) = self.kind.prefix() {
      write!(f, "{}", color.annotation().paint(prefix))?;
    }
    if self.export {
      write!(f, "$")?;
    }
    write!(f, "{}", color.parameter().paint(self.name.lexeme()))?;
    if let Some(ref default) = self.default {
      if let Some(ref one_of) = self.one_of {
        write!(
          f,
          "=one-of-or-default({}, {})",
          color.string().paint(&one_of.enum_name.to_string()),
          color.string().paint(&default.to_string())
        )?;
      } else {
        write!(f, "={}", color.string().paint(&default.to_string()))?;
      }
    } else if let Some(ref one_of) = self.one_of {
      write!(
        f,
        "=one-of({})",
        color.string().paint(&one_of.enum_name.to_string()),
      )?;
    }
    Ok(())
  }
}
