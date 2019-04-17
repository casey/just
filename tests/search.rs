use executable_path::executable_path;
use std::{fs, path, process, str};
use tempdir::TempDir;

fn search_test<P: AsRef<path::Path>>(path: P, args: &[&str]) {
  let binary = executable_path("just");

  let output = process::Command::new(binary)
    .current_dir(path)
    .args(args)
    .output()
    .expect("just invocation failed");

  assert_eq!(output.status.code().unwrap(), 0);

  let stdout = str::from_utf8(&output.stdout).unwrap();
  assert_eq!(stdout, "ok\n");

  let stderr = str::from_utf8(&output.stderr).unwrap();
  assert_eq!(stderr, "echo ok\n");
}

#[test]
fn test_justfile_search() {
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
  path.push("c");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("d");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  search_test(path, &[]);
}

#[test]
fn test_capitalized_justfile_search() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("Justfile");
  fs::write(&path, "default:\n\techo ok").unwrap();
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("b");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("c");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("d");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  search_test(path, &[]);
}

#[test]
fn test_capitalization_priority() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  fs::write(&path, "default:\n\techo ok").unwrap();
  path.pop();
  path.push("Justfile");
  fs::write(&path, "default:\n\techo fail").unwrap();
  path.pop();

  // if we see "default\n\techo fail" in `justfile` then we're running
  // in a case insensitive filesystem, so just bail
  path.push("justfile");
  if fs::read_to_string(&path).unwrap() == "default:\n\techo fail" {
    return;
  }
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("b");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("c");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("d");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  search_test(path, &[]);
}

#[test]
fn test_upwards_path_argument() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  fs::write(&path, "default:\n\techo ok").unwrap();
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  path.push("justfile");
  fs::write(&path, "default:\n\techo bad").unwrap();
  path.pop();

  search_test(&path, &["../"]);
  search_test(&path, &["../default"]);
}

#[test]
fn test_downwards_path_argument() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  fs::write(&path, "default:\n\techo bad").unwrap();
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  path.push("justfile");
  fs::write(&path, "default:\n\techo ok").unwrap();
  path.pop();
  path.pop();

  search_test(&path, &["a/"]);
  search_test(&path, &["a/default"]);
  search_test(&path, &["./a/"]);
  search_test(&path, &["./a/default"]);
  search_test(&path, &["./a/"]);
  search_test(&path, &["./a/default"]);
}
