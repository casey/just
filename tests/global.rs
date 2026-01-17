use super::*;

#[test]
fn macos() {
  if cfg!(not(target_os = "macos")) {
    return;
  }
  let tempdir = tempdir();

  let path = tempdir.path().to_owned();

  Test::with_tempdir(tempdir)
    .no_justfile()
    .test_round_trip(false)
    .write(
      "Library/Application Support/just/justfile",
      "@default:\n  echo foo",
    )
    .env("HOME", path.to_str().unwrap())
    .args(["--global-justfile"])
    .stdout("foo\n")
    .success();
}

#[test]
fn not_macos() {
  if cfg!(any(not(unix), target_os = "macos")) {
    return;
  }
  let tempdir = tempdir();

  let path = tempdir.path().to_owned();

  Test::with_tempdir(tempdir)
    .no_justfile()
    .test_round_trip(false)
    .write("just/justfile", "@default:\n  echo foo")
    .env("XDG_CONFIG_HOME", path.to_str().unwrap())
    .args(["--global-justfile"])
    .stdout("foo\n")
    .success();
}

#[test]
fn unix() {
  if cfg!(not(unix)) {
    return;
  }
  let tempdir = tempdir();

  let path = tempdir.path().to_owned();

  let tempdir = Test::with_tempdir(tempdir)
    .no_justfile()
    .test_round_trip(false)
    .write("justfile", "@default:\n  echo foo")
    .env("HOME", path.to_str().unwrap())
    .args(["--global-justfile"])
    .stdout("foo\n")
    .success()
    .tempdir;

  Test::with_tempdir(tempdir)
    .no_justfile()
    .test_round_trip(false)
    .write(".config/just/justfile", "@default:\n  echo bar")
    .env("HOME", path.to_str().unwrap())
    .args(["--global-justfile"])
    .stdout("bar\n")
    .success();
}

#[test]
fn case_insensitive() {
  if cfg!(any(not(unix), target_os = "macos")) {
    return;
  }
  let tempdir = tempdir();

  let path = tempdir.path().to_owned();

  Test::with_tempdir(tempdir)
    .no_justfile()
    .test_round_trip(false)
    .write("just/JUSTFILE", "@default:\n  echo foo")
    .env("XDG_CONFIG_HOME", path.to_str().unwrap())
    .args(["--global-justfile"])
    .stdout("foo\n")
    .success();
}
