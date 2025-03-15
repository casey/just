use super::*;

impl PlatformInterface for Platform {
  fn make_shebang_command(
    path: &Path,
    working_directory: Option<&Path>,
    shebang: Shebang,
  ) -> Result<Command, OutputError> {
    use std::borrow::Cow;

    // If the path contains forward slashes…
    let command = if shebang.interpreter.contains('/') {
      // …translate path to the interpreter from unix style to windows style.
      let mut cygpath = Command::new("cygpath");

      if let Some(working_directory) = working_directory {
        cygpath.current_dir(working_directory);
      }

      cygpath
        .arg("--windows")
        .arg(shebang.interpreter)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

      Cow::Owned(cygpath.output_guard_stdout()?)
    } else {
      // …otherwise use it as-is.
      Cow::Borrowed(shebang.interpreter)
    };

    let mut cmd = Command::new(command.as_ref());

    if let Some(working_directory) = working_directory {
      cmd.current_dir(working_directory);
    }

    if let Some(argument) = shebang.argument {
      cmd.arg(argument);
    }

    cmd.arg(path);
    Ok(cmd)
  }

  fn set_execute_permission(_path: &Path) -> io::Result<()> {
    // it is not necessary to set an execute permission on a script on windows, so
    // this is a nop
    Ok(())
  }

  fn signal_from_exit_status(_exit_status: process::ExitStatus) -> Option<i32> {
    // The rust standard library does not expose a way to extract a signal from a
    // windows process exit status, so just return None
    None
  }

  fn convert_native_path(working_directory: &Path, path: &Path) -> FunctionResult {
    // Translate path from windows style to unix style
    let mut cygpath = Command::new("cygpath");

    cygpath
      .current_dir(working_directory)
      .arg("--unix")
      .arg(path)
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped());

    match cygpath.output_guard_stdout() {
      Ok(shell_path) => Ok(shell_path),
      Err(_) => path
        .to_str()
        .map(str::to_string)
        .ok_or_else(|| String::from("Error getting current directory: unicode decode error")),
    }
  }

  fn install_signal_handler<T: Fn(Signal) + Send + 'static>(handler: T) -> RunResult<'static> {
    ctrlc::set_handler(move || handler(Signal::Interrupt))
      .map_err(|source| Error::SignalHandlerInstall { source })
  }
}
