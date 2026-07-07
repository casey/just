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
fn alias_to_absent_optional_module_is_disabled() {
  Test::new()
    .justfile(
      "
        mod? foo

        alias f := foo
      ",
    )
    .arg("f")
    .stderr("error: alias `f` depends on absent module `foo`\n")
    .failure();
}

#[test]
fn module_alias_resolves_in_show() {
  Test::new()
    .write("foo.just", "bar:\n @echo BAR")
    .justfile(
      "
        mod foo

        alias f := foo
      ",
    )
    .args(["--show", "f::bar"])
    .stdout("bar:\n    @echo BAR\n")
    .success();
}

#[test]
fn module_alias_resolves_in_list() {
  Test::new()
    .write("foo.just", "bar:\n @echo BAR")
    .justfile(
      "
        mod foo

        alias f := foo
      ",
    )
    .args(["--list", "f"])
    .stdout(
      "
        Available recipes:
            bar
      ",
    )
    .success();
}

#[test]
fn module_alias_resolves_in_usage() {
  Test::new()
    .write("foo.just", "bar:\n @echo BAR")
    .justfile(
      "
        mod foo

        alias f := foo
      ",
    )
    .args(["--usage", "f::bar"])
    .stdout("Usage: just f::bar\n")
    .success();
}
