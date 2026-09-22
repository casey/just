use super::*;

const DEFAULT_JUSTFILE_NAME: &str = JUSTFILE_NAMES[0];
pub(crate) const JUSTFILE_NAMES: [&str; 2] = ["justfile", ".justfile"];
const PROJECT_ROOT_CHILDREN: &[&str] = &[".bzr", ".git", ".hg", ".svn", "_darcs"];

#[derive(Debug)]
pub(crate) struct Search {
  pub(crate) justfile: Utf8PathBuf,
  pub(crate) tempdir: Option<TempDir>,
  pub(crate) working_directory: Utf8PathBuf,
}

impl Search {
  pub(crate) fn justfile_parent(&self) -> &Utf8Path {
    self.justfile.parent().unwrap()
  }

  fn global_justfile_paths() -> SearchResult<Vec<(Utf8PathBuf, &'static str)>> {
    let mut paths = Vec::new();

    if let Some(config_dir) = dir::config_directory()? {
      paths.push((config_dir.join(JUST_DIRECTORY), DEFAULT_JUSTFILE_NAME));
    }

    if let Some(home_dir) = dir::home_directory()? {
      paths.push((
        home_dir.join(".config").join(JUST_DIRECTORY),
        DEFAULT_JUSTFILE_NAME,
      ));

      for justfile_name in JUSTFILE_NAMES {
        paths.push((home_dir.clone(), justfile_name));
      }
    }

    Ok(paths)
  }

  /// Find justfile given search configuration and invocation directory
  pub(crate) fn search(config: &Config) -> SearchResult<Self> {
    match &config.search_config {
      SearchConfig::FromInvocationDirectory => {
        Self::find_in_directory(config, &config.invocation_directory)
      }
      SearchConfig::FromSearchDirectory { search_directory } => {
        let search_directory = Self::clean(config, search_directory);
        let justfile = Self::justfile(config, &search_directory)?;
        let working_directory = Self::working_directory_from_justfile(&justfile)?;
        Ok(Self {
          justfile,
          tempdir: None,
          working_directory,
        })
      }
      SearchConfig::FromStandardInput { working_directory } => {
        let source = io::read_to_string(io::stdin()).context(search_error::StdinIo)?;

        let (justfile, tempdir) = Self::tempdir_justfile(config, &source)?;

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
        let justfile = Self::clean(config, justfile);
        let working_directory = Self::working_directory_from_justfile(&justfile)?;
        Self::with_justfile(config, justfile, working_directory)
      }
      SearchConfig::WithJustfileAndWorkingDirectory {
        justfile,
        working_directory,
      } => {
        let justfile = Self::clean(config, justfile);

        justfile
          .parent()
          .context(search_error::JustfileHadNoParent { path: &justfile })?;

        Self::with_justfile(config, justfile, Self::clean(config, working_directory))
      }
    }
  }

  fn with_justfile(
    config: &Config,
    justfile: Utf8PathBuf,
    working_directory: Utf8PathBuf,
  ) -> SearchResult<Self> {
    if justfile
      .extension()
      .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
    {
      let markdown =
        fs::read_to_string(&justfile).context(search_error::FilesystemIo { path: &justfile })?;

      let source = tangle(&markdown);

      let (justfile, tempdir) = Self::tempdir_justfile(config, &source)?;

      Ok(Self {
        justfile,
        tempdir: Some(tempdir),
        working_directory,
      })
    } else {
      Ok(Self {
        justfile,
        tempdir: None,
        working_directory,
      })
    }
  }

  fn tempdir_justfile(config: &Config, source: &str) -> SearchResult<(Utf8PathBuf, TempDir)> {
    let mut builder = tempfile::Builder::new();

    builder.prefix(TEMPDIR_PREFIX);

    let tempdir = if let Some(tempdir) = &config.tempdir {
      builder.tempdir_in(tempdir)
    } else {
      builder.tempdir()
    }
    .context(search_error::TempdirIo)?;

    let justfile = dir::temporary_directory(&tempdir)?.join("justfile");

    fs::write(&justfile, source).context(search_error::FilesystemIo { path: &justfile })?;

    Ok((justfile, tempdir))
  }

  fn find_global_justfile() -> SearchResult<Utf8PathBuf> {
    for (directory, filename) in Self::global_justfile_paths()? {
      if let Ok(read_dir) = fs::read_dir(&directory) {
        for entry in read_dir {
          let entry = entry.context(search_error::FilesystemIo { path: &directory })?;

          let Ok(path) = entry.path().into_utf8() else {
            continue;
          };

          if path.file_name().unwrap().eq_ignore_ascii_case(filename) {
            return Ok(path);
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
      .context(search_error::JustfileHadNoParent {
        path: &self.justfile,
      })?;
    Self::find_in_directory(config, parent)
  }

  /// Find justfile starting in given directory searching upwards in directory tree
  fn find_in_directory(config: &Config, starting_dir: &Utf8Path) -> SearchResult<Self> {
    let justfile = Self::justfile(config, starting_dir)?;
    let working_directory = Self::working_directory_from_justfile(&justfile)?;
    Self::with_justfile(config, justfile, working_directory)
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
        let search_directory = Self::clean(config, search_directory);
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
        let justfile = Self::clean(config, justfile);
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
        justfile: Self::clean(config, justfile),
        tempdir: None,
        working_directory: Self::clean(config, working_directory),
      }),
    }
  }

  /// Search upwards from `directory` for a file whose name matches one of
  /// `JUSTFILE_NAMES`
  fn justfile(config: &Config, directory: &Utf8Path) -> SearchResult<Utf8PathBuf> {
    for directory in directory.ancestors() {
      let mut candidates = BTreeSet::new();

      let entries =
        fs::read_dir(directory).context(search_error::FilesystemIo { path: directory })?;

      for entry in entries {
        let entry = entry.context(search_error::FilesystemIo { path: directory })?;

        let Ok(path) = entry.path().into_utf8() else {
          continue;
        };

        let mut justfile_names: Box<dyn Iterator<Item = &str>> =
          if let Some(justfile_names) = &config.justfile_names {
            Box::new(justfile_names.iter().map(String::as_str))
          } else {
            Box::new(JUSTFILE_NAMES.into_iter())
          };

        let name = path.file_name().unwrap();
        if justfile_names.any(|justfile_name| name.eq_ignore_ascii_case(justfile_name)) {
          candidates.insert(path);
        }
      }

      match candidates.len() {
        0 => {}
        1 => return Ok(candidates.pop_first().unwrap()),
        _ => return Err(SearchError::MultipleCandidates { candidates }),
      }

      if let Some(ceiling) = &config.ceiling
        && directory == ceiling
      {
        break;
      }
    }

    Err(SearchError::NotFound)
  }

  fn clean(config: &Config, path: &Utf8Path) -> Utf8PathBuf {
    config.invocation_directory.join(path).clean()
  }

  /// Search upwards from `directory` for the root directory of a software
  /// project, as determined by the presence of one of the version control
  /// system directories given in `PROJECT_ROOT_CHILDREN`
  fn project_root(config: &Config, directory: &Utf8Path) -> SearchResult<Utf8PathBuf> {
    for directory in directory.ancestors() {
      let entries =
        fs::read_dir(directory).context(search_error::FilesystemIo { path: directory })?;

      for entry in entries {
        let entry = entry.context(search_error::FilesystemIo { path: directory })?;
        for project_root_child in PROJECT_ROOT_CHILDREN.iter().copied() {
          if entry.file_name() == project_root_child {
            return Ok(directory.to_owned());
          }
        }
      }

      if let Some(ceiling) = &config.ceiling
        && directory == ceiling
      {
        break;
      }
    }

    Ok(directory.to_owned())
  }

  fn working_directory_from_justfile(justfile: &Utf8Path) -> SearchResult<Utf8PathBuf> {
    Ok(
      justfile
        .parent()
        .context(search_error::JustfileHadNoParent { path: justfile })?
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
      let config = Config {
        invocation_directory: prefix.into(),
        ..Config::new().unwrap()
      };
      let have = Search::clean(&config, Utf8Path::new(suffix));
      assert_eq!(have, Utf8Path::new(want));
    }
  }
}
