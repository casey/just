use crate::common::*;
use std::fs;
use std::path::{Path, PathBuf};

const FILENAME: &str = "justfile";

pub fn justfile(directory: &Path) -> Result<PathBuf, SearchError> {
  let mut candidates = Vec::new();
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
      if name.eq_ignore_ascii_case(FILENAME) {
        candidates.push(entry.path());
      }
    }
  }
  if candidates.len() == 1 {
    Ok(candidates.pop().unwrap())
  } else if candidates.len() > 1 {
    Err(SearchError::MultipleCandidates { candidates })
  } else if let Some(parent_dir) = directory.parent() {
    justfile(parent_dir)
  } else {
    Err(SearchError::NotFound)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;

  #[test]
  fn not_found() {
    let tmp = testing::tempdir();
    match search::justfile(tmp.path()) {
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
    match search::justfile(path.as_path()) {
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
    match search::justfile(path.as_path()) {
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
    match search::justfile(path.as_path()) {
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
    match search::justfile(path.as_path()) {
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
    match search::justfile(path.as_path()) {
      Ok(found_path) => {
        path.pop();
        path.push(FILENAME);
        assert_eq!(found_path, path);
      }
      _ => panic!("No errors were expected"),
    }
  }
}
