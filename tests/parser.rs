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
fn bugfix_parameters() {
  Test::new()
    .justfile(
      "
        foo a b:
        bar a b: (foo a 'c')
      ",
    )
    .args(["bar", "A", "B"])
    .run();
}
