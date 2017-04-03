use ::prelude::*;

pub struct Platform;

pub trait PlatformInterface {
  /// Construct a command equivelant to running the script at `path` with the
  /// shebang line `shebang`
  fn make_shebang_command(path: &Path, command: &str, argument: Option<&str>) -> process::Command;

  /// Set the execute permission on the file pointed to by `path`
  fn set_execute_permission(path: &Path) -> Result<(), io::Error>;

  /// Extract the signal from a process exit status, if it was terminated by a signal
  fn signal_from_exit_status(exit_status: process::ExitStatus) -> Option<i32>;
}

#[cfg(unix)]
impl PlatformInterface for Platform {
  fn make_shebang_command(path: &Path, _command: &str, _argument: Option<&str>) -> process::Command {
    // shebang scripts can be executed directly on unix
    process::Command::new(path)
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
}

#[cfg(windows)]
impl PlatformInterface for Platform {
  fn make_shebang_command(path: &Path, command: &str, argument: Option<&str>) -> process::Command {
    let mut cmd = process::Command::new(command);
    if let Some(argument) = argument {
      cmd.arg(argument);
    }
    cmd.arg(path);
    cmd
  }

  fn set_execute_permission(_path: &Path) -> Result<(), io::Error> {
    // it is not necessary to set an execute permission on a script on windows,
    // so this is a nop
    Ok(())
  }

  fn signal_from_exit_status(_exit_status: process::ExitStatus) -> Option<i32> {
    // The rust standard library does not expose a way to extract a signal
    // from a process exit status, so just return None
    None
  }
}
