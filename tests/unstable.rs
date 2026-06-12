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
      .justfile("f(a) := a")
      .arg("--dump")
      .env("JUST_UNSTABLE", val)
      .stderr_regex("error: user-defined functions are currently unstable,.*")
      .failure();
  }
}

#[test]
fn set_unstable_false_with_env_var_unset() {
  Test::new()
    .justfile("f(a) := a")
    .arg("--dump")
    .stderr_regex("error: user-defined functions are currently unstable,.*")
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

#[test]
fn unstable_setting_does_not_affect_submodules() {
  Test::new()
    .justfile(
      "
        set unstable

        mod foo
      ",
    )
    .write("foo.just", "set lists\nbar:\n echo hello")
    .args(["foo", "bar"])
    .stderr_regex("error: the `lists` setting is currently unstable.*")
    .failure();
}
