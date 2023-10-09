use super::*;

#[test]
fn once() {
  Test::new()
    .justfile("x := 'a' / 'b'")
    .args(["--evaluate", "x"])
    .stdout("a/b")
    .run();
}

#[test]
fn twice() {
  Test::new()
    .justfile("x := 'a' / 'b' / 'c'")
    .args(["--evaluate", "x"])
    .stdout("a/b/c")
    .run();
}

#[test]
fn no_lhs_once() {
  Test::new()
    .justfile("x := / 'a'")
    .args(["--evaluate", "x"])
    .stdout("/a")
    .run();
}

#[test]
fn no_lhs_twice() {
  Test::new()
    .justfile("x := / 'a' / 'b'")
    .args(["--evaluate", "x"])
    .stdout("/a/b")
    .run();
  Test::new()
    .justfile("x := // 'a'")
    .args(["--evaluate", "x"])
    .stdout("//a")
    .run();
}

#[test]
fn no_rhs_once() {
  Test::new()
    .justfile("x := 'a' /")
    .stderr(
      "
      error: Expected backtick, identifier, '(', '/', or string, but found end of file
        |
      1 | x := 'a' /
        |           ^
    ",
    )
    .status(EXIT_FAILURE)
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
      error: Expected '*', ':', '$', identifier, or '+', but found '/'
        |
      1 | foo x='a' / 'b':
        |           ^
    ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn no_lhs_un_parenthesized() {
  Test::new()
    .justfile(
      "
      foo x=/ 'a' / 'b':
        echo {{x}}
    ",
    )
    .stderr(
      "
      error: Expected backtick, identifier, '(', or string, but found '/'
        |
      1 | foo x=/ 'a' / 'b':
        |       ^
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

#[test]
fn no_lhs_parenthesized() {
  Test::new()
    .justfile(
      "
      foo x=(/ 'a' / 'b'):
        echo {{x}}
    ",
    )
    .stderr("echo /a/b\n")
    .stdout("/a/b\n")
    .run();
}
