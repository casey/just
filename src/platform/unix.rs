use super::*;

impl PlatformInterface for Platform {
  fn make_shebang_command(
    path: &Path,
    working_directory: Option<&Path>,
    _shebang: Shebang,
  ) -> Result<Command, OutputError> {
    // shebang scripts can be executed directly on unix
    let mut command = Command::new(path);

    if let Some(working_directory) = working_directory {
      command.current_dir(working_directory);
    }

    Ok(command)
  }

  fn set_execute_permission(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // get current permissions
    let mut permissions = fs::metadata(path)?.permissions();

    // set the execute bit
    let current_mode = permissions.mode();
    permissions.set_mode(current_mode | 0o100);

    // set the new permissions
    fs::set_permissions(path, permissions)
  }

  fn signal_from_exit_status(exit_status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    exit_status.signal()
  }

  fn convert_native_path(_working_directory: &Path, path: &Path) -> FunctionResult {
    path
      .to_str()
      .map(str::to_string)
      .ok_or_else(|| String::from("Error getting current directory: unicode decode error"))
  }

  fn install_signal_handler<T: Fn(Signal) + Send + 'static>(handler: T) -> RunResult<'static> {
    let signals = crate::signals::Signals::new()?;

    std::thread::Builder::new()
      .name("signal handler".into())
      .spawn(move || {
        for signal in signals {
          match signal {
            Ok(signal) => handler(signal),
            Err(io_error) => eprintln!("warning: I/O error reading from signal pipe: {io_error}"),
          }
        }
      })
      .map_err(|io_error| Error::SignalHandlerSpawnThread { io_error })?;

    Ok(())
  }
}
