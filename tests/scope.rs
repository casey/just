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
    .run();
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
    .run();
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
    .run();
}
