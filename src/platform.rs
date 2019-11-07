use crate::common::*;

pub(crate) struct Platform;

#[cfg(unix)]
impl PlatformInterface for Platform {
  fn make_shebang_command(
    path: &Path,
    working_directory: &Path,
    _command: &str,
    _argument: Option<&str>,
  ) -> Result<Command, OutputError> {
    // shebang scripts can be executed directly on unix
    let mut cmd = Command::new(path);

    cmd.current_dir(working_directory);

    Ok(cmd)
  }

  fn set_execute_permission(path: &Path) -> Result<(), io::Error> {
    use std::os::unix::fs::PermissionsExt;

    // get current permissions
    let mut permissions = fs::metadata(&path)?.permissions();

    // set the execute bit
    let current_mode = permissions.mode();
    permissions.set_mode(current_mode | 0o100);

    // set the new permissions
    fs::set_permissions(&path, permissions)
  }

  fn signal_from_exit_status(exit_status: process::ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    exit_status.signal()
  }

  fn to_shell_path(_working_directory: &Path, path: &Path) -> Result<String, String> {
    path
      .to_str()
      .map(str::to_string)
      .ok_or_else(|| String::from("Error getting current directory: unicode decode error"))
  }
}

#[cfg(windows)]
impl PlatformInterface for Platform {
  fn make_shebang_command(
    path: &Path,
    working_directory: &Path,
    command: &str,
    argument: Option<&str>,
  ) -> Result<Command, OutputError> {
    // Translate path to the interpreter from unix style to windows style
    let mut cygpath = Command::new("cygpath");
    cygpath.current_dir(working_directory);
    cygpath.arg("--windows");
    cygpath.arg(command);

    let mut cmd = Command::new(output(cygpath)?);

    cmd.current_dir(working_directory);

    if let Some(argument) = argument {
      cmd.arg(argument);
    }
    cmd.arg(path);
    Ok(cmd)
  }

  fn set_execute_permission(_path: &Path) -> Result<(), io::Error> {
    // it is not necessary to set an execute permission on a script on windows,
    // so this is a nop
    Ok(())
  }

  fn signal_from_exit_status(_exit_status: process::ExitStatus) -> Option<i32> {
    // The rust standard library does not expose a way to extract a signal
    // from a windows process exit status, so just return None
    None
  }

  fn to_shell_path(working_directory: &Path, path: &Path) -> Result<String, String> {
    // Translate path from windows style to unix style
    let mut cygpath = Command::new("cygpath");
    cygpath.current_dir(working_directory);
    cygpath.arg("--unix");
    cygpath.arg(path);
    output(cygpath).map_err(|e| format!("Error converting shell path: {}", e))
  }
}
