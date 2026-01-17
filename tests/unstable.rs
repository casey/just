use super::*;

#[test]
fn set_unstable_true_with_env_var() {
  for val in ["true", "some-arbitrary-string"] {
    Test::new()
      .justfile("# hello")
      .args(["--fmt"])
      .env("JUST_UNSTABLE", val)
      .stderr_regex("Wrote justfile to `.*`\n")
      .success();
  }
}

#[test]
fn set_unstable_false_with_env_var() {
  for val in ["0", "", "false"] {
    Test::new()
      .justfile("")
      .args(["--fmt"])
      .env("JUST_UNSTABLE", val)
      .stderr_regex("error: The `--fmt` command is currently unstable.*")
      .failure();
  }
}

#[test]
fn set_unstable_false_with_env_var_unset() {
  Test::new()
    .justfile("")
    .args(["--fmt"])
    .stderr_regex("error: The `--fmt` command is currently unstable.*")
    .failure();
}

#[test]
fn set_unstable_with_setting() {
  Test::new()
    .justfile("set unstable")
    .arg("--fmt")
    .stderr_regex("Wrote justfile to .*")
    .success();
}

// This test should be re-enabled if we get a new unstable feature which is
// encountered in source files. (As opposed to, for example, the unstable
// `--fmt` subcommand, which is encountered on the command line.)
#[cfg(any())]
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
    .stderr_regex("error: Modules are currently unstable.*")
    .failure();
}
