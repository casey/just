use super::*;

#[test]
fn dont_run_duplicate_recipes() {
  Test::new()
    .justfile(
      "
      foo:
        # foo
    ",
    )
    .args(["foo", "foo"])
    .stderr(
      "
      # foo
    ",
    )
    .run();
}
