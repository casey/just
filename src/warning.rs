use crate::common::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Warning {}

impl Warning {
  #[allow(clippy::unused_self)]
  fn context(&self) -> Option<&Token> {
    None
  }
}

impl ColorDisplay for Warning {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    let warning = color.warning();
    let message = color.message();

    write!(f, "{} {}", warning.paint("warning:"), message.prefix())?;

    write!(f, "{}", message.suffix())?;

    if let Some(token) = self.context() {
      writeln!(f)?;
      write!(f, "{}", token.color_display(color))?;
    }

    Ok(())
  }
}

impl Serialize for Warning {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(None)?;

    map.serialize_entry("message", &self.color_display(Color::never()).to_string())?;

    map.end()
  }
}
