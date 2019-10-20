use std::{fs, process, str};

use executable_path::executable_path;

use test_utilities::tempdir;

#[test]
fn init_justfile_created() {
  let tmp = tempdir();
  let binary = executable_path("just");
  let args = &["--init"];

  let output = process::Command::new(binary)
    .current_dir(tmp.path())
    .args(args)
    .output()
    .expect("just invocation failed");

  assert_eq!(output.status.code().unwrap(), 0);

  let stdout = str::from_utf8(&output.stdout).unwrap();
  assert_eq!(stdout, "");
  let mut buf = tmp.path().to_path_buf();
  buf.push("justfile");

  assert!(buf.metadata().is_ok());

  let bytes = fs::read(buf).expect("unable to read Justfile");
  let justfile = str::from_utf8(&bytes).expect("unable to convert bytes to str");
  assert_eq!(justfile, "default:\n\techo 'Hello, world!'\n");
}

#[test]
fn init_justfile_created_at_git_root() {
  let tmp = tempdir();
  let binary = executable_path("just");
  let args = &["--init"];

  let mut path = tmp.path().to_path_buf();
  path.push(".git");
  fs::create_dir(&path).expect("unable to create intermediate directory: .git");

  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("unable to create intermediate directory: a");

  let output = process::Command::new(binary)
    .current_dir(path)
    .args(args)
    .output()
    .expect("just invocation failed");

  let stdout = str::from_utf8(&output.stdout).unwrap();
  assert_eq!(stdout, "");
  let mut buf = tmp.path().to_path_buf();
  buf.push("justfile");

  assert!(buf.metadata().is_ok());

  let bytes = fs::read(buf).expect("unable to read Justfile");
  let justfile = str::from_utf8(&bytes).expect("unable to convert bytes to str");
  assert_eq!(justfile, "default:\n\techo 'Hello, world!'\n");
}

#[test]
fn init_fails_if_justfile_exists() {
  let tmp = tempdir();
  let binary = executable_path("just");
  let args = &["--init"];

  let mut path = tmp.path().to_path_buf();
  path.push("Justfile");
  fs::write(&path, "default:\necho ok\n").unwrap();
  path.pop();

  let output = process::Command::new(binary)
    .current_dir(path)
    .args(args)
    .output()
    .expect("just invocation failed");

  let stdout = str::from_utf8(&output.stdout).unwrap();
  assert_eq!(stdout, "");

  let stderr = str::from_utf8(&output.stderr).unwrap();
  assert_eq!(stderr, "Justfile already exists at the project root\n")
}
