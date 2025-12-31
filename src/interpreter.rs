use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Interpreter {
  pub(crate) arguments: Vec<StringLiteral>,
  pub(crate) command: StringLiteral,
}

impl Interpreter {
  pub(crate) fn default_script_interpreter() -> &'static Interpreter {
    static INSTANCE: LazyLock<Interpreter> = LazyLock::new(|| Interpreter {
      arguments: vec![StringLiteral::from_raw("-eu")],
      command: StringLiteral::from_raw("sh"),
    });
    &INSTANCE
  }
}

impl Display for Interpreter {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.command)?;

    for argument in &self.arguments {
      write!(f, ", {argument}")?;
    }

    Ok(())
  }
}
