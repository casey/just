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
    .run();
}

#[test]
fn invalid_bang_operator() {
  Test::new()
    .justfile(
      "
      x := if '' !! '' { '' } else { '' }
      ",
    )
    .run();
}
