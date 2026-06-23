use super::*;

#[test]
fn alias_nested_module() {
  Test::new()
    .write(
      "foo.just",
      "
        mod bar
        baz:
         @echo FOO
      ",
    )
    .write(
      "bar.just",
      "
        baz:
         @echo BAZ
      ",
    )
    .justfile(
      "
        mod foo

        alias b := foo::bar::baz

        baz:
          @echo 'HERE'
      ",
    )
    .arg("b")
    .stdout("BAZ\n")
    .success();
}

#[test]
fn unknown_nested_alias() {
  Test::new()
    .write(
      "foo.just",
      "
        baz:
         @echo FOO
      ",
    )
    .justfile(
      "
        mod foo

        alias b := foo::bar::baz
      ",
    )
    .arg("b")
    .stderr(
      "\
        error: alias `b` has an unknown target `foo::bar::baz`
 ——▶ justfile:3:7
  │
3 │ alias b := foo::bar::baz
  │       ^
",
    )
    .failure();
}

#[test]
fn alias_in_submodule() {
  Test::new()
    .write(
      "foo.just",
      "
        alias b := bar

        bar:
          @echo BAR
      ",
    )
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo::b")
    .stdout("BAR\n")
    .success();
}

#[test]
fn module_alias() {
  Test::new()
    .write("foo.just", "bar:\n @echo BAR")
    .justfile(
      "
        mod foo

        alias f := foo
      ",
    )
    .arg("f")
    .arg("bar")
    .stdout("BAR\n")
    .success();
}

#[test]
fn module_alias_default_recipe() {
  Test::new()
    .write("foo.just", "bar:\n @echo BAR")
    .justfile(
      "
        mod foo

        alias f := foo
      ",
    )
    .arg("f")
    .stdout("BAR\n")
    .success();
}

#[test]
fn module_alias_argument() {
  Test::new()
    .write("foo.just", "bar baz:\n @echo {{baz}}")
    .justfile(
      "
        mod foo

        alias f := foo
      ",
    )
    .arg("f")
    .arg("bar")
    .arg("qux")
    .stdout("qux\n")
    .success();
}

#[test]
fn module_alias_nested() {
  Test::new()
    .write("foo/mod.just", "mod bar")
    .write("foo/bar.just", "baz:\n @echo BAZ")
    .justfile(
      "
        mod foo

        alias f := foo::bar
      ",
    )
    .arg("f")
    .arg("baz")
    .stdout("BAZ\n")
    .success();
}

#[test]
fn module_alias_not_listed() {
  Test::new()
    .write("foo.just", "bar:\n @echo BAR")
    .justfile(
      "
        mod foo

        alias f := foo
      ",
    )
    .arg("--list")
    .stdout(
      "
        Available recipes:
            foo ...
      ",
    )
    .success();
}

#[test]
fn module_alias_conflicts_with_module() {
  Test::new()
    .write("foo.just", "bar:")
    .justfile(
      "
        mod foo

        alias foo := foo
      ",
    )
    .arg("--list")
    .stderr(
      "\
error: module `foo` defined on line 1 is redefined as an alias on line 3
 ——▶ justfile:3:7
  │
3 │ alias foo := foo
  │       ^^^
",
    )
    .failure();
}
