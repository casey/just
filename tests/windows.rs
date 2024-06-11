use super::*;

#[test]
fn bug() {
  Test::new()
    .justfile(
      "
        default:
            #!bash
            echo FOO
      ",
    )
    .stdout("FOO\n")
    .run();
}
