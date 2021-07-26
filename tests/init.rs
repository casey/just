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
  let tempdir = Test::new()
    .no_justfile()
    .arg("--init")
    .stderr_regex("Wrote justfile to `.*`\n")
    .run();

  Test::with_tempdir(tempdir)
    .no_justfile()
    .arg("--init")
    .status(EXIT_FAILURE)
    .stderr_regex("error: Justfile `.*` already exists\n")
    .run();
}

#[test]
fn write_error() {
  let test = Test::new();

  let justfile_path = test.justfile_path();

  fs::create_dir(&justfile_path).unwrap();

  test
    .no_justfile()
    .args(&["--init"])
    .status(EXIT_FAILURE)
    .stderr(format!(
      "error: Failed to write justfile to `{}`: Is a directory (os error 21)\n",
      justfile_path.canonicalize().unwrap().display()
    ))
    .run();
}

#[test]
fn invocation_directory() {
  let tmp = temptree! {
    ".git": {},
  };

  let test = Test::with_tempdir(tmp);

  let justfile_path = test.justfile_path();

  let tmp = test
    .no_justfile()
    .stderr_regex("Wrote justfile to `.*`\n")
    .arg("--init")
    .run();

  assert_eq!(fs::read_to_string(justfile_path).unwrap(), EXPECTED);
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
