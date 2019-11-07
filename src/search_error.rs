use crate::common::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum SearchError {
  #[snafu(display(
    "Multiple candidate justfiles found in `{}`: {}",
    candidates[0].parent().unwrap().display(),
    List::and_ticked(
      candidates
        .iter()
        .map(|candidate| candidate.file_name().unwrap().to_string_lossy())
    ),
  ))]
  MultipleCandidates {
    candidates: Vec<PathBuf>,
  },
  #[snafu(display(
    "I/O error reading directory `{}`: {}",
    directory.display(),
    io_error
  ))]
  Io {
    directory: PathBuf,
    io_error: io::Error,
  },
  #[snafu(display("No justfile found"))]
  NotFound,
  Canonicalize {
    path: PathBuf,
    source: io::Error,
  },
  JustfileHadNoParent {
    path: PathBuf,
  },
}

impl Error for SearchError {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn multiple_candidates_formatting() {
    let error = SearchError::MultipleCandidates {
      candidates: vec![
        PathBuf::from("/foo/justfile"),
        PathBuf::from("/foo/JUSTFILE"),
      ],
    };

    assert_eq!(
      error.to_string(),
      "Multiple candidate justfiles found in `/foo`: `justfile` and `JUSTFILE`"
    )
  }
}
