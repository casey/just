use super::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)), context(suffix(false)))]
pub(crate) enum SearchError {
  #[snafu(display("I/O error at `{path}`: {source}"))]
  FilesystemIo {
    source: io::Error,
    path: Utf8PathBuf,
  },
  #[snafu(display("cannot initialize global justfile"))]
  GlobalJustfileInit,
  #[snafu(display("global justfile not found"))]
  GlobalJustfileNotFound,
  #[snafu(display("cannot use justfile from standard input with `--init`"))]
  InitWithJustfileFromStandardInput,
  #[snafu(display("justfile path had no parent: {path}"))]
  JustfileHadNoParent { path: Utf8PathBuf },
  #[snafu(display(
    "multiple candidate justfiles found in `{}`: {}",
    candidates.first().unwrap().parent().unwrap(),
    List::and_ticked(
      candidates
        .iter()
        .map(|candidate| candidate.file_name().unwrap())
    ),
  ))]
  MultipleCandidates { candidates: BTreeSet<Utf8PathBuf> },
  #[snafu(display("no justfile found"))]
  NotFound,
  #[snafu(transparent)]
  Path { source: PathError },
  #[snafu(display("error reading from standard input: {source}"))]
  StdinIo { source: io::Error },
  #[snafu(display("I/O error creating temporary directory: {source}"))]
  TempdirIo { source: io::Error },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn multiple_candidates_formatting() {
    let error = SearchError::MultipleCandidates {
      candidates: ["/foo/justfile", "/foo/JUSTFILE"]
        .into_iter()
        .map(Utf8PathBuf::from)
        .collect(),
    };

    assert_eq!(
      error.to_string(),
      "multiple candidate justfiles found in `/foo`: `JUSTFILE` and `justfile`"
    );
  }
}
