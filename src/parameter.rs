use super::*;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub(crate) struct Parameter<'src> {
  pub(crate) default: Option<Expression<'src>>,
  pub(crate) export: bool,
  pub(crate) kind: ParameterKind,
  pub(crate) long: Option<String>,
  pub(crate) name: Name<'src>,
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
