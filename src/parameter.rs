use super::*;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub(crate) struct Parameter<'src> {
  pub(crate) default: Option<Expression<'src>>,
  pub(crate) export: bool,
  pub(crate) flag: bool,
  pub(crate) help: Option<String>,
  pub(crate) kind: ParameterKind,
  pub(crate) long: Option<String>,
  pub(crate) max: Option<u64>,
  pub(crate) min: Option<u64>,
  pub(crate) multiple: bool,
  pub(crate) name: Name<'src>,
  #[serde(skip)]
  pub(crate) number: Number,
  pub(crate) pattern: Option<Pattern>,
  pub(crate) short: Option<char>,
  pub(crate) value: Option<Expression<'src>>,
}

impl<'src> Parameter<'src> {
  pub(crate) fn is_option(&self) -> bool {
    self.long.is_some() || self.short.is_some()
  }

  pub(crate) fn is_required(&self) -> bool {
    self.default.is_none()
      && !self.flag
      && (self.kind != ParameterKind::Star || self.min.is_some_and(|min| min > 0))
  }

  pub(crate) fn check_value_count(
    &self,
    recipe: &Recipe<'src>,
    value: &Value,
  ) -> Result<(), Error<'src>> {
    let found = value.elements().len();

    if let Some(min) = self.min
      && u64::try_from(found).unwrap() < min
    {
      return Err(Error::ArgumentTooFewValues {
        recipe: recipe.name(),
        parameter: self.name.lexeme(),
        found,
        min,
      });
    }

    if let Some(max) = self.max
      && u64::try_from(found).unwrap() > max
    {
      return Err(Error::ArgumentTooManyValues {
        recipe: recipe.name(),
        parameter: self.name.lexeme(),
        found,
        max,
      });
    }

    Ok(())
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
      pattern: Box::new(pattern.clone()),
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
    if let Some(default) = &self.default {
      write!(f, "={}", color.string().paint(&default.to_string()))?;
    }
    Ok(())
  }
}
