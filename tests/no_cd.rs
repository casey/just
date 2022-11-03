use super::*;

#[test]
fn linewise() {
  Test::new()
    .justfile(
      "
      [no-cd]
      foo:
        cat bar
    ",
    )
    .current_dir("foo")
    .tree(tree! {
      foo: {
        bar: "hello",
      }
    })
    .stderr("cat bar\n")
    .stdout("hello")
    .run();
}

#[test]
fn shebang() {
  Test::new()
    .justfile(
      "
      [no-cd]
      foo:
        #!/bin/sh
        cat bar
    ",
    )
    .current_dir("foo")
    .tree(tree! {
      foo: {
        bar: "hello",
      }
    })
    .stdout("hello")
    .run();
}
