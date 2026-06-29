use super::*;

const HELLO_SCRIPT: &str = "#!/usr/bin/env bash
echo hello
";

#[test]
fn finds_executable() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('hello.exe')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("hello.exe", HELLO_SCRIPT)
    .env("PATH", path.to_str().unwrap())
    .unstable()
    .stdout(path.join("hello.exe").display().to_string())
    .success();
}

#[test]
fn returns_empty_list_for_missing_executable() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := show(which('goodbye.exe'))
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("hello.exe", HELLO_SCRIPT)
    .env("PATH", path.to_str().unwrap())
    .unstable()
    .stdout("[]")
    .success();
}

#[test]
fn skips_non_executable_files() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('hi')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("hello.exe", HELLO_SCRIPT)
    .write("hi", "just some regular file")
    .env("PATH", path.to_str().unwrap())
    .unstable()
    .success();
}

#[test]
fn supports_multiple_paths() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());
  let path_var = env::join_paths([
    path.join("subdir1").to_str().unwrap(),
    path.join("subdir2").to_str().unwrap(),
  ])
  .unwrap();

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('hello1.exe') + '+' + which('hello2.exe')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("subdir1/hello1.exe", HELLO_SCRIPT)
    .write_executable("subdir2/hello2.exe", HELLO_SCRIPT)
    .env("PATH", path_var.to_str().unwrap())
    .unstable()
    .stdout(format!(
      "{}+{}",
      path.join("subdir1").join("hello1.exe").display(),
      path.join("subdir2").join("hello2.exe").display(),
    ))
    .success();
}

#[test]
fn supports_shadowed_executables() {
  enum Variation {
    Dir1Dir2, // PATH=/tmp/.../dir1:/tmp/.../dir2
    Dir2Dir1, // PATH=/tmp/.../dir2:/tmp/.../dir1
  }

  for variation in [Variation::Dir1Dir2, Variation::Dir2Dir1] {
    let tmp = tempdir();
    let path = PathBuf::from(tmp.path());

    let path_var = match variation {
      Variation::Dir1Dir2 => env::join_paths([
        path.join("dir1").to_str().unwrap(),
        path.join("dir2").to_str().unwrap(),
      ]),
      Variation::Dir2Dir1 => env::join_paths([
        path.join("dir2").to_str().unwrap(),
        path.join("dir1").to_str().unwrap(),
      ]),
    }
    .unwrap();

    let stdout = match variation {
      Variation::Dir1Dir2 => format!("{}", path.join("dir1").join("shadowed.exe").display()),
      Variation::Dir2Dir1 => format!("{}", path.join("dir2").join("shadowed.exe").display()),
    };

    Test::with_tempdir(tmp)
      .justfile(
        "
          set lists
          p := which('shadowed.exe')
        ",
      )
      .args(["--evaluate", "p"])
      .write_executable("dir1/shadowed.exe", HELLO_SCRIPT)
      .write_executable("dir2/shadowed.exe", HELLO_SCRIPT)
      .env("PATH", path_var.to_str().unwrap())
      .unstable()
      .stdout(stdout)
      .success();
  }
}

#[test]
fn ignores_nonexecutable_candidates() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  let path_var = env::join_paths([
    path.join("dummy").to_str().unwrap(),
    path.join("subdir").to_str().unwrap(),
    path.join("dummy").to_str().unwrap(),
  ])
  .unwrap();

  let dummy_exe = if cfg!(windows) {
    "dummy/foo"
  } else {
    "dummy/foo.exe"
  };

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('foo.exe')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("subdir/foo.exe", HELLO_SCRIPT)
    .write(dummy_exe, HELLO_SCRIPT)
    .env("PATH", path_var.to_str().unwrap())
    .unstable()
    .stdout(path.join("subdir").join("foo.exe").display().to_string())
    .success();
}

#[test]
fn handles_absolute_path() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());
  let abspath = path.join("subdir").join("foo.exe");

  Test::with_tempdir(tmp)
    .justfile(format!(
      "
        set lists
        p := which('{}')
      ",
      abspath.display()
    ))
    .write_executable("subdir/foo.exe", HELLO_SCRIPT)
    .write_executable("pathdir/foo.exe", HELLO_SCRIPT)
    .env("PATH", path.join("pathdir").to_str().unwrap())
    .unstable()
    .args(["--evaluate", "p"])
    .stdout(abspath.display().to_string())
    .success();
}

#[test]
fn handles_dotslash() {
  let tmp = tempdir();

  let path = if cfg!(windows) {
    tmp.path().into()
  } else {
    // canonicalize() is necessary here to account for the justfile prepending
    // the canonicalized working directory to 'subdir/foo.exe'.
    tmp.path().canonicalize().unwrap()
  };

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('./foo.exe')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("foo.exe", HELLO_SCRIPT)
    .write_executable("pathdir/foo.exe", HELLO_SCRIPT)
    .env("PATH", path.join("pathdir").to_str().unwrap())
    .unstable()
    .stdout(path.join("foo.exe").display().to_string())
    .success();
}

#[test]
fn handles_dir_slash() {
  let tmp = tempdir();

  let path = if cfg!(windows) {
    tmp.path().into()
  } else {
    // canonicalize() is necessary here to account for the justfile prepending
    // the canonicalized working directory to 'subdir/foo.exe'.
    tmp.path().canonicalize().unwrap()
  };

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('subdir/foo.exe')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("subdir/foo.exe", HELLO_SCRIPT)
    .write_executable("pathdir/foo.exe", HELLO_SCRIPT)
    .env("PATH", path.join("pathdir").to_str().unwrap())
    .unstable()
    .stdout(path.join("subdir").join("foo.exe").display().to_string())
    .success();
}

#[test]
fn requires_lists_setting() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile("p := which('hello.exe')")
    .args(["--evaluate", "p"])
    .write_executable("hello.exe", HELLO_SCRIPT)
    .env("PATH", path.to_str().unwrap())
    .unstable()
    .stderr(
      "
        error: the `which()` function requires `set lists`
         ——▶ justfile:1:6
          │
        1 │ p := which('hello.exe')
          │      ^^^^^
      ",
    )
    .failure();
}

#[test]
fn require_error() {
  Test::new()
    .justfile("p := require('asdfasdf')")
    .args(["--evaluate", "p"])
    .stderr(
      "
        error: call to function `require` failed: could not find executable `asdfasdf`
         ——▶ justfile:1:6
          │
        1 │ p := require('asdfasdf')
          │      ^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn finds_executable_via_pathext() {
  if !cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('foo')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("foo.exe", HELLO_SCRIPT)
    .env("PATH", path.to_str().unwrap())
    .env("PATHEXT", ".exe")
    .unstable()
    .stdout(path.join("foo.exe").display().to_string())
    .success();
}

#[test]
fn pathext_not_applied_when_candidate_has_extension() {
  if !cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('foo.bat')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("foo.bat.exe", HELLO_SCRIPT)
    .env("PATH", path.to_str().unwrap())
    .env("PATHEXT", ".EXE")
    .unstable()
    .success();
}

#[test]
fn pathext_custom_extension() {
  if !cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('foo')
      ",
    )
    .args(["--evaluate", "p"])
    .write("foo.bar", HELLO_SCRIPT)
    .env("PATH", path.to_str().unwrap())
    .env("PATHEXT", ".BAR")
    .unstable()
    .stdout(path.join("foo.BAR").display().to_string())
    .success();
}

#[test]
fn pathext_entry_missing_dot_is_error() {
  if !cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('foo')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("foo.exe", HELLO_SCRIPT)
    .env("PATH", path.to_str().unwrap())
    .env("PATHEXT", "EXE")
    .unstable()
    .stderr_regex(".*`PATHEXT` entry `EXE` does not start with `.`.*")
    .failure();
}

#[test]
fn pathext_ignored_on_non_windows() {
  if cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        set lists
        p := which('foo')
      ",
    )
    .args(["--evaluate", "p"])
    .write_executable("foo.exe", HELLO_SCRIPT)
    .env("PATH", path.to_str().unwrap())
    .env("PATHEXT", ".EXE")
    .unstable()
    .success();
}

#[test]
fn require_success() {
  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile("p := require('hello.exe')")
    .args(["--evaluate", "p"])
    .write_executable("hello.exe", HELLO_SCRIPT)
    .env("PATH", path.to_str().unwrap())
    .stdout(path.join("hello.exe").display().to_string())
    .success();
}
