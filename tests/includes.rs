use super::*;
use temptree::temptree;

#[test]
fn include_fails_without_unstable() {
  Test::new()
    .justfile("!include ./include.justfile")
    .status(EXIT_FAILURE)
    .stderr("error: The !include directive is currently unstable. Invoke `just` with the `--unstable` flag to enable unstable features.\n")
    .run();
}

#[test]
fn include_succeeds_with_unstable() {
  Test::new()
    .tree(tree! {
      "include.justfile": "
        b:
          @echo B
      ",
    })
    .justfile(
      "
        !include ./include.justfile

        a: b
          @echo A
      ",
    )
    .arg("--unstable")
    .test_round_trip(false)
    .arg("a")
    .stdout("B\nA\n")
    .run();
}

#[test]
fn trailing_spaces_after_include_are_ignored() {
  Test::new()
    .tree(tree! {
      "include.justfile": "
        a:
          @echo A
      ",
    })
    .justfile("!include ./include.justfile\x20")
    .arg("--unstable")
    .test_round_trip(false)
    .stdout("A\n")
    .run();
}

#[test]
fn include_directive_with_no_path() {
  Test::new()
    .justfile("!include")
    .arg("--unstable")
    .status(EXIT_FAILURE)
    .stderr_regex("error: !include statement in .* line 1 has no argument\n")
    .run();
}

#[test]
fn trailing_include() {
  Test::new()
    .justfile(
      "
      b:
      !include ./include.justfile
      ",
    )
    .arg("--unstable")
    .status(EXIT_FAILURE)
    .stderr("error: Expected character `=`\n  |\n2 | !include ./include.justfile\n  |  ^\n")
    .run();
}

#[test]
fn circular_include() {
  Test::new()
    .justfile("!include a")
    .tree(tree! {
      a: "!include b",
      b: "!include a",
    })
    .arg("--unstable")
    .status(EXIT_FAILURE)
    .stderr_regex("error: Include `.*/a` in `.*/b` is a circular include")
    .run();
}
