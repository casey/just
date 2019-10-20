use crate::common::*;

pub(crate) const FILENAME: &str = "justfile";

fn dir_traverse<F>(directory: &Path, mut pred: F) -> Result<(), SearchError>
  where F: FnMut(fs::DirEntry) -> Result<(), SearchError>
{
  let dir = fs::read_dir(directory).map_err(|io_error| SearchError::Io {
    io_error,
    directory: directory.to_owned(),
  })?;
  for entry in dir {
    let entry = entry.map_err(|io_error| SearchError::Io {
      io_error,
      directory: directory.to_owned(),
    })?;
    if let Err(err) = pred(entry) {
      return Err(err);
    }
  }

  Ok(())
}

pub(crate) fn search_for_justfile<F>(directory: &Path, mut pred: F) -> Result<(), SearchError>
  where F: FnMut(fs::DirEntry) -> Result<(), SearchError>
{
  dir_traverse(directory, |entry| {
    if let Some(name) = entry.file_name().to_str() {
      if name.eq_ignore_ascii_case(FILENAME) {
        return pred(entry);
      }
    }
    Ok(())
  })
}

pub(crate) fn justfile(directory: &Path) -> Result<PathBuf, SearchError> {
  let mut candidates = Vec::new();
  search_for_justfile(directory, |entry| {
    candidates.push(entry.path());
    Ok(())
  })?;
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

pub(crate) fn project_root(start: &Path) -> io::Result<&Path> {
  for directory in start.ancestors() {
    for name in &[".git", "Cargo.toml"] {
      match directory.join(name).metadata() {
        Ok(_) => {
          return Ok(directory);
        },
        Err(err) => {
          if err.kind() != io::ErrorKind::NotFound {
            return Err(err);
          }
        }
      }
    }
  }

  Ok(start)
}

#[cfg(test)]
mod tests {
  use super::*;

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

  #[test]
  fn get_project_root() {
    let tmpdir = testing::tempdir();
    let path = tmpdir.path();
    assert_eq!(project_root(path).unwrap(), path);
  }

  #[test]
  fn git_project_root() {
    let tmpdir = testing::tempdir();
    let mut path = tmpdir.path().to_path_buf();
    path.push(".git");
    assert_eq!(project_root(&path).unwrap(), path);
  }

  #[test]
  fn git_nested_project_root() {
    let tmpdir = testing::tempdir();
    let mut path = tmpdir.path().to_path_buf();
    let expected = path.clone();
    path.push(".git");
    fs::create_dir(&path).expect("unable to create .git directory");

    path.pop();
    path.push("a");
    path.push("b");
    path.push("c");
    fs::create_dir_all(&path).expect("unable to create intermediate directories");
    assert_eq!(project_root(&path).unwrap(), expected);
  }

  #[test]
  fn cargo_project_root() {
    let tmpdir = testing::tempdir();
    let mut path = tmpdir.path().to_path_buf();
    path.push("Cargo.toml");
    assert_eq!(project_root(&path).unwrap(), path);
  }

  #[test]
  fn cargo_nested_project_root() {
    let tmpdir = testing::tempdir();
    let mut path = tmpdir.path().to_path_buf();
    let expected = path.clone();
    path.push("Cargo.toml");
    fs::create_dir(&path).expect("unable to create Cargo.toml");

    path.pop();
    path.push("a");
    path.push("b");
    path.push("c");
    fs::create_dir_all(&path).expect("unable to create intermediate directories");
    assert_eq!(project_root(&path).unwrap(), expected);
  }
}
