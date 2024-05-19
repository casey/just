use super::*;

#[test]
#[cfg(target_os = "macos")]
fn macos() {
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
    .run();
}

#[test]
#[cfg(all(unix, not(target_os = "macos")))]
fn not_macos() {
  let tempdir = tempdir();

  let path = tempdir.path().to_owned();

  Test::with_tempdir(tempdir)
    .no_justfile()
    .test_round_trip(false)
    .write("just/justfile", "@default:\n  echo foo")
    .env("XDG_CONFIG_HOME", path.to_str().unwrap())
    .args(["--global-justfile"])
    .stdout("foo\n")
    .run();
}

#[test]
#[cfg(unix)]
fn unix() {
  let tempdir = tempdir();

  let path = tempdir.path().to_owned();

  let tempdir = Test::with_tempdir(tempdir)
    .no_justfile()
    .test_round_trip(false)
    .write("justfile", "@default:\n  echo foo")
    .env("HOME", path.to_str().unwrap())
    .args(["--global-justfile"])
    .stdout("foo\n")
    .run()
    .tempdir;

  Test::with_tempdir(tempdir)
    .no_justfile()
    .test_round_trip(false)
    .write(".config/just/justfile", "@default:\n  echo bar")
    .env("HOME", path.to_str().unwrap())
    .args(["--global-justfile"])
    .stdout("bar\n")
    .run();
}
