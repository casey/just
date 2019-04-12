use crate::common::*;

#[derive(PartialEq, Debug)]
pub struct Parameter<'a> {
  pub default: Option<Expression<'a>>,
  pub name: &'a str,
  pub token: Token<'a>,
  pub variadic: bool,
}

impl<'a> Display for Parameter<'a> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    let color = Color::fmt(f);
    if self.variadic {
      write!(f, "{}", color.annotation().paint("+"))?;
    }
    write!(f, "{}", color.parameter().paint(self.name))?;
    if let Some(ref default) = self.default {
      write!(f, "={}", color.string().paint(&default.to_string()))?;
    }
    Ok(())
  }
}
