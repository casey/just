use std::io;
use std::path::PathBuf;

pub enum SearchError {
  MultipleCandidates {
    candidates: Vec<PathBuf>,
  },
  Io {
    directory: PathBuf,
    io_error: io::Error,
  },
  NotFound,
}
