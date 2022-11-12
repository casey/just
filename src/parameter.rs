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
}

impl<'src> Parameter<'src> {
  pub(crate) fn format_eval<'this, 'eval, 'run>(
    &'this self,
    evaluator: &'eval std::cell::RefCell<Evaluator<'src, 'run>>,
  ) -> FmtEval<'this, 'eval, 'src, 'run> {
    FmtEval(self, evaluator)
  }
}

pub struct FmtEval<'param, 'eval, 'src, 'run>(
  &'param Parameter<'src>,
  &'eval std::cell::RefCell<Evaluator<'src, 'run>>,
);

impl<'this, 'eval, 'src, 'run> ColorDisplay for FmtEval<'this, 'eval, 'src, 'run> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    if self.0.export {
      write!(f, "$")?;
    }
    if let Some(prefix) = self.0.kind.prefix() {
      write!(f, "{}", color.annotation().paint(prefix))?;
    }
    write!(f, "{}", color.parameter().paint(self.0.name.lexeme()))?;
    if let Some(ref default) = self.0.default {
      write!(
        f,
        "={}",
        color.string().paint(&format!(
          "'{}'",
          self.1.borrow_mut().evaluate_expression(default).unwrap()
        ))
      )?;
    }
    Ok(())
  }
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
