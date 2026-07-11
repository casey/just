use super::*;

#[test]
fn allow_missing_recipes_in_run_invocation() {
  Test::new()
    .justfile("")
    .arg("foo")
    .stderr("error: justfile does not contain recipe `foo`\n")
    .failure();

  Test::new()
    .justfile("")
    .args(["--allow-missing", "foo"])
    .success();
}

#[test]
fn allow_missing_modules_in_run_invocation() {
  Test::new()
    .justfile("")
    .arg("foo::bar")
    .stderr("error: justfile does not contain submodule `foo`\n")
    .failure();

  Test::new()
    .justfile("")
    .args(["--allow-missing", "foo::bar"])
    .success();
}

#[test]
fn allow_missing_does_not_apply_to_compilation_errors() {
  Test::new()
    .justfile("bar: foo")
    .args(["--allow-missing", "foo"])
    .stderr(
      "
        error: recipe `bar` has unknown dependency `foo`
         ——▶ justfile:1:6
          │
        1 │ bar: foo
          │      ^^^
      ",
    )
    .failure();
}

#[test]
fn allow_missing_does_not_apply_to_other_subcommands() {
  Test::new()
    .justfile("")
    .args(["--allow-missing", "--show", "foo"])
    .stderr("error: justfile does not contain recipe `foo`\n")
    .failure();
}

#[test]
fn allow_missing_ignores_absent_optional_module() {
  Test::new()
    .justfile("mod? foo")
    .args(["--allow-missing", "foo::bar"])
    .success();
}
