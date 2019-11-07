use crate::common::*;

pub(crate) trait PlatformInterface {
  /// Construct a command equivalent to running the script at `path` with the
  /// shebang line `shebang`
  fn make_shebang_command(
    path: &Path,
    working_directory: &Path,
    command: &str,
    argument: Option<&str>,
  ) -> Result<Command, OutputError>;

  /// Set the execute permission on the file pointed to by `path`
  fn set_execute_permission(path: &Path) -> Result<(), io::Error>;

  /// Extract the signal from a process exit status, if it was terminated by a signal
  fn signal_from_exit_status(exit_status: process::ExitStatus) -> Option<i32>;

  /// Translate a path from a "native" path to a path the interpreter expects
  fn to_shell_path(working_directory: &Path, path: &Path) -> Result<String, String>;
}
