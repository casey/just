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
    .success();
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
    .success();
}

#[test]
fn private_attribute_for_module() {
  Test::new()
    .write("foo.just", "bar:")
    .justfile(
      r"
        [private]
        mod foo

        baz:
      ",
    )
    .test_round_trip(false)
    .arg("--list")
    .stdout(
      "
        Available recipes:
            baz
      ",
    )
    .success();
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
    .success();
}
