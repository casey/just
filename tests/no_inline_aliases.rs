use super::*;

#[test]
fn no_inline_aliases_flag() {
  Test::new()
    .justfile(
      "
      alias t := test1

      test1:
        @echo 'test1'

      test2:
        @echo 'test2'
      ",
    )
    .args(["--no-inline-aliases", "--list"])
    .stdout("Available recipes:\n    test1\n    t     # alias for `test1`\n    test2\n")
    .run();
}
