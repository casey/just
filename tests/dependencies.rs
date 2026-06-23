use super::*;

#[test]
fn recipe_doubly_nested_module_dependencies() {
  Test::new()
    .write(
      "foo.just",
      unindent(
        "
          mod bar
          baz:
           @echo FOO
        ",
      ),
    )
    .write(
      "bar.just",
      unindent(
        "
          baz:
           @echo BAZ
        ",
      ),
    )
    .justfile(
      "
        mod foo

        baz: foo::bar::baz
      ",
    )
    .arg("baz")
    .stdout("BAZ\n")
    .success();
}

#[test]
fn recipe_singly_nested_module_dependencies() {
  Test::new()
    .write(
      "foo.just",
      unindent(
        "
          mod bar
          baz:
           @echo BAR
        ",
      ),
    )
    .write(
      "bar.just",
      unindent(
        "
          baz:
           @echo BAZ
        ",
      ),
    )
    .justfile(
      "
        mod foo
        baz: foo::baz
      ",
    )
    .arg("baz")
    .stdout("BAR\n")
    .success();
}

#[test]
fn dependency_not_in_submodule() {
  Test::new()
    .write(
      "foo.just",
      unindent(
        "
          qux:
           @echo QUX
        ",
      ),
    )
    .justfile(
      "
        mod foo
        baz: foo::baz
      ",
    )
    .arg("baz")
    .stderr(
      "error: recipe `baz` has unknown dependency `foo::baz`
 ——▶ justfile:2:11
  │
2 │ baz: foo::baz
  │           ^^^
",
    )
    .failure();
}

#[test]
fn dependency_submodule_missing() {
  Test::new()
    .justfile(
      "
        foo:
          @echo FOO
        bar:
          @echo BAR
        baz: foo::bar
      ",
    )
    .arg("baz")
    .stderr(
      "error: recipe `baz` has unknown dependency `foo::bar`
 ——▶ justfile:5:11
  │
5 │ baz: foo::bar
  │           ^^^
",
    )
    .failure();
}

#[test]
fn recipe_dependency_on_module_fails() {
  Test::new()
    .write(
      "foo.just",
      unindent(
        "
          mod bar
          baz:
           @echo BAR
        ",
      ),
    )
    .write(
      "bar.just",
      unindent(
        "
          baz:
           @echo BAZ
        ",
      ),
    )
    .justfile(
      "
        mod foo
        baz: foo::bar
      ",
    )
    .arg("baz")
    .stderr(
      "error: recipe `baz` has unknown dependency `foo::bar`
 ——▶ justfile:2:11
  │
2 │ baz: foo::bar
  │           ^^^
",
    )
    .failure();
}

#[test]
fn recipe_module_dependency_subsequent_mix() {
  Test::new()
    .write(
      "foo.just",
      unindent(
        "
          bar:
           @echo BAR
        ",
      ),
    )
    .justfile(
      "
        mod foo
        baz:
          @echo BAZ
        quux: foo::bar && baz
          @echo QUUX
      ",
    )
    .arg("quux")
    .stdout("BAR\nQUUX\nBAZ\n")
    .success();
}

#[test]
fn recipe_module_dependency_only_runs_once() {
  Test::new()
    .write(
      "foo.just",
      unindent(
        "
          bar: baz

          baz:
           @echo BAZ
        ",
      ),
    )
    .justfile(
      "
        mod foo
        qux: foo::bar foo::baz
      ",
    )
    .arg("qux")
    .stdout("BAZ\n")
    .success();
}
