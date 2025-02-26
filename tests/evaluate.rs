use super::*;

#[test]
fn evaluate() {
  Test::new()
    .arg("--evaluate")
    .justfile(
      r#"
foo := "a\t"
hello := "c"
bar := "b\t"
ab := foo + bar + hello

wut:
  touch /this/is/not/a/file
"#,
    )
    .stdout(
      r#"ab    := "a	b	c"
bar   := "b	"
foo   := "a	"
hello := "c"
"#,
    )
    .run();
}

#[test]
fn evaluate_empty() {
  Test::new()
    .arg("--evaluate")
    .justfile(
      "
    a := 'foo'
  ",
    )
    .stdout(
      r#"
    a := "foo"
  "#,
    )
    .run();
}

#[test]
fn evaluate_multiple() {
  Test::new()
    .arg("--evaluate")
    .arg("a")
    .arg("c")
    .justfile(
      "
    a := 'x'
    b := 'y'
    c := 'z'
  ",
    )
    .stderr("error: `--evaluate` used with unexpected argument: `c`\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn evaluate_single_free() {
  Test::new()
    .arg("--evaluate")
    .arg("b")
    .justfile(
      "
    a := 'x'
    b := 'y'
    c := 'z'
  ",
    )
    .stdout("y")
    .run();
}

#[test]
fn evaluate_no_suggestion() {
  Test::new()
    .arg("--evaluate")
    .arg("aby")
    .justfile(
      "
    abc := 'x'
  ",
    )
    .status(EXIT_FAILURE)
    .stderr(
      "
    error: Justfile does not contain variable `aby`.
    Did you mean `abc`?
  ",
    )
    .run();
}

#[test]
fn evaluate_suggestion() {
  Test::new()
    .arg("--evaluate")
    .arg("goodbye")
    .justfile(
      "
    hello := 'x'
  ",
    )
    .status(EXIT_FAILURE)
    .stderr(
      "
    error: Justfile does not contain variable `goodbye`.
  ",
    )
    .run();
}

#[test]
fn evaluate_private() {
  Test::new()
    .arg("--evaluate")
    .justfile(
      "
    [private]
    foo := 'one'
    bar := 'two'
    _baz := 'three'
  ",
    )
    .stdout("bar  := \"two\"\n")
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn evaluate_single_private() {
  Test::new()
    .arg("--evaluate")
    .arg("foo")
    .justfile(
      "
    [private]
    foo := 'one'
    bar := 'two'
    _baz := 'three'
  ",
    )
    .stdout("one")
    .status(EXIT_SUCCESS)
    .run();
}
