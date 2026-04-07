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
    .success();
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
      "
        error: Expected character `=` or `~`
        ——▶ justfile:1:13
          │
        1 │ x := if '' !! '' { '' } else { '' }
          │             ^
      ",
    )
    .failure();
}

#[test]
fn truncated_bang_operator() {
  Test::new()
    .justfile("x := if '' !")
    .stderr(
      "
        error: Expected character `=` or `~` but found end-of-file
        ——▶ justfile:1:13
          │
        1 │ x := if '' !
          │             ^
      ",
    )
    .failure();
}

#[test]
fn comment_after_unexport() {
  Test::new()
    .justfile(
      "
        unexport foo # bar

        baz:
      ",
    )
    .success();
}
