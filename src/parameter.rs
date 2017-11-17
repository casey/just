use std::fmt::{self, Display};
use color::Color;
use token::Token;

#[derive(PartialEq, Debug)]
pub struct Parameter<'a> {
  pub default:  Option<String>,
  pub name:     &'a str,
  pub token:    Token<'a>,
  pub variadic: bool,
}

impl<'a> Display for Parameter<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let color = Color::fmt(f);
    if self.variadic {
      write!(f, "{}", color.annotation().paint("+"))?;
    }
    write!(f, "{}", color.parameter().paint(self.name))?;
    if let Some(ref default) = self.default {
      let escaped = default.chars().flat_map(char::escape_default).collect::<String>();;
      write!(f, r#"='{}'"#, color.string().paint(&escaped))?;
    }
    Ok(())
  }
}

