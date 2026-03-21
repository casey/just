use super::*;

#[test]
fn shell_on_custom_path() {
  if cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        foo:
          echo bar
      ",
    )
    .write("myshell.exe", "#!/bin/sh\n/bin/sh \"$@\"")
    .make_executable("myshell.exe")
    .env("PATH", path.to_str().unwrap())
    .args(["--shell", "myshell.exe", "--shell-arg", "-c"])
    .shell(false)
    .stdout("bar\n")
    .stderr("echo bar\n")
    .success();
}

#[test]
fn command_on_custom_path() {
  if cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile("")
    .write("foo.exe", "#!/bin/sh\necho bar")
    .make_executable("foo.exe")
    .env("PATH", path.to_str().unwrap())
    .args(["--command", "foo.exe"])
    .shell(false)
    .stdout("bar\n")
    .success();
}

#[test]
fn shell_resolved_via_pathext() {
  if !cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        foo:
          echo bar
      ",
    )
    .write("myshell.cmd", "@cmd %*")
    .env("PATH", path.to_str().unwrap())
    .env("PATHEXT", ".CMD")
    .args(["--shell", "myshell", "--shell-arg", "/C"])
    .shell(false)
    .stdout("bar\r\n")
    .stderr("echo bar\n")
    .success();
}

#[test]
fn command_resolved_via_pathext() {
  if !cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile("")
    .write("foo.cmd", "@echo bar")
    .env("PATH", path.to_str().unwrap())
    .env("PATHEXT", ".CMD")
    .args(["--command", "foo"])
    .shell(false)
    .stdout("bar\r\n")
    .success();
}

#[test]
fn path_ordering() {
  if !cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  let path_var = env::join_paths([
    path.join("dir1").to_str().unwrap(),
    path.join("dir2").to_str().unwrap(),
  ])
  .unwrap();

  Test::with_tempdir(tmp)
    .justfile("")
    .write("dir1/foo.cmd", "@echo dir1")
    .write("dir2/foo.cmd", "@echo dir2")
    .env("PATH", path_var.to_str().unwrap())
    .env("PATHEXT", ".CMD")
    .args(["--command", "foo"])
    .shell(false)
    .stdout("dir1\r\n")
    .success();
}

#[test]
fn script_interpreter_resolved_via_pathext() {
  if !cfg!(windows) {
    return;
  }

  let tmp = tempdir();
  let path = PathBuf::from(tmp.path());

  Test::with_tempdir(tmp)
    .justfile(
      "
        [script('myinterp')]
        [extension('bat')]
        foo:
          @echo bar
      ",
    )
    .write("myinterp.cmd", "@cmd /C %1")
    .env("PATH", path.to_str().unwrap())
    .env("PATHEXT", ".CMD")
    .shell(false)
    .stdout_regex("(?s).*bar.*")
    .stderr_regex(".*")
    .success();
}
