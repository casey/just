use super::*;

test! {
  name:     no_stdout,
  justfile: r#"
default:
  @echo hello
"#,
  args:     ("--quiet"),
  stdout:   "",
}

test! {
  name:     stderr,
  justfile: r#"
default:
  @echo hello 1>&2
"#,
  args:     ("--quiet"),
  stdout:   "",
}

test! {
  name:     command_echoing,
  justfile: r#"
default:
  exit
"#,
  args:     ("--quiet"),
  stdout:   "",
}

test! {
  name:     error_messages,
  justfile: r#"
default:
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  status:   100,
}

test! {
  name:     assignment_backtick_stderr,
  justfile: r#"
a := `echo hello 1>&2`
default:
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  status:   100,
}

test! {
  name:     interpolation_backtick_stderr,
  justfile: r#"
default:
  echo `echo hello 1>&2`
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  status:   100,
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
  stderr: r#"
    error: a value is required for '--choose <CHOOSE>' but none was supplied

    For more information, try '--help'.
  "#,
  status: EXIT_FAILURE_CLAP,
  shell: false,
}

test! {
  name: choose_status,
  justfile: "foo:",
  args: ("--choose", "--quiet", "--chooser", "/usr/bin/env false"),
  status: EXIT_FAILURE_CLAP,
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
