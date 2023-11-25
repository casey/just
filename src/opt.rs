use super::*;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub(crate) struct Opt<'src> {
  pub(crate) default: Option<Expression<'src>>,
  pub(crate) key: Name<'src>,
  pub(crate) variable: Name<'src>,
}

impl<'src> Opt<'src> {
  pub(crate) fn accepts(&self, arg: &str) -> bool {
    arg
      .strip_prefix("--")
      .map(|key| key == self.key.lexeme())
      .unwrap_or_default()
  }
}

impl<'src> ColorDisplay for Opt<'src> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> Result<(), fmt::Error> {
    write!(
      f,
      "--{} {}",
      color.annotation().paint(self.key.lexeme()),
      color.parameter().paint(self.variable.lexeme())
    )?;

    if let Some(ref default) = self.default {
      write!(f, "={}", color.string().paint(&default.to_string()))?;
    }

    Ok(())
  }
}
