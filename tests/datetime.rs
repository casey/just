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
    .run();
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
    .run();
}
