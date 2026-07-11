use super::*;

pub(crate) trait CommandExt {
  fn output_guard(self) -> (io::Result<process::Output>, Option<Signal>);

  fn output_guard_stdout(self) -> Result<String, OutputError>;

  fn resolve(program: impl AsRef<OsStr>) -> Command;

  fn shell_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Command;

  fn status_guard(self) -> (io::Result<ExitStatus>, Option<Signal>);
}

impl CommandExt for Command {
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

  fn shell_arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Command {
    #[cfg(windows)]
    if ShellKind::from(&*self) == ShellKind::Cmd {
      use std::os::windows::process::CommandExt;
      return self.raw_arg(arg);
    }

    self.arg(arg)
  }

  fn status_guard(self) -> (io::Result<ExitStatus>, Option<Signal>) {
    SignalHandler::spawn(self, |mut child| child.wait())
  }
}
