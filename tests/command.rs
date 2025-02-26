use super::*;

#[test]
fn long() {
  Test::new()
    .arg("--command")
    .arg("printf")
    .arg("foo")
    .justfile("
    x:
      echo XYZ
  ")
    .stdout("foo")
    .run();
}

#[test]
fn short() {
  Test::new()
    .arg("-c")
    .arg("printf")
    .arg("foo")
    .justfile("
    x:
      echo XYZ
  ")
    .stdout("foo")
    .run();
}

#[test]
fn command_color() {
  Test::new()
    .arg("--color")
    .arg("always")
    .arg("--command-color")
    .arg("cyan")
    .justfile("
    x:
      echo XYZ
  ")
    .stdout("XYZ\n")
    .stderr("\u{1b}[1;36mecho XYZ\u{1b}[0m\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn no_binary() {
  Test::new()
    .arg("--command")
    .justfile("
    x:
      echo XYZ
  ")
    .stderr("
    error: a value is required for '--command <COMMAND>...' but none was supplied

    For more information, try '--help'.
  ")
    .status(2)
    .run();
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
    .run();
}

test! {
  name: exports_are_available,
  justfile: "
    export FOO := 'bar'

    x:
      echo XYZ
  ",
  args: ("--command", "sh", "-c", "printf $FOO"),
  stdout: "bar",
}

test! {
  name: set_overrides_work,
  justfile: "
    export FOO := 'bar'

    x:
      echo XYZ
  ",
  args: ("--set", "FOO", "baz", "--command", "sh", "-c", "printf $FOO"),
  stdout: "baz",
}

test! {
  name: run_in_shell,
  justfile: "
    set shell := ['printf']
  ",
  args: ("--shell-command", "--command", "bar baz"),
  stdout: "bar baz",
  shell: false,
}

test! {
  name: exit_status,
  justfile: "
    x:
      echo XYZ
  ",
  args: ("--command", "false"),
  stderr_regex: "error: Command `false` failed: exit (code|status): 1\n",
  status: EXIT_FAILURE,
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
