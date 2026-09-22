use super::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)), context(suffix(false)))]
pub(crate) enum DatetimeFormatError {
  #[snafu(display("failed to format time with format string `{format}`"))]
  Format { format: String },
  #[snafu(display("failed to parse time format string `{format}`: {source}"))]
  Parse {
    format: String,
    source: chrono::ParseError,
  },
}
