use super::*;

#[derive(Debug)]
pub(crate) enum DatetimeFormatError {
  Format {
    format: String,
  },
  Parse {
    format: String,
    source: chrono::ParseError,
  },
}

impl Display for DatetimeFormatError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Format { format } => {
        write!(f, "failed to format time with format string `{format}`")
      }
      Self::Parse { source, format } => {
        write!(f, "failed to parse time format string `{format}`: {source}")
      }
    }
  }
}
