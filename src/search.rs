use {super::*, std::path::Component};

const DEFAULT_JUSTFILE_NAME: &str = JUSTFILE_NAMES[0];
pub(crate) const JUSTFILE_NAMES: [&str; 2] = ["justfile", ".justfile"];
const PROJECT_ROOT_CHILDREN: &[&str] = &[".bzr", ".git", ".hg", ".svn", "_darcs"];

#[derive(Debug)]
pub(crate) struct Search {
  pub(crate) justfile: PathBuf,
  pub(crate) working_directory: PathBuf,
}

impl Search {
  fn global_justfile_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(config_dir) = dirs::config_dir() {
      paths.push(config_dir.join("just").join(DEFAULT_JUSTFILE_NAME));
    }

    if let Some(home_dir) = dirs::home_dir() {
      paths.push(
        home_dir
          .join(".config")
          .join("just")
          .join(DEFAULT_JUSTFILE_NAME),
      );

      for justfile_name in JUSTFILE_NAMES {
        paths.push(home_dir.join(justfile_name));
      }
    }

    paths
  }

  /// Find justfile given search configuration and invocation directory
  pub(crate) fn find(
    search_config: &SearchConfig,
    invocation_directory: &Path,
  ) -> SearchResult<Self> {
    match search_config {
      SearchConfig::FromInvocationDirectory => Self::find_in_directory(invocation_directory),
      SearchConfig::FromSearchDirectory { search_directory } => {
        let search_directory = Self::clean(invocation_directory, search_directory);
        let justfile = Self::justfile(&search_directory)?;
        let working_directory = Self::working_directory_from_justfile(&justfile)?;
        Ok(Self {
          justfile,
          working_directory,
        })
      }
      SearchConfig::GlobalJustfile => Ok(Self {
        justfile: Self::global_justfile_paths()
          .iter()
          .find(|path| path.exists())
          .cloned()
          .ok_or(SearchError::GlobalJustfileNotFound)?,
        working_directory: Self::project_root(invocation_directory)?,
      }),
      SearchConfig::WithJustfile { justfile } => {
        let justfile = Self::clean(invocation_directory, justfile);
        let working_directory = Self::working_directory_from_justfile(&justfile)?;
        Ok(Self {
          justfile,
          working_directory,
        })
      }
      SearchConfig::WithJustfileAndWorkingDirectory {
        justfile,
        working_directory,
      } => Ok(Self {
        justfile: Self::clean(invocation_directory, justfile),
        working_directory: Self::clean(invocation_directory, working_directory),
      }),
    }
  }

  /// Find justfile starting from parent directory of current justfile
  pub(crate) fn search_parent_directory(&self) -> SearchResult<Self> {
    let parent = self
      .justfile
      .parent()
      .and_then(|path| path.parent())
      .ok_or_else(|| SearchError::JustfileHadNoParent {
        path: self.justfile.clone(),
      })?;
    Self::find_in_directory(parent)
  }

  /// Find justfile starting in given directory searching upwards in directory tree
  fn find_in_directory(starting_dir: &Path) -> SearchResult<Self> {
    let justfile = Self::justfile(starting_dir)?;
    let working_directory = Self::working_directory_from_justfile(&justfile)?;
    Ok(Self {
      justfile,
      working_directory,
    })
  }

  /// Get working directory and justfile path for newly-initialized justfile
  pub(crate) fn init(
    search_config: &SearchConfig,
    invocation_directory: &Path,
  ) -> SearchResult<Self> {
    match search_config {
      SearchConfig::FromInvocationDirectory => {
        let working_directory = Self::project_root(invocation_directory)?;
        let justfile = working_directory.join(DEFAULT_JUSTFILE_NAME);
        Ok(Self {
          justfile,
          working_directory,
        })
      }
      SearchConfig::FromSearchDirectory { search_directory } => {
        let search_directory = Self::clean(invocation_directory, search_directory);
        let working_directory = Self::project_root(&search_directory)?;
        let justfile = working_directory.join(DEFAULT_JUSTFILE_NAME);
        Ok(Self {
          justfile,
          working_directory,
        })
      }
      SearchConfig::GlobalJustfile => Err(SearchError::GlobalJustfileInit),
      SearchConfig::WithJustfile { justfile } => {
        let justfile = Self::clean(invocation_directory, justfile);
        let working_directory = Self::working_directory_from_justfile(&justfile)?;
        Ok(Self {
          justfile,
          working_directory,
        })
      }
      SearchConfig::WithJustfileAndWorkingDirectory {
        justfile,
        working_directory,
      } => Ok(Self {
        justfile: Self::clean(invocation_directory, justfile),
        working_directory: Self::clean(invocation_directory, working_directory),
      }),
    }
  }

  /// Search upwards from `directory` for a file whose name matches one of
  /// `JUSTFILE_NAMES`
  fn justfile(directory: &Path) -> SearchResult<PathBuf> {
    for directory in directory.ancestors() {
      let mut candidates = BTreeSet::new();

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
          for justfile_name in JUSTFILE_NAMES {
            if name.eq_ignore_ascii_case(justfile_name) {
              candidates.insert(entry.path());
            }
          }
        }
      }

      match candidates.len() {
        0 => {}
        1 => return Ok(candidates.into_iter().next().unwrap()),
        _ => return Err(SearchError::MultipleCandidates { candidates }),
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

  /// Search upwards from `directory` for the root directory of a software
  /// project, as determined by the presence of one of the version control
  /// system directories given in `PROJECT_ROOT_CHILDREN`
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
        for project_root_child in PROJECT_ROOT_CHILDREN.iter().copied() {
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
  use temptree::temptree;

  #[test]
  fn not_found() {
    let tmp = testing::tempdir();
    match Search::justfile(tmp.path()) {
      Err(SearchError::NotFound) => {}
      _ => panic!("No justfile found error was expected"),
    }
  }

  #[test]
  fn multiple_candidates() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    path.push(DEFAULT_JUSTFILE_NAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push(DEFAULT_JUSTFILE_NAME.to_uppercase());
    if fs::File::open(path.as_path()).is_ok() {
      // We are in case-insensitive file system
      return;
    }
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    match Search::justfile(path.as_path()) {
      Err(SearchError::MultipleCandidates { .. }) => {}
      _ => panic!("Multiple candidates error was expected"),
    }
  }

  #[test]
  fn found() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    path.push(DEFAULT_JUSTFILE_NAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    if let Err(err) = Search::justfile(path.as_path()) {
      panic!("No errors were expected: {err}");
    }
  }

  #[test]
  fn found_spongebob_case() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    let spongebob_case = DEFAULT_JUSTFILE_NAME
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
      panic!("No errors were expected: {err}");
    }
  }

  #[test]
  fn found_from_inner_dir() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    path.push(DEFAULT_JUSTFILE_NAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push("a");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    path.push("b");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    if let Err(err) = Search::justfile(path.as_path()) {
      panic!("No errors were expected: {err}");
    }
  }

  #[test]
  fn found_and_stopped_at_first_justfile() {
    let tmp = testing::tempdir();
    let mut path = tmp.path().to_path_buf();
    path.push(DEFAULT_JUSTFILE_NAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push("a");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    path.push(DEFAULT_JUSTFILE_NAME);
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push("b");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    match Search::justfile(path.as_path()) {
      Ok(found_path) => {
        path.pop();
        path.push(DEFAULT_JUSTFILE_NAME);
        assert_eq!(found_path, path);
      }
      Err(err) => panic!("No errors were expected: {err}"),
    }
  }

  #[test]
  fn justfile_symlink_parent() {
    let tmp = temptree! {
      src: "",
      sub: {},
    };

    let src = tmp.path().join("src");
    let sub = tmp.path().join("sub");
    let justfile = sub.join("justfile");

    #[cfg(unix)]
    std::os::unix::fs::symlink(src, &justfile).unwrap();

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
      #[cfg(windows)]
      ("//foo", "bar//baz", "//foo\\bar\\baz"),
      #[cfg(not(windows))]
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
