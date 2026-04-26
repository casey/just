use super::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum SearchError {
  #[snafu(display("cannot initialize global justfile"))]
  GlobalJustfileInit,
  #[snafu(display("global justfile not found"))]
  GlobalJustfileNotFound,
  #[snafu(display(
    "I/O error reading directory `{}`: {}",
    directory.display(),
    io_error
  ))]
  Io {
    directory: PathBuf,
    io_error: io::Error,
  },
  #[snafu(display("justfile path had no parent: {}", path.display()))]
  JustfileHadNoParent { path: PathBuf },
  #[snafu(display(
    "multiple candidate justfiles found in `{}`: {}",
    candidates.iter().next().unwrap().parent().unwrap().display(),
    List::and_ticked(
      candidates
        .iter()
        .map(|candidate| candidate.file_name().unwrap().to_string_lossy())
    ),
  ))]
  MultipleCandidates { candidates: BTreeSet<PathBuf> },
  #[snafu(display("no justfile found"))]
  NotFound,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn multiple_candidates_formatting() {
    let error = SearchError::MultipleCandidates {
      candidates: [Path::new("/foo/justfile"), Path::new("/foo/JUSTFILE")]
        .iter()
        .map(|path| path.to_path_buf())
        .collect(),
    };

    assert_eq!(
      error.to_string(),
      "multiple candidate justfiles found in `/foo`: `JUSTFILE` and `justfile`"
    );
  }
}
