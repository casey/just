use super::*;

#[test]
fn recipe_dependency_nested_module() {
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
fn recipe_dependency_nested_module2() {
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
