use super::*;

#[test]
fn once() {
  Test::new()
    .justfile("x := 'a' / 'b'")
    .args(&["--evaluate", "x"])
    .stdout("a/b")
    .run();
}

#[test]
fn twice() {
  Test::new()
    .justfile("x := 'a' / 'b' / 'c'")
    .args(&["--evaluate", "x"])
    .stdout("a/b/c")
    .run();
}

#[test]
fn default_un_parenthesized() {
  Test::new()
    .justfile(
      "
      foo x='a' / 'b':
        echo {{x}}
    ",
    )
    .stderr(
      "
      error: Expected '*', ':', '$', end of line, identifier, or '+', but found '/'
        |
      1 | foo x='a' / 'b':
        |           ^
    ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn default_parenthesized() {
  Test::new()
    .justfile(
      "
      foo x=('a' / 'b'):
        echo {{x}}
    ",
    )
    .stderr("echo a/b\n")
    .stdout("a/b\n")
    .run();
}
