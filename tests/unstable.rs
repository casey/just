use super::*;

#[test]
fn set_unstable_true_with_env_var() {
  let justfile = r#"
default:
    echo 'foo'
  "#;

  for val in ["true", "some-arbitrary-string"] {
    Test::new()
      .justfile(justfile)
      .args(["--fmt"])
      .env("JUST_UNSTABLE", val)
      .status(EXIT_SUCCESS)
      .stderr_regex("Wrote justfile to `.*`\n")
      .run();
  }
}

#[test]
fn set_unstable_false_with_env_var() {
  let justfile = r#"
default:
    echo 'foo'
  "#;
  for val in ["0", "", "false"] {
    Test::new()
    .justfile(justfile)
    .args(["--fmt"])
    .env("JUST_UNSTABLE", val)
    .status(EXIT_FAILURE)
    .stderr("error: The `--fmt` command is currently unstable. Invoke `just` with the `--unstable` flag to enable unstable features.\n")
    .run();
  }
}

#[test]
fn set_unstable_false_with_env_var_unset() {
  let justfile = r#"
default:
    echo 'foo'
  "#;
  Test::new()
    .justfile(justfile)
    .args(["--fmt"])
    .status(EXIT_FAILURE)
    .stderr("error: The `--fmt` command is currently unstable. Invoke `just` with the `--unstable` flag to enable unstable features.\n")
    .run();
}

#[test]
fn set_unstable_with_setting() {
  Test::new()
    .justfile(
      "
        set unstable

        mod foo
      ",
    )
    .write("foo.just", "@bar:\n echo BAR")
    .args(["foo", "bar"])
    .stdout("BAR\n")
    .run();
}

#[test]
fn unstable_setting_does_not_affect_submodules() {
  Test::new()
    .justfile(
      "
        set unstable

        mod foo
      ",
    )
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:\n echo hello")
    .args(["foo", "bar"])
    .stderr(
      "error: Modules are currently unstable. \
      Invoke `just` with the `--unstable` flag to enable unstable features.\n",
    )
    .status(EXIT_FAILURE)
    .run();
}
