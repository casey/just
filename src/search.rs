use crate::common::*;

use std::path::Component;

pub(crate) const FILENAME: &str = "justfile";
const PROJECT_ROOT_CHILDREN: &[&str] = &[".bzr", ".git", ".hg", ".svn", "_darcs"];

pub(crate) struct Search {
  pub(crate) justfile:          PathBuf,
  pub(crate) working_directory: PathBuf,
}

impl Search {
  pub(crate) fn find(
    search_config: &SearchConfig,
    invocation_directory: &Path,
  ) -> SearchResult<Self> {
    match search_config {
      SearchConfig::FromInvocationDirectory => {
        let justfile = Self::justfile(&invocation_directory)?;

        let working_directory = Self::working_directory_from_justfile(&justfile)?;

        Ok(Self {
          justfile,
          working_directory,
        })
      },

      SearchConfig::FromSearchDirectory { search_directory } => {
        let search_directory = Self::clean(invocation_directory, search_directory);

        let justfile = Self::justfile(&search_directory)?;

        let working_directory = Self::working_directory_from_justfile(&justfile)?;

        Ok(Self {
          justfile,
          working_directory,
        })
      },

      SearchConfig::WithJustfile { justfile } => {
        let justfile = Self::clean(invocation_directory, justfile);

        let working_directory = Self::working_directory_from_justfile(&justfile)?;

        Ok(Self {
          justfile,
          working_directory,
        })
      },

      SearchConfig::WithJustfileAndWorkingDirectory {
        justfile,
        working_directory,
      } => Ok(Self {
        justfile:          Self::clean(invocation_directory, justfile),
        working_directory: Self::clean(invocation_directory, working_directory),
      }),
    }
  }

  pub(crate) fn init(
    search_config: &SearchConfig,
    invocation_directory: &Path,
  ) -> SearchResult<Self> {
    match search_config {
      SearchConfig::FromInvocationDirectory => {
        let working_directory = Self::project_root(&invocation_directory)?;

        let justfile = working_directory.join(FILENAME);

        Ok(Self {
          justfile,
          working_directory,
        })
      },

      SearchConfig::FromSearchDirectory { search_directory } => {
        let search_directory = Self::clean(invocation_directory, search_directory);

        let working_directory = Self::project_root(&search_directory)?;

        let justfile = working_directory.join(FILENAME);

        Ok(Self {
          justfile,
          working_directory,
        })
      },

      SearchConfig::WithJustfile { justfile } => {
        let justfile = Self::clean(invocation_directory, justfile);

        let working_directory = Self::working_directory_from_justfile(&justfile)?;

        Ok(Self {
          justfile,
          working_directory,
        })
      },

      SearchConfig::WithJustfileAndWorkingDirectory {
        justfile,
        working_directory,
      } => Ok(Self {
        justfile:          Self::clean(invocation_directory, justfile),
        working_directory: Self::clean(invocation_directory, working_directory),
      }),
    }
  }

  fn justfile(directory: &Path) -> SearchResult<PathBuf> {
    for directory in directory.ancestors() {
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
        return Ok(candidates.pop().unwrap());
      } else if candidates.len() > 1 {
        return Err(SearchError::MultipleCandidates { candidates });
      }
    }

    Err(SearchError::NotFound)
  }

  fn clean(invocation_directory: &Path, path: &Path) -> PathBuf {
    let path = invocation_directory.join(path);

    let mut clean = Vec::new();

    for component in path.components() {
      if component == Component::ParentDir {
        if let Some(Component::Normal(_)) = clean.last() {
          clean.pop();
        }
      } else {
        clean.push(component);
      }
    }

    clean.into_iter().collect()
  }

  fn project_root(directory: &Path) -> SearchResult<PathBuf> {
    for directory in directory.ancestors() {
      let entries = fs::read_dir(directory).map_err(|io_error| SearchError::Io {
        io_error,
        directory: directory.to_owned(),
      })?;

      for entry in entries {
        let entry = entry.map_err(|io_error| SearchError::Io {
          io_error,
          directory: directory.to_owned(),
        })?;
        for project_root_child in PROJECT_ROOT_CHILDREN.iter().cloned() {
          if entry.file_name() == project_root_child {
            return Ok(directory.to_owned());
          }
        }
      }
    }

    Ok(directory.to_owned())
  }

  fn working_directory_from_justfile(justfile: &Path) -> SearchResult<PathBuf> {
    Ok(
      justfile
        .parent()
        .ok_or_else(|| SearchError::JustfileHadNoParent {
          path: justfile.to_path_buf(),
        })?
        .to_owned(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utilities::tmptree;

  #[test]
  fn not_found() {
    let tmp = testing::tempdir();
    match Search::justfile(tmp.path()) {
      Err(SearchError::NotFound) => {},
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
    if fs::File::open(path.as_path()).is_ok() {
      // We are in case-insensitive file system
      return;
    }
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    match Search::justfile(path.as_path()) {
      Err(SearchError::MultipleCandidates { .. }) => {},
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
    if let Err(err) = Search::justfile(path.as_path()) {
      panic!("No errors were expected: {}", err);
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
    if let Err(err) = Search::justfile(path.as_path()) {
      panic!("No errors were expected: {}", err);
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
    if let Err(err) = Search::justfile(path.as_path()) {
      panic!("No errors were expected: {}", err);
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
      },
      Err(err) => panic!("No errors were expected: {}", err),
    }
  }

  #[test]
  fn justfile_symlink_parent() {
    let tmp = tmptree! {
      src: "",
      sub: {},
    };

    let src = tmp.path().join("src");
    let sub = tmp.path().join("sub");
    let justfile = sub.join("justfile");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&src, &justfile).unwrap();

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&src, &justfile).unwrap();

    let search_config = SearchConfig::FromInvocationDirectory;

    let search = Search::find(&search_config, &sub).unwrap();

    assert_eq!(search.justfile, justfile);
    assert_eq!(search.working_directory, sub);
  }

  #[test]
  fn clean() {
    let cases = &[
      ("/", "foo", "/foo"),
      ("/bar", "/foo", "/foo"),
      ("//foo", "bar//baz", "/foo/bar/baz"),
      ("/", "..", "/"),
      ("/", "/..", "/"),
      ("/..", "", "/"),
      ("/../../../..", "../../../", "/"),
      ("/.", "./", "/"),
      ("/foo/../", "bar", "/bar"),
      ("/foo/bar", "..", "/foo"),
      ("/foo/bar/", "..", "/foo"),
    ];

    for (prefix, suffix, want) in cases {
      let have = Search::clean(Path::new(prefix), Path::new(suffix));
      assert_eq!(have, Path::new(want));
    }
  }
}
