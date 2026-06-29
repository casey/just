use super::*;

#[test]
fn once() {
  assert_eval("'a' / 'b'", "a/b");
}

#[test]
fn twice() {
  assert_eval("'a' / 'b' / 'c'", "a/b/c");
}

#[test]
fn no_lhs_once() {
  assert_eval("/ 'a'", "/a");
}

#[test]
fn no_lhs_twice() {
  assert_eval("/ 'a' / 'b'", "/a/b");
  assert_eval("// 'a'", "//a");
}

#[test]
fn no_rhs_once() {
  Test::new()
    .justfile("x := 'a' /")
    .stderr(
      "
        error: expected backtick, '!', '[', identifier, '(', '/', or string, but found end of file
         ——▶ justfile:1:11
          │
        1 │ x := 'a' /
          │           ^
      ",
    )
    .failure();
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
        error: expected '*', ':', '$', identifier, or '+', but found '/'
         ——▶ justfile:1:11
          │
        1 │ foo x='a' / 'b':
          │           ^
      ",
    )
    .failure();
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
        error: expected backtick, '!', '[', identifier, '(', or string, but found '/'
         ——▶ justfile:1:7
          │
        1 │ foo x=/ 'a' / 'b':
          │       ^
      ",
    )
    .failure();
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
    .success();
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
    .success();
}
