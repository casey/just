use super::*;

pub(crate) trait PlatformInterface {
  /// Construct a command equivalent to running the script at `path` with the
  /// shebang line `shebang`
  fn make_shebang_command(
    path: &Path,
    working_directory: Option<&Path>,
    shebang: Shebang,
  ) -> Result<Command, OutputError>;

  /// Set the execute permission on the file pointed to by `path`
  fn set_execute_permission(path: &Path) -> Result<(), io::Error>;

  /// Extract the signal from a process exit status, if it was terminated by a
  /// signal
  fn signal_from_exit_status(exit_status: ExitStatus) -> Option<i32>;

  /// Convert a signal into an exit code, for the case where a process was
  /// terminated by a signal
  /// See https://tldp.org/LDP/abs/html/exitcodes.html
  fn exit_code_from_signal(signal: i32) -> i32;

  /// Translate a path from a "native" path to a path the interpreter expects
  fn convert_native_path(working_directory: &Path, path: &Path) -> Result<String, String>;
}
