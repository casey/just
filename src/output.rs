use super::*;

/// Run a command and return the data it wrote to stdout as a string
pub(crate) fn output(mut command: Command) -> Result<String, OutputError> {
  match command.output() {
    Ok(output) => {
      if let Some(code) = output.status.code() {
        if code != 0 {
          return Err(OutputError::Code(code));
        }
      } else {
        let signal = Platform::signal_from_exit_status(output.status);
        return Err(match signal {
          Some(signal) => OutputError::Signal(signal),
          None => OutputError::Unknown,
        });
      }
      match str::from_utf8(&output.stdout) {
        Err(error) => Err(OutputError::Utf8(error)),
        Ok(utf8) => Ok(
          if utf8.ends_with('\n') {
            &utf8[0..utf8.len() - 1]
          } else if utf8.ends_with('\r') {
            &utf8[0..utf8.len() - 1]
          } else if utf8.ends_with("\r\n") {
            &utf8[0..utf8.len() - 2]
          } else {
            utf8
          }
          .to_owned(),
        ),
      }
    }
    Err(io_error) => Err(OutputError::Io(io_error)),
  }
}
