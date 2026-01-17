use super::*;

#[test]
fn alias_nested_module() {
  Test::new()
    .write("foo.just", "mod bar\nbaz: \n @echo FOO")
    .write("bar.just", "baz:\n @echo BAZ")
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
    .write("foo.just", "baz: \n @echo FOO")
    .justfile(
      "
      mod foo

      alias b := foo::bar::baz
      ",
    )
    .arg("b")
    .stderr(
      "\
        error: Alias `b` has an unknown target `foo::bar::baz`
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
