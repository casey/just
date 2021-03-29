use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Warning {}

impl Warning {
  fn context(&self) -> Option<&Token> {
    #![allow(clippy::unused_self)]
    unreachable!()
  }
}

impl Display for Warning {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let warning = Color::fmt(f).warning();
    let message = Color::fmt(f).message();

    write!(f, "{} {}", warning.paint("warning:"), message.prefix())?;

    write!(f, "{}", message.suffix())?;

    if let Some(token) = self.context() {
      writeln!(f)?;
      token.write_context(f, Color::fmt(f).warning())?;
    }

    Ok(())
  }
}
