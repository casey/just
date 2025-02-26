use super::*;

#[test]
fn no_stdout() {
  Test::new()
    .arg("--quiet")
    .justfile(r"
default:
  @echo hello
")
    .stdout("")
    .run();
}

#[test]
fn stderr() {
  Test::new()
    .arg("--quiet")
    .justfile(r"
default:
  @echo hello 1>&2
")
    .stdout("")
    .run();
}

#[test]
fn command_echoing() {
  Test::new()
    .arg("--quiet")
    .justfile(r"
default:
  exit
")
    .stdout("")
    .run();
}

#[test]
fn error_messages() {
  Test::new()
    .arg("--quiet")
    .justfile(r"
default:
  exit 100
")
    .stdout("")
    .status(100)
    .run();
}

#[test]
fn assignment_backtick_stderr() {
  Test::new()
    .arg("--quiet")
    .justfile(r"
a := `echo hello 1>&2`
default:
  exit 100
")
    .stdout("")
    .status(100)
    .run();
}

#[test]
fn interpolation_backtick_stderr() {
  Test::new()
    .arg("--quiet")
    .justfile(r"
default:
  echo `echo hello 1>&2`
  exit 100
")
    .stdout("")
    .status(100)
    .run();
}

test! {
  name: choose_none,
  justfile: "",
  args: ("--choose", "--quiet"),
  status: EXIT_FAILURE,
}

test! {
  name: choose_invocation,
  justfile: "foo:",
  args: ("--choose", "--quiet", "--shell", "asdfasdfasfdasdfasdfadsf"),
  status: EXIT_FAILURE,
  shell: false,
}

test! {
  name: choose_status,
  justfile: "foo:",
  args: ("--choose", "--quiet", "--chooser", "/usr/bin/env false"),
  status: EXIT_FAILURE,
}

test! {
  name: edit_invocation,
  justfile: "foo:",
  args: ("--edit", "--quiet"),
  env: {
    "VISUAL": "adsfasdfasdfadsfadfsaf",
  },
  status: EXIT_FAILURE,
}

test! {
  name: edit_status,
  justfile: "foo:",
  args: ("--edit", "--quiet"),
  env: {
    "VISUAL": "false",
  },
  status: EXIT_FAILURE,
}

test! {
  name: init_exists,
  justfile: "foo:",
  args: ("--init", "--quiet"),
  status: EXIT_FAILURE,
}

test! {
  name: show_missing,
  justfile: "foo:",
  args: ("--show", "bar", "--quiet"),
  status: EXIT_FAILURE,
}

test! {
  name: quiet_shebang,
  justfile: "
    @foo:
      #!/bin/sh
  ",
  args: ("--quiet"),
}

#[test]
fn no_quiet_setting() {
  Test::new()
    .justfile(
      "
        foo:
          echo FOO
      ",
    )
    .stdout("FOO\n")
    .stderr("echo FOO\n")
    .run();
}

#[test]
fn quiet_setting() {
  Test::new()
    .justfile(
      "
      set quiet

      foo:
        echo FOO
      ",
    )
    .stdout("FOO\n")
    .run();
}

#[test]
fn quiet_setting_with_no_quiet_attribute() {
  Test::new()
    .justfile(
      "
      set quiet

      [no-quiet]
      foo:
        echo FOO
      ",
    )
    .stdout("FOO\n")
    .stderr("echo FOO\n")
    .run();
}

#[test]
fn quiet_setting_with_quiet_recipe() {
  Test::new()
    .justfile(
      "
      set quiet

      @foo:
        echo FOO
      ",
    )
    .stdout("FOO\n")
    .run();
}

#[test]
fn quiet_setting_with_quiet_line() {
  Test::new()
    .justfile(
      "
      set quiet

      foo:
        @echo FOO
      ",
    )
    .stdout("FOO\n")
    .run();
}

#[test]
fn quiet_setting_with_no_quiet_attribute_and_quiet_recipe() {
  Test::new()
    .justfile(
      "
      set quiet

      [no-quiet]
      @foo:
        echo FOO
      ",
    )
    .stdout("FOO\n")
    .run();
}

#[test]
fn quiet_setting_with_no_quiet_attribute_and_quiet_line() {
  Test::new()
    .justfile(
      "
      set quiet

      [no-quiet]
      foo:
        @echo FOO
      ",
    )
    .stdout("FOO\n")
    .run();
}
