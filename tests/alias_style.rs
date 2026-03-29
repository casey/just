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
    .success();
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
    .success();
}

#[test]
fn right() {
  Test::new()
    .justfile(
      "
        alias f := foo

        # comment
        foo:

        bar:
      ",
    )
    .args(["--alias-style=right", "--list"])
    .stdout(
      "
        Available recipes:
            bar
            foo # comment [alias: f]
      ",
    )
    .success();
}

#[test]
fn left() {
  Test::new()
    .justfile(
      "
        alias f := foo

        # comment
        foo:

        bar:
      ",
    )
    .args(["--alias-style=left", "--list"])
    .stdout(
      "
        Available recipes:
            bar
            foo # [alias: f] comment
      ",
    )
    .success();
}

#[test]
fn separate() {
  Test::new()
    .justfile(
      "
        alias f := foo

        # comment
        foo:

        bar:
      ",
    )
    .args(["--alias-style=separate", "--list"])
    .stdout(
      "
        Available recipes:
            bar
            foo # comment
            f   # alias for `foo`
      ",
    )
    .success();
}
