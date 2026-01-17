use super::*;

#[test]
fn assert_pass() {
  Test::new()
    .justfile(
      "
        foo:
          {{ assert('a' == 'a', 'error message') }}
      ",
    )
    .success();
}

#[test]
fn assert_fail() {
  Test::new()
    .justfile(
      "
        foo:
          {{ assert('a' != 'a', 'error message') }}
      ",
    )
    .stderr(
      "
        error: Assert failed: error message
         ——▶ justfile:2:6
          │
        2 │   {{ assert('a' != 'a', 'error message') }}
          │      ^^^^^^
      ",
    )
    .failure();
}
