use super::*;

#[test]
fn recipe_nested_module_dependencies() {
  Test::new()
    .write("foo.just", "mod bar\nbaz: \n @echo FOO")
    .write("bar.just", "baz:\n @echo BAZ")
    .justfile(
      "
      mod foo

      baz: foo::bar::baz
      ",
    )
    .arg("baz")
    .stdout("BAZ\n")
    .run();
}

#[test]
fn recipe_nested_module_dependencies2() {
  Test::new()
    .write("foo.just", "mod bar\nbaz: \n @echo BAR")
    .write("bar.just", "baz:\n @echo BAZ")
    .justfile(
      "
      mod foo
      baz: foo::baz
      ",
    )
    .arg("baz")
    .stdout("BAR\n")
    .run();
}

#[test]
fn recipe_dependency_on_module_fails() {
  Test::new()
    .write("foo.just", "mod bar\nbaz: \n @echo BAR")
    .write("bar.just", "baz:\n @echo BAZ")
    .justfile(
      "
      mod foo
      baz: foo::bar
      ",
    )
    .arg("baz")
    .status(1)
    .stderr(
      "error: Recipe `baz` has unknown dependency `foo::bar`
 ——▶ justfile:2:11
  │
2 │ baz: foo::bar
  │           ^^^
",
    )
    .run();
}

#[test]
fn recipe_module_dependency_subsequent_mix() {
  Test::new()
    .write("foo.just", "bar: \n @echo BAR")
    .justfile(
      "
      mod foo
      baz :
        @echo BAZ
      quux: foo::bar && baz
        @echo QUUX
      ",
    )
    .arg("quux")
    .stdout("BAR\nQUUX\nBAZ\n")
    .run();
}

#[test]
fn recipe_module_dependency_only_runs_once() {
  Test::new()
    .write("foo.just", "bar: baz \n  \nbaz: \n @echo BAZ")
    .justfile(
      "
      mod foo
      qux: foo::bar foo::baz
      ",
    )
    .arg("qux")
    .stdout("BAZ\n")
    .run();
}
