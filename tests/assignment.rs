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
    error: Expected keyword `true` or `false` but found identifier `fals`
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
    error: Expected identifier, but found end of line
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
        error: Assignment `x` has invalid attribute `group`
         ——▶ justfile:2:1
          │
        2 │ x := 'foo'
          │ ^
      ",
    )
    .failure();
}
