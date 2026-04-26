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
        error: expected character `=` or `~`
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
        error: expected character `=` or `~` but found end-of-file
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

#[test]
fn attribute_without_item() {
  Test::new()
    .justfile(
      "
        [confirm]
      ",
    )
    .stderr(
      "
        error: expected '@', '[', comment, end of line, or identifier, but found end of file
         ——▶ justfile:1:11
          │
        1 │ [confirm]
          │          ^
      ",
    )
    .failure();
}
