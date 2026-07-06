use super::*;

#[test]
fn set_export_parse_error() {
  Test::new()
    .justfile(
      "
        set export := fals
      ",
    )
    .stderr(
      "
        error: expected keyword `true` or `false` but found identifier `fals`
         ——▶ justfile:1:15
          │
        1 │ set export := fals
          │               ^^^^
      ",
    )
    .failure();
}

#[test]
fn set_export_parse_error_eol() {
  Test::new()
    .justfile(
      "
        set export :=
      ",
    )
    .stderr(
      "
        error: expected identifier, but found end of line
         ——▶ justfile:1:14
          │
        1 │ set export :=
          │              ^
      ",
    )
    .failure();
}

#[test]
fn invalid_attributes_are_an_error() {
  Test::new()
    .justfile(
      "
        [group: 'bar']
        x := 'foo'
      ",
    )
    .args(["--evaluate", "x"])
    .stderr(
      "
        error: assignment `x` has invalid attribute `group`
         ——▶ justfile:2:1
          │
        2 │ x := 'foo'
          │ ^
      ",
    )
    .failure();
}

#[test]
fn local_variables_can_shadow_constants() {
  Test::new()
    .justfile(
      "
        a := HEX

        HEX := 'foo'
      ",
    )
    .args(["--evaluate", "a"])
    .stdout("foo")
    .success();
}

#[test]
fn non_const_variables_shadowing_constants_are_an_error_in_const_contexts() {
  Test::new()
    .justfile(
      "
        HEX := `echo foo`

        set tempdir := HEX
      ",
    )
    .stderr(
      "
        error: cannot access non-const variable `HEX` in const context
         ——▶ justfile:3:16
          │
        3 │ set tempdir := HEX
          │                ^^^
      ",
    )
    .failure();
}

#[test]
fn variables_shadowing_constants_can_be_overridden() {
  Test::new()
    .justfile(
      "
        a := HEX

        HEX := 'foo'
      ",
    )
    .args(["--set", "HEX", "bar", "--evaluate", "a"])
    .stdout("bar")
    .success();
}
