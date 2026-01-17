use super::*;

#[test]
fn long() {
  Test::new()
    .arg("--command")
    .arg("printf")
    .arg("foo")
    .justfile(
      "
    x:
      echo XYZ
  ",
    )
    .stdout("foo")
    .success();
}

#[test]
fn short() {
  Test::new()
    .arg("-c")
    .arg("printf")
    .arg("foo")
    .justfile(
      "
    x:
      echo XYZ
  ",
    )
    .stdout("foo")
    .success();
}

#[test]
fn command_color() {
  Test::new()
    .arg("--color")
    .arg("always")
    .arg("--command-color")
    .arg("cyan")
    .justfile(
      "
    x:
      echo XYZ
  ",
    )
    .stdout("XYZ\n")
    .stderr("\u{1b}[1;36mecho XYZ\u{1b}[0m\n")
    .success();
}

#[test]
fn no_binary() {
  Test::new()
    .arg("--command")
    .justfile(
      "
    x:
      echo XYZ
  ",
    )
    .stderr(
      "
    error: a value is required for '--command <COMMAND>...' but none was supplied

    For more information, try '--help'.
  ",
    )
    .status(2);
}

#[test]
fn env_is_loaded() {
  Test::new()
    .justfile(
      "
        set dotenv-load

        x:
          echo XYZ
      ",
    )
    .args(["--command", "sh", "-c", "printf $DOTENV_KEY"])
    .write(".env", "DOTENV_KEY=dotenv-value")
    .stdout("dotenv-value")
    .success();
}

#[test]
fn exports_are_available() {
  Test::new()
    .arg("--command")
    .arg("sh")
    .arg("-c")
    .arg("printf $FOO")
    .justfile(
      "
    export FOO := 'bar'

    x:
      echo XYZ
  ",
    )
    .stdout("bar")
    .success();
}

#[test]
fn set_overrides_work() {
  Test::new()
    .arg("--set")
    .arg("FOO")
    .arg("baz")
    .arg("--command")
    .arg("sh")
    .arg("-c")
    .arg("printf $FOO")
    .justfile(
      "
    export FOO := 'bar'

    x:
      echo XYZ
  ",
    )
    .stdout("baz")
    .success();
}

#[test]
fn run_in_shell() {
  Test::new()
    .arg("--shell-command")
    .arg("--command")
    .arg("bar baz")
    .justfile(
      "
    set shell := ['printf']
  ",
    )
    .stdout("bar baz")
    .shell(false)
    .success();
}

#[test]
fn exit_status() {
  Test::new()
    .arg("--command")
    .arg("false")
    .justfile(
      "
    x:
      echo XYZ
  ",
    )
    .stderr_regex("error: Command `false` failed: exit (code|status): 1\n")
    .failure();
}

#[test]
fn working_directory_is_correct() {
  let tmp = tempdir();

  fs::write(tmp.path().join("justfile"), "").unwrap();
  fs::write(tmp.path().join("bar"), "baz").unwrap();
  fs::create_dir(tmp.path().join("foo")).unwrap();

  let output = Command::new(executable_path("just"))
    .args(["--command", "cat", "bar"])
    .current_dir(tmp.path().join("foo"))
    .output()
    .unwrap();

  assert_eq!(str::from_utf8(&output.stderr).unwrap(), "");

  assert!(output.status.success());

  assert_eq!(str::from_utf8(&output.stdout).unwrap(), "baz");
}

#[test]
fn command_not_found() {
  let tmp = tempdir();

  fs::write(tmp.path().join("justfile"), "").unwrap();

  let output = Command::new(executable_path("just"))
    .args(["--command", "asdfasdfasdfasdfadfsadsfadsf", "bar"])
    .output()
    .unwrap();

  assert!(str::from_utf8(&output.stderr)
    .unwrap()
    .starts_with("error: Failed to invoke `asdfasdfasdfasdfadfsadsfadsf` `bar`:"));

  assert!(!output.status.success());
}
