use crate::common::*;

const FILENAME: &str = "justfile";

pub(crate) struct Search {
  pub(crate) justfile: PathBuf,
  pub(crate) working_directory: PathBuf,
}

impl Search {
  pub(crate) fn search(
    search_config: &SearchConfig,
    invocation_directory: &Path,
  ) -> SearchResult<Search> {
    match search_config {
      SearchConfig::FromInvocationDirectory => {
        let justfile = Self::justfile(&invocation_directory)?;

        let working_directory = Self::working_directory_from_justfile(&justfile)?;

        Ok(Search {
          justfile,
          working_directory,
        })
      }

      SearchConfig::FromSearchDirectory { search_directory } => {
        let justfile = Self::justfile(search_directory)?;

        let working_directory = Self::working_directory_from_justfile(&justfile)?;

        Ok(Search {
          justfile,
          working_directory,
        })
      }

      SearchConfig::WithJustfile { justfile } => {
        let justfile: PathBuf = justfile.to_path_buf();

        let working_directory = Self::working_directory_from_justfile(&justfile)?;

        Ok(Search {
          justfile,
          working_directory,
        })
      }

      SearchConfig::WithJustfileAndWorkingDirectory {
        justfile,
        working_directory,
      } => Ok(Search {
        justfile: justfile.to_path_buf(),
        working_directory: working_directory.to_path_buf(),
      }),
    }
  }

  fn justfile(directory: &Path) -> SearchResult<PathBuf> {
    let mut candidates = Vec::new();
    let entries = fs::read_dir(directory).map_err(|io_error| SearchError::Io {
      io_error,
      directory: directory.to_owned(),
    })?;
    for entry in entries {
      let entry = entry.map_err(|io_error| SearchError::Io {
        io_error,
        directory: directory.to_owned(),
      })?;
      if let Some(name) = entry.file_name().to_str() {
        if name.eq_ignore_ascii_case(FILENAME) {
          candidates.push(entry.path());
        }
      }
    }
    if candidates.len() == 1 {
      Ok(candidates.pop().unwrap())
    } else if candidates.len() > 1 {
      Err(SearchError::MultipleCandidates { candidates })
    } else if let Some(parent) = directory.parent() {
      Self::justfile(parent)
    } else {
      Err(SearchError::NotFound)
    }
  }

  fn working_directory_from_justfile(justfile: &Path) -> SearchResult<PathBuf> {
    let justfile_canonical = justfile
      .canonicalize()
      .context(search_error::Canonicalize { path: justfile })?;

    Ok(
      justfile_canonical
        .parent()
        .ok_or_else(|| SearchError::JustfileHadNoParent {
          path: justfile_canonical.clone(),
        })?
        .to_owned(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn not_found() {
    let tmp = testing::tempdir();
    match Search::justfile(tmp.path()) {
      Err(SearchError::NotFound) => {
        assert!(true);
      }
      _ => panic!("No justfile found error was expected"),
    }
  }

  #[test]
  fn multiple_candidates() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    path.push(FILENAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push(FILENAME.to_uppercase());
    if let Ok(_) = fs::File::open(path.as_path()) {
      // We are in case-insensitive file system
      return;
    }
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    match Search::justfile(path.as_path()) {
      Err(SearchError::MultipleCandidates { .. }) => {
        assert!(true);
      }
      _ => panic!("Multiple candidates error was expected"),
    }
  }

  #[test]
  fn found() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    path.push(FILENAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    match Search::justfile(path.as_path()) {
      Ok(_path) => {
        assert!(true);
      }
      _ => panic!("No errors were expected"),
    }
  }

  #[test]
  fn found_spongebob_case() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    let spongebob_case = FILENAME
      .chars()
      .enumerate()
      .map(|(i, c)| {
        if i % 2 == 0 {
          c.to_ascii_uppercase()
        } else {
          c
        }
      })
      .collect::<String>();
    path.push(spongebob_case);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    match Search::justfile(path.as_path()) {
      Ok(_path) => {
        assert!(true);
      }
      _ => panic!("No errors were expected"),
    }
  }

  #[test]
  fn found_from_inner_dir() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    path.push(FILENAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push("a");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    path.push("b");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    match Search::justfile(path.as_path()) {
      Ok(_path) => {
        assert!(true);
      }
      _ => panic!("No errors were expected"),
    }
  }

  #[test]
  fn found_and_stopped_at_first_justfile() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    path.push(FILENAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push("a");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    path.push(FILENAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push("b");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    match Search::justfile(path.as_path()) {
      Ok(found_path) => {
        path.pop();
        path.push(FILENAME);
        assert_eq!(found_path, path);
      }
      _ => panic!("No errors were expected"),
    }
  }
}
