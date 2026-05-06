use {super::*, std::path::Component};

const DEFAULT_JUSTFILE_NAME: &str = JUSTFILE_NAMES[0];
pub(crate) const JUSTFILE_NAMES: [&str; 2] = ["justfile", ".justfile"];
const PROJECT_ROOT_CHILDREN: &[&str] = &[".bzr", ".git", ".hg", ".svn", "_darcs"];

#[derive(Debug)]
pub(crate) struct Search {
  pub(crate) justfile: PathBuf,
  #[allow(unused)]
  pub(crate) tempdir: Option<TempDir>,
  pub(crate) working_directory: PathBuf,
}

impl Search {
  fn global_justfile_paths() -> Vec<(PathBuf, &'static str)> {
    let mut paths = Vec::new();

    if let Some(config_dir) = dirs::config_dir() {
      paths.push((config_dir.join(JUST_DIRECTORY), DEFAULT_JUSTFILE_NAME));
    }

    if let Some(home_dir) = dirs::home_dir() {
      paths.push((
        home_dir.join(".config").join(JUST_DIRECTORY),
        DEFAULT_JUSTFILE_NAME,
      ));

      for justfile_name in JUSTFILE_NAMES {
        paths.push((home_dir.clone(), justfile_name));
      }
    }

    paths
  }

  /// Find justfile given search configuration and invocation directory
  pub(crate) fn search(config: &Config) -> SearchResult<Self> {
    match &config.search_config {
      SearchConfig::FromInvocationDirectory => {
        Self::find_in_directory(config, &config.invocation_directory)
      }
      SearchConfig::FromSearchDirectory { search_directory } => {
        let search_directory = Self::clean(&config.invocation_directory, search_directory);
        let justfile = Self::justfile(config, &search_directory)?;
        let working_directory = Self::working_directory_from_justfile(&justfile)?;
        Ok(Self {
          justfile,
          tempdir: None,
          working_directory,
        })
      }
      SearchConfig::FromStandardInput { working_directory } => {
        let source =
          io::read_to_string(io::stdin()).map_err(|io_error| SearchError::StdinIo { io_error })?;

        let mut builder = tempfile::Builder::new();

        builder.prefix(TEMPDIR_PREFIX);

        let tempdir = if let Some(tempdir) = &config.tempdir {
          builder.tempdir_in(tempdir)
        } else {
          builder.tempdir()
        }
        .map_err(|io_error| SearchError::TempdirIo { io_error })?;

        let justfile = tempdir.path().join("justfile");

        fs::write(&justfile, source).map_err(|io_error| SearchError::FilesystemIo {
          io_error,
          path: justfile.clone(),
        })?;

        Ok(Self {
          justfile,
          tempdir: Some(tempdir),
          working_directory: working_directory
            .as_ref()
            .unwrap_or(&config.invocation_directory)
            .clone(),
        })
      }
      SearchConfig::GlobalJustfile => Ok(Self {
        justfile: Self::find_global_justfile()?,
        tempdir: None,
        working_directory: Self::project_root(config, &config.invocation_directory)?,
      }),
      SearchConfig::WithJustfile { justfile } => {
        let justfile = Self::clean(&config.invocation_directory, justfile);
        let working_directory = Self::working_directory_from_justfile(&justfile)?;
        Ok(Self {
          justfile,
          tempdir: None,
          working_directory,
        })
      }
      SearchConfig::WithJustfileAndWorkingDirectory {
        justfile,
        working_directory,
      } => Ok(Self {
        justfile: Self::clean(&config.invocation_directory, justfile),
        tempdir: None,
        working_directory: Self::clean(&config.invocation_directory, working_directory),
      }),
    }
  }

  fn find_global_justfile() -> SearchResult<PathBuf> {
    for (directory, filename) in Self::global_justfile_paths() {
      if let Ok(read_dir) = fs::read_dir(&directory) {
        for entry in read_dir {
          let entry = entry.map_err(|io_error| SearchError::FilesystemIo {
            io_error,
            path: directory.clone(),
          })?;
          if let Some(candidate) = entry.file_name().to_str() {
            if candidate.eq_ignore_ascii_case(filename) {
              return Ok(entry.path());
            }
          }
        }
      }
    }

    Err(SearchError::GlobalJustfileNotFound)
  }

  /// Find justfile starting from parent directory of current justfile
  pub(crate) fn search_parent_directory(&self, config: &Config) -> SearchResult<Self> {
    let parent = self
      .justfile
      .parent()
      .and_then(|path| path.parent())
      .ok_or_else(|| SearchError::JustfileHadNoParent {
        path: self.justfile.clone(),
      })?;
    Self::find_in_directory(config, parent)
  }

  /// Find justfile starting in given directory searching upwards in directory tree
  fn find_in_directory(config: &Config, starting_dir: &Path) -> SearchResult<Self> {
    let justfile = Self::justfile(config, starting_dir)?;
    let working_directory = Self::working_directory_from_justfile(&justfile)?;
    Ok(Self {
      justfile,
      tempdir: None,
      working_directory,
    })
  }

  /// Get working directory and justfile path for newly-initialized justfile
  pub(crate) fn init(config: &Config) -> SearchResult<Self> {
    let default_justfile_name = || {
      config
        .justfile_names
        .as_ref()
        .and_then(|names| names.first().map(String::as_str))
        .unwrap_or(DEFAULT_JUSTFILE_NAME)
    };

    match &config.search_config {
      SearchConfig::FromInvocationDirectory => {
        let working_directory = Self::project_root(config, &config.invocation_directory)?;
        let justfile = working_directory.join(default_justfile_name());
        Ok(Self {
          justfile,
          tempdir: None,
          working_directory,
        })
      }
      SearchConfig::FromStandardInput { .. } => Err(SearchError::InitWithJustfileFromStandardInput),
      SearchConfig::FromSearchDirectory { search_directory } => {
        let search_directory = Self::clean(&config.invocation_directory, search_directory);
        let working_directory = Self::project_root(config, &search_directory)?;
        let justfile = working_directory.join(default_justfile_name());
        Ok(Self {
          justfile,
          tempdir: None,
          working_directory,
        })
      }
      SearchConfig::GlobalJustfile => Err(SearchError::GlobalJustfileInit),
      SearchConfig::WithJustfile { justfile } => {
        let justfile = Self::clean(&config.invocation_directory, justfile);
        let working_directory = Self::working_directory_from_justfile(&justfile)?;
        Ok(Self {
          justfile,
          tempdir: None,
          working_directory,
        })
      }
      SearchConfig::WithJustfileAndWorkingDirectory {
        justfile,
        working_directory,
      } => Ok(Self {
        justfile: Self::clean(&config.invocation_directory, justfile),
        tempdir: None,
        working_directory: Self::clean(&config.invocation_directory, working_directory),
      }),
    }
  }

  /// Search upwards from `directory` for a file whose name matches one of
  /// `JUSTFILE_NAMES`
  fn justfile(config: &Config, directory: &Path) -> SearchResult<PathBuf> {
    for directory in directory.ancestors() {
      let mut candidates = BTreeSet::new();

      let entries = fs::read_dir(directory).map_err(|io_error| SearchError::FilesystemIo {
        io_error,
        path: directory.to_owned(),
      })?;

      for entry in entries {
        let entry = entry.map_err(|io_error| SearchError::FilesystemIo {
          io_error,
          path: directory.to_owned(),
        })?;
        if let Some(name) = entry.file_name().to_str() {
          let justfile_names: Box<dyn Iterator<Item = &str>> =
            if let Some(justfile_names) = &config.justfile_names {
              Box::new(justfile_names.iter().map(String::as_str))
            } else {
              Box::new(JUSTFILE_NAMES.into_iter())
            };

          for justfile_name in justfile_names {
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

      if let Some(ceiling) = &config.ceiling {
        if directory == ceiling {
          break;
        }
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
  fn project_root(config: &Config, directory: &Path) -> SearchResult<PathBuf> {
    for directory in directory.ancestors() {
      let entries = fs::read_dir(directory).map_err(|io_error| SearchError::FilesystemIo {
        io_error,
        path: directory.to_owned(),
      })?;

      for entry in entries {
        let entry = entry.map_err(|io_error| SearchError::FilesystemIo {
          io_error,
          path: directory.to_owned(),
        })?;
        for project_root_child in PROJECT_ROOT_CHILDREN.iter().copied() {
          if entry.file_name() == project_root_child {
            return Ok(directory.to_owned());
          }
        }
      }

      if let Some(ceiling) = &config.ceiling {
        if directory == ceiling {
          break;
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

  #[test]
  fn clean() {
    let cases = &[
      ("/", "foo", "/foo"),
      ("/bar", "/foo", "/foo"),
      if cfg!(windows) {
        ("//foo", "bar//baz", "//foo\\bar\\baz")
      } else {
        ("/", "..", "/")
      },
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
