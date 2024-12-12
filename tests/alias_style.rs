use super::*;

#[test]
fn default() {
  Test::new()
    .justfile(
      "
        alias f := foo

        # comment
        foo:

        bar:
      ",
    )
    .args(["--list"])
    .stdout(
      "
        Available recipes:
            bar
            foo # comment [alias: f]
      ",
    )
    .run();
}

#[test]
fn multiple() {
  Test::new()
    .justfile(
      "
        alias a := foo
        alias b := foo

        # comment
        foo:

        bar:
      ",
    )
    .args(["--list"])
    .stdout(
      "
        Available recipes:
            bar
            foo # comment [aliases: a, b]
      ",
    )
    .run();
}

#[test]
fn inline() {
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
            foo # comment [alias: f]
      ",
    )
    .run();
}

#[test]
fn inline_left() {
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
            foo # [alias: f] comment
      ",
    )
    .run();
}

#[test]
fn recipe() {
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
