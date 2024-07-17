use super::*;

#[test]
fn basic() {
  Test::new()
    .justfile(
      "
        [script('sh', '-u')]
        foo:
          echo FOO

      ",
    )
    .stdout("FOO\n")
    .run();
}
