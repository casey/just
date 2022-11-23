use super::*;

/// A single function parameter
#[derive(PartialEq, Debug, Clone, Serialize)]
pub(crate) struct Parameter<'src, T = Expression<'src>> {
  /// An optional default expression
  pub(crate) default: Option<T>,
  /// Export parameter as environment variable
  pub(crate) export: bool,
  /// The kind of parameter
  pub(crate) kind: ParameterKind,
  /// The parameter name
  pub(crate) name: Name<'src>,
}

impl<'src> Parameter<'src> {
  pub(crate) fn evaluate_default<'run>(
    &self,
    evaluator: &mut Evaluator<'src, 'run>,
  ) -> RunResult<'src, Option<Parameter<'src, String>>> {
    if let Some(ref default) = self.default {
      Ok(Some(Parameter {
        default: Some(format!("'{}'", evaluator.evaluate_expression(default)?)),
        export: self.export,
        kind: self.kind,
        name: self.name,
      }))
    } else {
      Ok(None)
    }
  }
}

impl<'src, T: Display> ColorDisplay for Parameter<'src, T> {
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
