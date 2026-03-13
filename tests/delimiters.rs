use super::*;

#[test]
fn mismatched_delimiter() {
  Test::new()
    .justfile("(]")
    .stderr(
      "
    error: Mismatched closing delimiter `]`. (Did you mean to close the `(` on line 1?)
     ——▶ justfile:1:2
      │
    1 │ (]
      │  ^
  ",
    )
    .failure();
}

#[test]
fn unexpected_delimiter() {
  Test::new()
    .justfile("]")
    .stderr(
      "
    error: Unexpected closing delimiter `]`
     ——▶ justfile:1:1
      │
    1 │ ]
      │ ^
  ",
    )
    .failure();
}

#[test]
fn paren_continuation() {
  Test::new()
    .justfile(
      "
    x := (
          'a'
              +
      'b'
    )

    foo:
      echo {{x}}
  ",
    )
    .stdout("ab\n")
    .stderr("echo ab\n")
    .success();
}

#[test]
fn brace_continuation() {
  Test::new()
    .justfile(
      "
    x := if '' == '' {
      'a'
    } else {
      'b'
    }

    foo:
      echo {{x}}
  ",
    )
    .stdout("a\n")
    .stderr("echo a\n")
    .success();
}

#[test]
fn bracket_continuation() {
  Test::new()
    .justfile(
      "
    set shell := [
      'sh',
      '-cu',
    ]

    foo:
      echo foo
  ",
    )
    .stdout("foo\n")
    .stderr("echo foo\n")
    .success();
}

#[test]
fn dependency_continuation() {
  Test::new()
    .justfile(
      "
    foo: (
    bar 'bar'
    )
      echo foo

    bar x:
      echo {{x}}
  ",
    )
    .stdout("bar\nfoo\n")
    .stderr("echo bar\necho foo\n")
    .success();
}

#[test]
fn interpolation_continuation() {
  Test::new()
    .justfile(
      "
    foo:
      echo {{ (
        'a' + 'b')}}
  ",
    )
    .stderr("echo ab\n")
    .stdout("ab\n")
    .success();
}
