use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Interpreter<T = StringLiteral> {
  pub(crate) arguments: Vec<T>,
  pub(crate) command: T,
}

impl Interpreter {
  pub(crate) fn default_script_interpreter() -> &'static Interpreter<String> {
    static INSTANCE: LazyLock<Interpreter<String>> = LazyLock::new(|| Interpreter::<String> {
      arguments: vec!["-eu".into()],
      command: "sh".into(),
    });
    &INSTANCE
  }
}

impl<T: Display> Display for Interpreter<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.command)?;

    for argument in &self.arguments {
      write!(f, ", {argument}")?;
    }

    Ok(())
  }
}
