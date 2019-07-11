use crate::common::*;

#[derive(Debug)]
pub enum OutputError {
  /// Non-zero exit code
  Code(i32),
  /// IO error
  Io(io::Error),
  /// Terminated by signal
  Signal(i32),
  /// Unknown failure
  Unknown,
  /// Stdout not UTF-8
  Utf8(std::str::Utf8Error),
}

impl Display for OutputError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      OutputError::Code(code) => write!(f, "Process exited with status code {}", code),
      OutputError::Io(ref io_error) => write!(f, "Error executing process: {}", io_error),
      OutputError::Signal(signal) => write!(f, "Process terminated by signal {}", signal),
      OutputError::Unknown => write!(f, "Process experienced an unknown failure"),
      OutputError::Utf8(ref err) => write!(f, "Could not convert process stdout to UTF-8: {}", err),
    }
  }
}

