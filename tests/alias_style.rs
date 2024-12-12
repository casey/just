use super::*;

#[test]
fn alias_style_inline() {
  Test::new()
    .justfile(
      "
        alias f := foo

        # comment
        foo:

        bar:
      ",
    )
    .args(["--alias-style=inline", "--list"])
    .stdout(
      "
        Available recipes:
            bar
            foo # comment [aliases: f]
      ",
    )
    .run();
}

#[test]
fn alias_style_inline_left() {
  Test::new()
    .justfile(
      "
        alias f := foo

        # comment
        foo:

        bar:
      ",
    )
    .args(["--alias-style=inline-left", "--list"])
    .stdout(
      "
        Available recipes:
            bar
            foo # [aliases: f] comment
      ",
    )
    .run();
}

#[test]
fn alias_style_recipe() {
  Test::new()
    .justfile(
      "
        alias f := foo

        # comment
        foo:

        bar:
      ",
    )
    .args(["--alias-style=recipe", "--list"])
    .stdout(
      "
        Available recipes:
            bar
            foo # comment
            f   # alias for `foo`
      ",
    )
    .run();
}
