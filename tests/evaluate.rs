use super::*;

#[test]
fn evaluate() {
  Test::new()
    .arg("--evaluate")
    .justfile(r#"
foo := "a\t"
hello := "c"
bar := "b\t"
ab := foo + bar + hello

wut:
  touch /this/is/not/a/file
"#)
    .stdout(r#"ab    := "a	b	c"
bar   := "b	"
foo   := "a	"
hello := "c"
"#)
    .run();
}

#[test]
fn evaluate_empty() {
  Test::new()
    .arg("--evaluate")
    .justfile("
    a := 'foo'
  ")
    .stdout(r#"
    a := "foo"
  "#)
    .run();
}

test! {
  name:     evaluate_multiple,
  justfile: "
    a := 'x'
    b := 'y'
    c := 'z'
  ",
  args:   ("--evaluate", "a", "c"),
  stderr: "error: `--evaluate` used with unexpected argument: `c`\n",
  status: EXIT_FAILURE,
}

test! {
  name:     evaluate_single_free,
  justfile: "
    a := 'x'
    b := 'y'
    c := 'z'
  ",
  args:   ("--evaluate", "b"),
  stdout: "y",
}

test! {
  name:     evaluate_no_suggestion,
  justfile: "
    abc := 'x'
  ",
  args:   ("--evaluate", "aby"),
  stderr: "
    error: Justfile does not contain variable `aby`.
    Did you mean `abc`?
  ",
  status: EXIT_FAILURE,
}

test! {
  name:     evaluate_suggestion,
  justfile: "
    hello := 'x'
  ",
  args:   ("--evaluate", "goodbye"),
  stderr: "
    error: Justfile does not contain variable `goodbye`.
  ",
  status: EXIT_FAILURE,
}

test! {
  name:    evaluate_private,
  justfile: "
    [private]
    foo := 'one'
    bar := 'two'
    _baz := 'three'
  ",
  args:   ("--evaluate"),
  stdout: "bar  := \"two\"\n",
  status: EXIT_SUCCESS,
}

test! {
  name:    evaluate_single_private,
  justfile: "
    [private]
    foo := 'one'
    bar := 'two'
    _baz := 'three'
  ",
  args:   ("--evaluate", "foo"),
  stdout: "one",
  status: EXIT_SUCCESS,
}
