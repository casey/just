use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Interpreter<'src> {
  pub(crate) arguments: Vec<StringLiteral<'src>>,
  pub(crate) command: StringLiteral<'src>,
}

impl<'src> Interpreter<'src> {
  pub(crate) fn default_script_interpreter() -> &'static Interpreter<'static> {
    static INSTANCE: Lazy<Interpreter<'static>> = Lazy::new(|| Interpreter {
      arguments: vec![StringLiteral::from_raw("-eu")],
      command: StringLiteral::from_raw("sh"),
    });
    &INSTANCE
  }
}

impl<'src> Display for Interpreter<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.command)?;

    for argument in &self.arguments {
      write!(f, ", {argument}")?;
    }

    Ok(())
  }
}
