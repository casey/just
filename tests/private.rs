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

test! {
  name: dont_list_private_variables,
  justfile: "
      set allow-private-variables
      [private]
      foo := 'one'
      bar := 'two'
      _baz := 'three'
      ",
   args: ("--variables"),
   stdout: "bar\n",
}

test! {
  name: list_private_variables_if_not_opted_in,
  justfile: "
      [private]
      foo := 'one'
      bar := 'two'
      _baz := 'three'
      ",
   args: ("--variables"),
   stdout: "_baz bar foo\n",
}

test! {
  name: allows_private_overrides,
  justfile: "
      set allow-private-variables

      [private]
      foo := 'one'
      bar := 'two'
      _baz := 'three'

      default:
        @echo {{foo}}
  ",
  args: ("foo=two"),
  stdout: "two\n",
  status: EXIT_SUCCESS,
}

test! {
  name: allows_implicit_private_overrides,
  justfile: "
      set allow-private-variables

      [private]
      foo := 'one'
      bar := 'two'
      _baz := 'three'

      default:
        @echo {{_baz}}
  ",
  args: ("_baz=two"),
  stdout: "two\n",
  status: EXIT_SUCCESS,
}

test! {
  name: allowed_public_overrides,
  justfile: "
      set allow-private-variables

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

test! {
  name: ignore_private_without_setting,
  justfile: "
      [private]
      foo := 'one'
      bar := 'two'
      _baz := 'three'

      default:
        @echo hello
  ",
  args: ("foo=two"),
  stdout: "hello\n",
  stderr: "",
  status: EXIT_SUCCESS,
}
