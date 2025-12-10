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
  /// The parameter pattern
  pub(crate) pattern: Option<Pattern>,
}

impl<'src> Parameter<'src> {
  pub(crate) fn is_required(&self) -> bool {
    self.default.is_none() && self.kind != ParameterKind::Star
  }

  pub(crate) fn check_pattern_match(
    &self,
    recipe: &Recipe<'src>,
    value: &str,
  ) -> Result<(), Error<'src>> {
    let Some(pattern) = &self.pattern else {
      return Ok(());
    };

    if pattern.is_match(value) {
      return Ok(());
    }

    Err(Error::ArgumentPatternMismatch {
      argument: value.into(),
      parameter: self.name.lexeme(),
      pattern: pattern.clone(),
      recipe: recipe.name(),
    })
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
      write!(f, "={}", color.string().paint(&default.to_string()))?;
    }
    Ok(())
  }
}
