use super::*;

#[test]
fn foo() {
  Test::new()
    .justfile(
      "
      foo
        bar:
        @echo bar
    ",
    )
    .stdout("bar\n")
    .run();
}
