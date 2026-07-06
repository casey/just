use super::*;

#[test]
fn datetime() {
  Test::new()
    .justfile(
      "
        x := datetime('%Y-%m-%d %z')
      ",
    )
    .args(["--eval", "x"])
    .stdout_regex(r"\d\d\d\d-\d\d-\d\d [+-]\d\d\d\d")
    .success();
}

#[test]
fn datetime_utc() {
  Test::new()
    .justfile(
      "
        x := datetime_utc('%Y-%m-%d %Z')
      ",
    )
    .args(["--eval", "x"])
    .stdout_regex(r"\d\d\d\d-\d\d-\d\d UTC")
    .success();
}

#[test]
fn invalid_format_string_error() {
  Test::new()
    .justfile("x := datetime('%!')")
    .args(["--evaluate", "x"])
    .stderr(
      "
        error: call to function `datetime` failed: error: failed to parse time format string `%!`: bad or unsupported format string
         ——▶ justfile:1:6
          │
        1 │ x := datetime('%!')
          │      ^^^^^^^^
      ")
    .failure();
}

#[test]
fn parse_only_specifier_error() {
  Test::new()
    .justfile("x := datetime('%#z')")
    .arg("--evaluate")
    .stderr(
      "
        error: call to function `datetime` failed: error: failed to format time with format string `%#z`
         ——▶ justfile:1:6
          │
        1 │ x := datetime('%#z')
          │      ^^^^^^^^
      "
    )
    .failure();
}
