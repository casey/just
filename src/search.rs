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

#[cfg(test)]
mod test {
  use crate::search::search;
  use crate::search_error::SearchError;
  use std::fs;
  use tempdir::TempDir;

  #[test]
  fn not_found() {
    let tmp = TempDir::new("just-test-justfile-search")
      .expect("test justfile search: failed to create temporary directory");
    let path = tmp.path().to_path_buf();
    match search(path.as_path()) {
      Err(SearchError::NotFound) => {
        assert!(true);
      }
      _ => panic!("No justfile found error was expected"),
    }
  }

  #[test]
  #[cfg(unix)]
  fn multiple_candidates() {
    let tmp = TempDir::new("just-test-justfile-search")
      .expect("test justfile search: failed to create temporary directory");
    let mut path = tmp.path().to_path_buf();
    path.push("justfile");
    path.push("JUSTFILE");
    match search(path.as_path()) {
      Err(SearchError::MultipleCandidates { candidates }) => {
        assert!(true);
      }
      _ => panic!("Multiple candidates error was expected"),
    }
  }

  #[test]
  fn found() {
    let tmp = TempDir::new("just-test-justfile-search")
      .expect("test justfile search: failed to create temporary directory");
    let mut path = tmp.path().to_path_buf();
    path.push("justfile");
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    match search(path.as_path()) {
      Ok(_path) => {
        assert!(true);
      }
      _ => panic!("No errors were expected"),
    }
  }

  #[test]
  fn found_studly_caps() {
    let tmp = TempDir::new("just-test-justfile-search")
      .expect("test justfile search: failed to create temporary directory");
    let mut path = tmp.path().to_path_buf();
    path.push("JuStFiLE");
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    match search(path.as_path()) {
      Ok(_path) => {
        assert!(true);
      }
      _ => panic!("No errors were expected"),
    }
  }

  #[test]
  fn found_from_inner_dir() {
    let tmp = TempDir::new("just-test-justfile-search")
      .expect("test justfile search: failed to create temporary directory");
    let mut path = tmp.path().to_path_buf();
    path.push("justfile");
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push("a");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    path.push("b");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    match search(path.as_path()) {
      Ok(_path) => {
        assert!(true);
      }
      _ => panic!("No errors were expected"),
    }
  }

  #[test]
  fn found_and_stopped_at_first_justfile() {
    let tmp = TempDir::new("just-test-justfile-search")
      .expect("test justfile search: failed to create temporary directory");
    let mut path = tmp.path().to_path_buf();
    path.push("justfile");
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push("a");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    path.push("justfile");
    fs::write(&path, "default:\n\techo ok").unwrap();
    path.pop();
    path.push("b");
    fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
    match search(path.as_path()) {
      Ok(found_path) => {
        path.pop();
        path.push("justfile");
        assert_eq!(found_path, path);
      }
      _ => panic!("No errors were expected"),
    }
  }
}
