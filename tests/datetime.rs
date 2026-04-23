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
    .stderr_regex("error: Call to function `datetime` failed: invalid format string `%!`: .*\n")
    .failure();
}
