use super::*;

pub(crate) struct Source<'src> {
  pub(crate) path: PathBuf,
  pub(crate) depth: u32,
  pub(crate) namepath: Namepath<'src>,
  pub(crate) working_directory: PathBuf,
}

impl<'src> Source<'src> {
  pub(crate) fn root(path: &Path) -> Self {
    Self {
      path: path.into(),
      depth: 0,
      namepath: Namepath::default(),
      working_directory: path.parent().unwrap().into(),
    }
  }

  /// Create a justfile source from a filesystem path.
  /// - Path to a file: The file is used as the root justfile
  /// - Path to a directory with a single justfile: The file is used.
  /// - Path to a directory without a justfile: Search continues in parent dir.
  /// - Two or more justfiles are found, e.g. `justfile` and `.justfile`: Error is returned.
  /// - Not a single justfile is found in all the parents up to the top: Error is returned.
  pub(crate) fn from_path(path: &Path) -> SearchResult<Self> {
    let io_err = |io_error| SearchError::Io {
      directory: path.into(),
      io_error,
    };
    let parent_dir = path.parent().unwrap_or(&path).to_path_buf();
    let mut path = canonicalize(path)?;

    // Path is a file
    if path.is_file() {
      return Ok(Self {
        path,
        depth: 0,
        namepath: Namepath::default(),
        working_directory: parent_dir,
      });
    }

    // Path is a dir
    loop {
      // Get all justfile candidates from the dir
      let mut candidates: Vec<PathBuf> = Vec::with_capacity(search::JUSTFILE_NAMES.len());
      let read_dir = path.read_dir().map_err(io_err)?;
      let dir_entries = read_dir.map(|entry| entry.map_err(io_err));
      for dir_entry in dir_entries {
        if let Some(file_name) = dir_entry?.path().file_name() {
          for justfile in search::JUSTFILE_NAMES {
            if justfile.to_lowercase().as_str() == file_name.to_ascii_lowercase().as_os_str() {
              candidates.push(path.join(file_name));
            }
          }
        }
      }

      // Multiple justfiles found in dir
      if candidates.len() > 1 {
        return Err(SearchError::MultipleCandidates { candidates });
      }

      // Exactly one justfile found in dir
      if candidates.len() == 1 {
        return Ok(Self {
          path: candidates[0].clone(),
          depth: 0,
          namepath: Namepath::default(),
          working_directory: path,
        });
      }

      // Nothing found and root dir reached
      let parent_dir = path.parent().unwrap_or(&path).to_path_buf();
      if path == parent_dir {
        return Err(SearchError::NotFound);
      }

      // Nothing found but check parent
      path = parent_dir;
    }
  }

  pub(crate) fn import(&self, path: PathBuf) -> Self {
    Self {
      depth: self.depth + 1,
      path,
      namepath: self.namepath.clone(),
      working_directory: self.working_directory.clone(),
    }
  }

  pub(crate) fn module(&self, name: Name<'src>, path: PathBuf) -> Self {
    Self {
      working_directory: path.parent().unwrap().into(),
      path,
      depth: self.depth + 1,
      namepath: self.namepath.join(name),
    }
  }
}

fn canonicalize(path: &Path) -> SearchResult<PathBuf> {
  path
    .canonicalize()
    .map_err(|err| SearchError::MalformedPath {
      path: path.to_path_buf(),
      err,
    })
}
