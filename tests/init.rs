use crate::common::*;

const EXPECTED: &str = "default:\n\techo 'Hello, world!'\n";

#[test]
fn current_dir() {
  let tmp = tempdir();

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--init")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    EXPECTED
  );
}

#[test]
fn exists() {
  let tmp = tempdir();

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--init")
    .output()
    .unwrap();

  assert!(output.status.success());

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--init")
    .output()
    .unwrap();

  assert!(!output.status.success());

  assert_eq!(
    str::from_utf8(&output.stderr).unwrap(),
    format!(
      "error: Justfile `{}` already exists\n",
      tmp
        .path()
        .join("justfile")
        .canonicalize()
        .unwrap()
        .display()
    ),
  );
}

#[test]
fn write_error() {
  let tmp = tempdir();

  fs::create_dir(tmp.path().join("justfile")).unwrap();

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--init")
    .output()
    .unwrap();

  assert!(!output.status.success());

  assert_eq!(
    str::from_utf8(&output.stderr).unwrap(),
    format!(
      "error: Failed to write justfile to `{}`: Is a directory (os error 21)\n",
      tmp
        .path()
        .join("justfile")
        .canonicalize()
        .unwrap()
        .display()
    ),
  );
}

#[test]
fn invocation_directory() {
  let tmp = temptree! {
    ".git": {},
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--init")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    EXPECTED
  );
}

#[test]
fn parent_dir() {
  let tmp = temptree! {
    ".git": {},
    sub: {},
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path().join("sub"))
    .arg("--init")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    EXPECTED
  );
}

#[test]
fn alternate_marker() {
  let tmp = temptree! {
    "_darcs": {},
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--init")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    EXPECTED
  );
}

#[test]
fn search_directory() {
  let tmp = temptree! {
    sub: {
      ".git": {},
    },
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--init")
    .arg("sub/")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("sub/justfile")).unwrap(),
    EXPECTED
  );
}

#[test]
fn justfile() {
  let tmp = temptree! {
    sub: {
      ".git": {},
    },
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path().join("sub"))
    .arg("--init")
    .arg("--justfile")
    .arg(tmp.path().join("justfile"))
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    EXPECTED
  );
}

#[test]
fn justfile_and_working_directory() {
  let tmp = temptree! {
    sub: {
      ".git": {},
    },
  };

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path().join("sub"))
    .arg("--init")
    .arg("--justfile")
    .arg(tmp.path().join("justfile"))
    .arg("--working-directory")
    .arg("/")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    EXPECTED
  );
}
