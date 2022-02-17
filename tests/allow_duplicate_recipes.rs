use crate::common::*;

#[test]
fn allow_duplicate_recipes() {
  Test::new()
    .justfile(
      "
      b:
        echo foo
      b:
        echo bar

      set allow-duplicate-recipes
    ",
    )
    .stdout("bar\n")
    .stderr("echo bar\n")
    .run();
}

#[test]
fn allow_duplicate_recipes_with_args() {
  Test::new()
    .justfile(
      "
      b a:
        echo foo
      b c d:
        echo bar {{c}} {{d}}

      set allow-duplicate-recipes
    ",
    )
    .args(&["b", "one", "two"])
    .stdout("bar one two\n")
    .stderr("echo bar one two\n")
    .run();
}
