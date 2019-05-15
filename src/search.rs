use crate::search_error::SearchError;
use std::fs;
use std::path::{Path, PathBuf};

pub fn search(directory: &Path) -> Result<PathBuf, SearchError> {
  let mut files = Vec::new();
  let dir = fs::read_dir(directory).map_err(|io_error| SearchError::Io {
    io_error,
    directory: directory.to_owned(),
  })?;
  for entry in dir {
    let entry = entry.map_err(|io_error| SearchError::Io {
      io_error,
      directory: directory.to_owned(),
    })?;
    if let Some(name) = entry.file_name().to_str() {
      use caseless::default_caseless_match_str;
      if default_caseless_match_str(name, "justfile") {
        files.push(entry.path());
      }
    }
  }
  if files.len() == 1 {
    Ok(files.pop().unwrap())
  } else if files.len() > 1 {
    Err(SearchError::MultipleCandidates { candidates: files })
  } else if let Some(parent_dir) = directory.parent() {
    search(parent_dir)
  } else {
    Err(SearchError::NotFound)
  }
}
