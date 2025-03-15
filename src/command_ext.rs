use super::*;

pub(crate) trait CommandExt {
  fn export(
    &mut self,
    settings: &Settings,
    dotenv: &BTreeMap<String, String>,
    scope: &Scope,
    unexports: &HashSet<String>,
  ) -> &mut Command;

  fn export_scope(&mut self, settings: &Settings, scope: &Scope, unexports: &HashSet<String>);

  fn output_guard(self) -> (io::Result<process::Output>, Option<Signal>);

  fn output_guard_stdout(self) -> Result<String, OutputError>;

  fn status_guard(self) -> (io::Result<ExitStatus>, Option<Signal>);
}

impl CommandExt for Command {
  fn export(
    &mut self,
    settings: &Settings,
    dotenv: &BTreeMap<String, String>,
    scope: &Scope,
    unexports: &HashSet<String>,
  ) -> &mut Command {
    for (name, value) in dotenv {
      self.env(name, value);
    }

    if let Some(parent) = scope.parent() {
      self.export_scope(settings, parent, unexports);
    }

    self
  }

  fn export_scope(&mut self, settings: &Settings, scope: &Scope, unexports: &HashSet<String>) {
    if let Some(parent) = scope.parent() {
      self.export_scope(settings, parent, unexports);
    }

    for unexport in unexports {
      self.env_remove(unexport);
    }

    for binding in scope.bindings() {
      if binding.export || (settings.export && !binding.constant) {
        self.env(binding.name.lexeme(), &binding.value);
      }
    }
  }

  fn output_guard(self) -> (io::Result<process::Output>, Option<Signal>) {
    SignalHandler::spawn(self, process::Child::wait_with_output)
  }

  fn output_guard_stdout(self) -> Result<String, OutputError> {
    let (result, caught) = self.output_guard();

    let output = result.map_err(OutputError::Io)?;

    OutputError::result_from_exit_status(output.status)?;

    let output = str::from_utf8(&output.stdout).map_err(OutputError::Utf8)?;

    if let Some(signal) = caught {
      return Err(OutputError::Interrupted(signal));
    }

    Ok(
      output
        .strip_suffix("\r\n")
        .or_else(|| output.strip_suffix("\n"))
        .unwrap_or(output)
        .into(),
    )
  }

  fn status_guard(self) -> (io::Result<ExitStatus>, Option<Signal>) {
    SignalHandler::spawn(self, |mut child| child.wait())
  }
}
