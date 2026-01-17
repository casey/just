use super::*;

#[test]
fn dont_run_duplicate_recipes() {
  Test::new()
    .justfile(
      "
      set dotenv-load # foo
      bar:
      ",
    )
    .run_success();
}

#[test]
fn invalid_bang_operator() {
  Test::new()
    .justfile(
      "
      x := if '' !! '' { '' } else { '' }
      ",
    )
    .stderr(
      r"
error: Expected character `=` or `~`
 ——▶ justfile:1:13
  │
1 │ x := if '' !! '' { '' } else { '' }
  │             ^
",
    )
    .run_failure();
}

#[test]
fn truncated_bang_operator() {
  Test::new()
    .justfile("x := if '' !")
    .stderr(
      r"
error: Expected character `=` or `~` but found end-of-file
 ——▶ justfile:1:13
  │
1 │ x := if '' !
  │             ^
",
    )
    .run_failure();
}
