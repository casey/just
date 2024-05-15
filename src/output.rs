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
        Ok(output) => Ok(
          if output.ends_with("\r\n") {
            &output[0..output.len() - 2]
          } else if output.ends_with('\n') {
            &output[0..output.len() - 1]
          } else {
            output
          }
          .to_owned(),
        ),
      }
    }
    Err(io_error) => Err(OutputError::Io(io_error)),
  }
}
