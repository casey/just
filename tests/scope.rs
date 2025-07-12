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
