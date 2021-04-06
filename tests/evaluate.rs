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
  name:     evaluate_arguments,
  justfile: "
    a := 'x'
    b := 'y'
    c := 'z'
  ",
  args:     ("--evaluate", "a", "c"),
  stdout:   r#"
    a := "x"
    c := "z"
  "#,
}
