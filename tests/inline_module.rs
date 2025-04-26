use super::*;

#[test]
fn alias_nested_module() {
  Test::new()
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
    .run();
}
