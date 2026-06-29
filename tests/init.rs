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
  Test::new()
    .arg("--init")
    .stderr_regex("Wrote justfile to `.*`\n")
    .success()
    .test()
    .arg("--init")
    .stderr_regex("error: justfile `.*` already exists\n")
    .failure();
}

#[test]
fn write_error() {
  let test = Test::new();

  let justfile_path = test.justfile_path();

  fs::create_dir(justfile_path).unwrap();

  test
    .args(["--init"])
    .stderr_regex(if cfg!(windows) {
      r"error: failed to write justfile to `.*`: Access is denied. \(os error 5\)\n"
    } else {
      r"error: failed to write justfile to `.*`: Is a directory \(os error 21\)\n"
    })
    .failure();
}

#[test]
fn invocation_directory() {
  let test = Test::new().create_dir(".git");

  let justfile_path = test.justfile_path();

  let _tmp = test
    .stderr_regex("Wrote justfile to `.*`\n")
    .arg("--init")
    .success();

  assert_eq!(fs::read_to_string(justfile_path).unwrap(), INIT_JUSTFILE);
}

#[test]
fn parent_dir() {
  let tmp = tempdir();
  fs::create_dir(tmp.path().join(".git")).unwrap();
  fs::create_dir(tmp.path().join("sub")).unwrap();

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
  let tmp = tempdir();
  fs::create_dir(tmp.path().join("_darcs")).unwrap();

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
  let tmp = tempdir();
  fs::create_dir_all(tmp.path().join("sub/.git")).unwrap();

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
  let tmp = tempdir();
  fs::create_dir_all(tmp.path().join("sub/.git")).unwrap();

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
  let tmp = tempdir();
  fs::create_dir_all(tmp.path().join("sub/.git")).unwrap();

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
fn justfile_name_from_invocation_directory() {
  Test::new()
    .create_dir(".git")
    .args(["--init", "--justfile-name", "foo"])
    .stderr_regex("Wrote justfile to `.*`\n")
    .expect_file("foo", INIT_JUSTFILE)
    .success();
}

#[test]
fn justfile_name_from_search_directory() {
  Test::new()
    .create_dir("sub/.git")
    .args(["--init", "--justfile-name", "foo", "sub/"])
    .stderr_regex("Wrote justfile to `.*`\n")
    .expect_file("sub/foo", INIT_JUSTFILE)
    .success();
}

#[test]
fn fmt_compatibility() {
  Test::new()
    .arg("--init")
    .stderr_regex("Wrote justfile to `.*`\n")
    .success()
    .test()
    .arg("--unstable")
    .arg("--check")
    .arg("--fmt")
    .success();
}
