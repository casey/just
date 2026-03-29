use super::*;

#[test]
fn parameter_default_values_may_use_earlier_parameters() {
  Test::new()
    .justfile(
      "
        @foo a b=a:
          echo {{ b }}
      ",
    )
    .args(["foo", "bar"])
    .stdout("bar\n")
    .success();
}

#[test]
fn parameter_default_values_may_not_use_later_parameters() {
  Test::new()
    .justfile(
      "
        @foo a b=c c='':
          echo {{ b }}
      ",
    )
    .args(["foo", "bar"])
    .stderr(
      "
        error: Variable `c` not defined
         ——▶ justfile:1:10
          │
        1 │ @foo a b=c c='':
          │          ^
      ",
    )
    .failure();
}

#[test]
fn star_may_follow_default() {
  Test::new()
    .justfile(
      "
        foo bar='baz' *bob:
          @echo {{bar}} {{bob}}
      ",
    )
    .args(["foo", "hello", "goodbye"])
    .stdout("hello goodbye\n")
    .success();
}
