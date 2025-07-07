use super::*;

#[cfg(unix)]
fn convert_native_path(path: &Path) -> String {
  fs::canonicalize(path)
    .expect("canonicalize failed")
    .to_str()
    .map(str::to_string)
    .expect("unicode decode failed")
}

#[cfg(windows)]
fn convert_native_path(path: &Path) -> String {
  // Translate path from windows style to unix style
  let mut cygpath = Command::new(env::var_os("JUST_CYGPATH").unwrap_or("cygpath".into()));
  cygpath.arg("--unix");
  cygpath.arg(path);

  let output = cygpath.output().expect("executing cygpath failed");

  assert!(output.status.success());

  let stdout = str::from_utf8(&output.stdout).expect("cygpath output was not utf8");

  if stdout.ends_with('\n') {
    &stdout[0..stdout.len() - 1]
  } else if stdout.ends_with("\r\n") {
    &stdout[0..stdout.len() - 2]
  } else {
    stdout
  }
  .to_owned()
}

#[test]
fn test_invocation_directory() {
  let tmp = tempdir();

  let mut justfile_path = tmp.path().to_path_buf();
  justfile_path.push("justfile");
  fs::write(
    justfile_path,
    "default:\n @cd {{invocation_directory()}}\n @echo {{invocation_directory()}}",
  )
  .unwrap();

  let mut subdir = tmp.path().to_path_buf();
  subdir.push("subdir");
  fs::create_dir(&subdir).unwrap();

  let output = Command::new(executable_path("just"))
    .current_dir(&subdir)
    .args(["--shell", "sh"])
    .output()
    .expect("just invocation failed");

  let mut failure = false;

  let expected_status = 0;
  let expected_stdout = convert_native_path(&subdir) + "\n";
  let expected_stderr = "";

  let status = output.status.code().unwrap();
  if status != expected_status {
    println!("bad status: {status} != {expected_status}");
    failure = true;
  }

  let stdout = str::from_utf8(&output.stdout).unwrap();
  if stdout != expected_stdout {
    println!("bad stdout:\ngot:\n{stdout:?}\n\nexpected:\n{expected_stdout:?}");
    failure = true;
  }

  let stderr = str::from_utf8(&output.stderr).unwrap();
  if stderr != expected_stderr {
    println!("bad stderr:\ngot:\n{stderr:?}\n\nexpected:\n{expected_stderr:?}");
    failure = true;
  }

  assert!(!failure, "test failed");
}

#[test]
fn invocation_directory_native() {
  let Output {
    stdout, tempdir, ..
  } = Test::new()
    .justfile("x := invocation_directory_native()")
    .args(["--evaluate", "x"])
    .stdout_regex(".*")
    .run();

  if cfg!(windows) {
    assert_eq!(Path::new(&stdout), tempdir.path());
  } else {
    assert_eq!(Path::new(&stdout), tempdir.path().canonicalize().unwrap());
  }
}
