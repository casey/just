use super::*;

#[test]
fn alias_style_inline() {
  Test::new()
    .justfile(
      "
      alias t := test1

      # A test recipe
      test1:
        @echo 'test1'

      test2:
        @echo 'test2'
      ",
    )
    .args(["--alias-style=inline", "--list"])
    .stdout("Available recipes:\n    test1 # A test recipe [aliases: t]\n    test2\n")
    .run();
}

#[test]
fn alias_style_inline_left() {
  Test::new()
    .justfile(
      "
      alias t := test1

      # A test recipe
      test1:
        @echo 'test1'

      test2:
        @echo 'test2'
      ",
    )
    .args(["--alias-style=inline-left", "--list"])
    .stdout("Available recipes:\n    test1 # [aliases: t] A test recipe\n    test2\n")
    .run();
}

#[test]
fn alias_style_recipe() {
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
    .args(["--alias-style=recipe", "--list"])
    .stdout("Available recipes:\n    test1\n    t     # alias for `test1`\n    test2\n")
    .run();
}
