use std::path::PathBuf;
use std::{fmt, io};

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

impl fmt::Display for SearchError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      SearchError::Io {
        directory,
        io_error,
      } => write!(
        f,
        "IO error has occurred while operating on directory {:#?}: {}",
        directory, io_error
      ),
      SearchError::MultipleCandidates { candidates } => {
        write!(f, "Multiple justfiles found: {:#?}", candidates)
      }
      SearchError::NotFound => write!(f, "No justfile found"),
    }
  }
}
