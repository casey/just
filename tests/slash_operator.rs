use super::*;

#[test]
fn once() {
  Test::new()
    .justfile("x := 'a' / 'b'")
    .args(["--evaluate", "x"])
    .stdout("a/b")
    .run_success();
}

#[test]
fn twice() {
  Test::new()
    .justfile("x := 'a' / 'b' / 'c'")
    .args(["--evaluate", "x"])
    .stdout("a/b/c")
    .run_success();
}

#[test]
fn no_lhs_once() {
  Test::new()
    .justfile("x := / 'a'")
    .args(["--evaluate", "x"])
    .stdout("/a")
    .run_success();
}

#[test]
fn no_lhs_twice() {
  Test::new()
    .justfile("x := / 'a' / 'b'")
    .args(["--evaluate", "x"])
    .stdout("/a/b")
    .run_success();
  Test::new()
    .justfile("x := // 'a'")
    .args(["--evaluate", "x"])
    .stdout("//a")
    .run_success();
}

#[test]
fn no_rhs_once() {
  Test::new()
    .justfile("x := 'a' /")
    .stderr(
      "
      error: Expected backtick, identifier, '(', '/', or string, but found end of file
       ——▶ justfile:1:11
        │
      1 │ x := 'a' /
        │           ^
    ",
    )
    .run_failure();
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
       ——▶ justfile:1:11
        │
      1 │ foo x='a' / 'b':
        │           ^
    ",
    )
    .run_failure();
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
       ——▶ justfile:1:7
        │
      1 │ foo x=/ 'a' / 'b':
        │       ^
    ",
    )
    .run_failure();
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
    .run_success();
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
    .run_success();
}
