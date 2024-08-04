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
fn private_attribute_for_assignment() {
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

test! {
  name: no_private_overrides,
  justfile: "
      [private]
      foo := 'one'
      bar := 'two'
      _baz := 'three'

      default:
        @echo hello
  ",
  args: ("foo=two"),
  stdout: "",
  stderr: "error: Variable `foo` overridden on the command line but not present in justfile\n",
  status: EXIT_FAILURE,
}

test! {
  name: no_private_implicit_overrides,
  justfile: "
      [private]
      foo := 'one'
      bar := 'two'
      _baz := 'three'

      default:
        @echo hello
  ",
  args: ("_baz=two"),
  stdout: "",
  stderr: "error: Variable `_baz` overridden on the command line but not present in justfile\n",
  status: EXIT_FAILURE,
}

test! {
  name: allowed_public_overrides,
  justfile: "
      [private]
      foo := 'one'
      bar := 'two'
      _baz := 'three'

      default:
        @echo hello
  ",
  args: ("bar=two"),
  stdout: "hello\n",
  stderr: "",
  status: EXIT_SUCCESS,
}
