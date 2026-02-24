use super::*;

#[test]
fn constants_are_defined() {
  assert_eval_eq("HEX", "0123456789abcdef");
}

#[test]
fn constants_can_have_different_values_on_windows() {
  assert_eval_eq("PATH_SEP", if cfg!(windows) { "\\" } else { "/" });
  assert_eval_eq("PATH_VAR_SEP", if cfg!(windows) { ";" } else { ":" });
}

#[test]
fn constants_are_defined_in_recipe_bodies() {
  Test::new()
    .justfile(
      "
        @foo:
          echo {{HEX}}
      ",
    )
    .stdout("0123456789abcdef\n")
    .success();
}

#[test]
fn constants_are_defined_in_recipe_parameters() {
  Test::new()
    .justfile(
      "
        @foo hex=HEX:
          echo {{hex}}
      ",
    )
    .stdout("0123456789abcdef\n")
    .success();
}

#[test]
fn constants_can_be_redefined() {
  Test::new()
    .justfile(
      "
        HEX := 'foo'
      ",
    )
    .args(["--evaluate", "HEX"])
    .stdout("foo")
    .success();
}

#[test]
fn constants_are_not_exported() {
  Test::new()
    .justfile(
      r#"
        set export

        foo:
          @'{{just_executable()}}' --request '{"environment-variable": "HEXUPPER"}'
      "#,
    )
    .response(Response::EnvironmentVariable(None))
    .success();
}
