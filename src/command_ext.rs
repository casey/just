use super::*;

type Environment = BTreeMap<String, Option<String>>;

fn export_scope(
  environment: &mut Environment,
  scope: &Scope,
  settings: &Settings,
  unexports: &HashSet<String>,
) {
  if let Some(parent) = scope.parent() {
    export_scope(environment, parent, settings, unexports);
  }

  for unexport in unexports {
    environment.insert(unexport.clone(), None);
  }

  for binding in scope.bindings() {
    if (binding.export || (settings.export && !binding.prelude)) && !binding.value.is_empty() {
      environment.insert(
        binding.name.lexeme().to_string(),
        Some(binding.value.join()),
      );
    }
  }
}

pub(crate) trait CommandExt {
  fn export(
    &mut self,
    settings: &Settings,
    dotenv: &BTreeMap<String, String>,
    scope: &Scope,
    unexports: &HashSet<String>,
  ) -> &mut Command;

  fn environment(
    dotenv: &BTreeMap<String, String>,
    scope: &Scope,
    settings: &Settings,
    unexports: &HashSet<String>,
  ) -> Environment {
    let mut environment = BTreeMap::new();

    for (name, value) in dotenv {
      environment.insert(name.clone(), Some(value.clone()));
    }

    if let Some(parent) = scope.parent() {
      export_scope(&mut environment, parent, settings, unexports);
    }

    environment
  }

  fn export_environment(&mut self, environment: Environment) -> &mut Command;

  fn output_guard(self) -> (io::Result<process::Output>, Option<Signal>);

  fn output_guard_stdout(self) -> Result<String, OutputError>;

  fn resolve(program: impl AsRef<OsStr>) -> Command;

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
    self.export_environment(Self::environment(dotenv, scope, settings, unexports))
  }

  fn export_environment(&mut self, environment: BTreeMap<String, Option<String>>) -> &mut Self {
    for (name, value) in environment {
      match value {
        Some(value) => self.env(name, value),
        None => self.env_remove(name),
      };
    }
    self
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

  fn resolve(program: impl AsRef<OsStr>) -> Self {
    let program = Path::new(program.as_ref());

    if !cfg!(windows) {
      return Self::new(program);
    }

    let mut candidates = vec![program.into()];

    let mut components = program.components();
    if matches!(components.next(), Some(Component::Normal(_)))
      && components.next().is_none()
      && let Some(path) = env::var_os("PATH")
    {
      for path in env::split_paths(&path) {
        candidates.push(path.join(program));
      }
    }

    let extensions = if program.extension().is_none() {
      let pathext = env::var_os("PATHEXT")
        .unwrap_or(".COM;.EXE;.BAT;.CMD".into())
        .to_string_lossy()
        .into_owned();
      let mut extensions = Vec::new();
      for extension in pathext.split(';') {
        if let Some(extension) = extension.strip_prefix('.') {
          extensions.push(extension.to_owned());
        }
      }
      Some(extensions)
    } else {
      None
    };

    for candidate in candidates {
      if let Some(extensions) = &extensions {
        for extension in extensions {
          let path = candidate.with_extension(extension);
          if path.is_file() {
            return Self::new(path);
          }
        }
      } else if candidate.is_file() {
        return Self::new(candidate);
      }
    }

    Self::new(program)
  }

  fn status_guard(self) -> (io::Result<ExitStatus>, Option<Signal>) {
    SignalHandler::spawn(self, |mut child| child.wait())
  }
}
