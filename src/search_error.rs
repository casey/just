use super::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum SearchError {
  #[snafu(display(
    "I/O error reading directory `{}`: {}",
    directory.display(),
    io_error
  ))]
  Io {
    directory: PathBuf,
    io_error: io::Error,
  },
  MalformedPath { path: PathBuf, err: io::Error },
  #[snafu(display("Justfile path had no parent: {}", path.display()))]
  JustfileHadNoParent { path: PathBuf },
  #[snafu(display(
    "Multiple candidate justfiles found in `{}`: {}",
    candidates.iter().next().unwrap().parent().unwrap().display(),
    List::and_ticked(
      candidates
        .iter()
        .map(|candidate| candidate.file_name().unwrap().to_string_lossy())
    ),
  ))]
  MultipleCandidates { candidates: Vec<PathBuf> },
  #[snafu(display("No justfile found"))]
  NotFound,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn multiple_candidates_formatting() {
    let error = SearchError::MultipleCandidates {
      candidates: [Path::new("/foo/JUSTFILE"), Path::new("/foo/justfile")]
        .iter()
        .map(|path| path.to_path_buf())
        .collect(),
    };

    assert_eq!(
      error.to_string(),
      "Multiple candidate justfiles found in `/foo`: `JUSTFILE` and `justfile`"
    );
  }
}
