use super::*;

#[test]
fn no_stdout() {
  Test::new()
    .arg("--quiet")
    .justfile(
      r"
default:
  @echo hello
",
    )
    .success();
}

#[test]
fn stderr() {
  Test::new()
    .arg("--quiet")
    .justfile(
      r"
default:
  @echo hello 1>&2
",
    )
    .success();
}

#[test]
fn command_echoing() {
  Test::new()
    .arg("--quiet")
    .justfile(
      r"
default:
  exit
",
    )
    .success();
}

#[test]
fn error_messages() {
  Test::new()
    .arg("--quiet")
    .justfile(
      r"
default:
  exit 100
",
    )
    .status(100);
}

#[test]
fn assignment_backtick_stderr() {
  Test::new()
    .arg("--quiet")
    .justfile(
      r"
a := `echo hello 1>&2`
default:
  exit 100
",
    )
    .status(100);
}

#[test]
fn interpolation_backtick_stderr() {
  Test::new()
    .arg("--quiet")
    .justfile(
      r"
default:
  echo `echo hello 1>&2`
  exit 100
",
    )
    .status(100);
}

#[test]
fn choose_none() {
  Test::new()
    .arg("--choose")
    .arg("--quiet")
    .justfile("")
    .failure();
}

#[test]
fn choose_invocation() {
  Test::new()
    .arg("--choose")
    .arg("--quiet")
    .arg("--shell")
    .arg("asdfasdfasfdasdfasdfadsf")
    .justfile("foo:")
    .shell(false)
    .failure();
}

#[test]
fn choose_status() {
  Test::new()
    .arg("--choose")
    .arg("--quiet")
    .arg("--chooser")
    .arg("/usr/bin/env false")
    .justfile("foo:")
    .failure();
}

#[test]
fn edit_invocation() {
  Test::new()
    .arg("--edit")
    .arg("--quiet")
    .env("VISUAL", "adsfasdfasdfadsfadfsaf")
    .justfile("foo:")
    .failure();
}

#[test]
fn edit_status() {
  Test::new()
    .arg("--edit")
    .arg("--quiet")
    .env("VISUAL", "false")
    .justfile("foo:")
    .failure();
}

#[test]
fn init_exists() {
  Test::new()
    .arg("--init")
    .arg("--quiet")
    .justfile("foo:")
    .failure();
}

#[test]
fn show_missing() {
  Test::new()
    .arg("--show")
    .arg("bar")
    .arg("--quiet")
    .justfile("foo:")
    .failure();
}

#[test]
fn quiet_shebang() {
  Test::new()
    .arg("--quiet")
    .justfile(
      "
    @foo:
      #!/bin/sh
  ",
    )
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .success();
}
