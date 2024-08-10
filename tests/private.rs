use super::*;

#[test]
fn private_attribute_for_recipe() {
  Test::new()
    .justfile(
      "
      [private]
      foo:
      ",
    )
    .args(["--list"])
    .stdout(
      "
      Available recipes:
      ",
    )
    .run();
}

#[test]
fn private_attribute_for_alias() {
  Test::new()
    .justfile(
      "
      [private]
      alias f := foo

      foo:
      ",
    )
    .args(["--list"])
    .stdout(
      "
      Available recipes:
          foo
      ",
    )
    .run();
}

#[test]
fn private_variables_are_not_listed() {
  Test::new()
    .justfile(
      "
      [private]
      foo := 'one'
      bar := 'two'
      _baz := 'three'
      ",
    )
    .args(["--variables"])
    .stdout("bar\n")
    .run();
}
