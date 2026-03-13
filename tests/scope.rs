use super::*;

#[test]
fn dependencies_in_submodules_run_with_submodule_scope() {
  Test::new()
    .write("bar.just", "x := 'X'\nbar a=x:\n echo {{ a }} {{ x }}")
    .justfile(
      "
        mod bar

        foo: bar::bar
      ",
    )
    .stdout("X X\n")
    .stderr("echo X X\n")
    .success();
}

#[test]
fn aliases_in_submodules_run_with_submodule_scope() {
  Test::new()
    .write("bar.just", "x := 'X'\nbar a=x:\n echo {{ a }} {{ x }}")
    .justfile(
      "
        mod bar

        alias foo := bar::bar
      ",
    )
    .arg("foo")
    .stdout("X X\n")
    .stderr("echo X X\n")
    .success();
}

#[test]
fn dependencies_in_nested_submodules_run_with_submodule_scope() {
  Test::new()
    .write(
      "b.just",
      "
x := 'y'

foo:
    @echo {{ x }}
",
    )
    .write("a.just", "mod b")
    .stdout("y\n")
    .justfile("mod a")
    .args(["a", "b", "foo"])
    .success();
}

#[test]
fn imported_recipes_run_in_correct_scope() {
  Test::new()
    .justfile(
      "
        mod a
        mod b
      ",
    )
    .write("a.just", "X := 'A'\nimport 'shared.just'")
    .write("b.just", "X := 'B'\nimport 'shared.just'")
    .write("shared.just", "foo:\n @echo {{ X }}")
    .args(["a::foo", "b::foo"])
    .stdout("A\nB\n")
    .success();
}
