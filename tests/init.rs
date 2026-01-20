use {super::*, just::INIT_JUSTFILE};

#[test]
fn current_dir() {
  let tmp = tempdir();

  let output = Command::new(JUST)
    .current_dir(tmp.path())
    .arg("--init")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    INIT_JUSTFILE
  );
}

#[test]
fn exists() {
  let output = Test::new()
    .no_justfile()
    .arg("--init")
    .stderr_regex("Wrote justfile to `.*`\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .no_justfile()
    .arg("--init")
    .stderr_regex("error: Justfile `.*` already exists\n")
    .failure();
}

#[test]
fn write_error() {
  let test = Test::new();

  let justfile_path = test.justfile_path();

  fs::create_dir(justfile_path).unwrap();

  test
    .no_justfile()
    .args(["--init"])
    .stderr_regex(if cfg!(windows) {
      r"error: Failed to write justfile to `.*`: Access is denied. \(os error 5\)\n"
    } else {
      r"error: Failed to write justfile to `.*`: Is a directory \(os error 21\)\n"
    })
    .failure();
}

#[test]
fn invocation_directory() {
  let tmp = temptree! {
    ".git": {},
  };

  let test = Test::with_tempdir(tmp);

  let justfile_path = test.justfile_path();

  let _tmp = test
    .no_justfile()
    .stderr_regex("Wrote justfile to `.*`\n")
    .arg("--init")
    .success();

  assert_eq!(fs::read_to_string(justfile_path).unwrap(), INIT_JUSTFILE);
}

#[test]
fn parent_dir() {
  let tmp = temptree! {
    ".git": {},
    sub: {},
  };

  let output = Command::new(JUST)
    .current_dir(tmp.path().join("sub"))
    .arg("--init")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    INIT_JUSTFILE
  );
}

#[test]
fn alternate_marker() {
  let tmp = temptree! {
    "_darcs": {},
  };

  let output = Command::new(JUST)
    .current_dir(tmp.path())
    .arg("--init")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    INIT_JUSTFILE
  );
}

#[test]
fn search_directory() {
  let tmp = temptree! {
    sub: {
      ".git": {},
    },
  };

  let output = Command::new(JUST)
    .current_dir(tmp.path())
    .arg("--init")
    .arg("sub/")
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("sub/justfile")).unwrap(),
    INIT_JUSTFILE
  );
}

#[test]
fn justfile() {
  let tmp = temptree! {
    sub: {
      ".git": {},
    },
  };

  let output = Command::new(JUST)
    .current_dir(tmp.path().join("sub"))
    .arg("--init")
    .arg("--justfile")
    .arg(tmp.path().join("justfile"))
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(
    fs::read_to_string(tmp.path().join("justfile")).unwrap(),
    INIT_JUSTFILE
  );
}

#[test]
fn justfile_and_working_directory() {
  let tmp = temptree! {
    sub: {
      ".git": {},
    },
  };

  let output = Command::new(JUST)
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
    INIT_JUSTFILE
  );
}

#[test]
fn fmt_compatibility() {
  let output = Test::new()
    .no_justfile()
    .arg("--init")
    .stderr_regex("Wrote justfile to `.*`\n")
    .success();
  Test::with_tempdir(output.tempdir)
    .no_justfile()
    .arg("--unstable")
    .arg("--check")
    .arg("--fmt")
    .success();
}
