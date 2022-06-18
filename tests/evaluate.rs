use crate::common::*;

test! {
  name:     evaluate,
  justfile: r#"
foo := "a\t"
hello := "c"
bar := "b\t"
ab := foo + bar + hello

wut:
  touch /this/is/not/a/file
"#,
  args:     ("--evaluate"),
  stdout:   r#"ab    := "a	b	c"
bar   := "b	"
foo   := "a	"
hello := "c"
"#,
}

test! {
  name:     evaluate_empty,
  justfile: "
    a := 'foo'
  ",
  args:     ("--evaluate"),
  stdout:   r#"
    a := "foo"
  "#,
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
