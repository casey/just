use super::*;

#[test]
fn bare_bash_in_shebang() {
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
