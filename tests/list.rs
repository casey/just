use super::*;

#[test]
fn list_displays_recipes_in_submodules() {
  Test::new()
    .write("foo.just", "bar:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("--list")
    .stdout(
      "
      Available recipes:
          foo:
              bar
    ",
    )
    .run();
}

#[test]
fn modules_are_space_separated_in_output() {
  Test::new()
    .write("foo.just", "foo:")
    .write("bar.just", "bar:")
    .justfile(
      "
        mod foo

        mod bar
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("--list")
    .stdout(
      "
      Available recipes:
          bar:
              bar

          foo:
              foo
    ",
    )
    .run();
}

#[test]
fn module_recipe_list_alignment_ignores_private_recipes() {
  Test::new()
    .write(
      "foo.just",
      "
# foos
foo:
 @echo FOO

[private]
barbarbar:
 @echo BAR

@_bazbazbaz:
 @echo BAZ
      ",
    )
    .justfile("mod foo")
    .test_round_trip(false)
    .arg("--unstable")
    .arg("--list")
    .stdout(
      "
        Available recipes:
            foo:
                foo # foos
      ",
    )
    .run();
}

#[test]
fn nested_modules_are_properly_indented() {
  Test::new()
    .write("foo.just", "mod bar")
    .write("bar.just", "baz:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("--list")
    .stdout(
      "
      Available recipes:
          foo:
              bar:
                  baz
    ",
    )
    .run();
}
