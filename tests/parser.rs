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

#[test]
fn backslash_eof() {
  Test::new()
    .justfile("foo:\n\\")
    .stderr(
      "
        error: expected escape sequence but found end-of-file
         ——▶ justfile:2:1
          │
        2 │ \\
          │ ^
      ",
    )
    .failure();
}
