use crate::common::*;

test! {
  name:     raw_string,
  justfile: r#"
export EXPORTED_VARIABLE := '\z'

recipe:
  printf "$EXPORTED_VARIABLE"
"#,
  stdout:   "\\z",
  stderr:   "printf \"$EXPORTED_VARIABLE\"\n",
}

test! {
  name:     multiline_raw_string,
  justfile: "
string := 'hello
whatever'

a:
  echo '{{string}}'
",
  args:     ("a"),
  stdout:   "hello
whatever
",
  stderr:   "echo 'hello
whatever'
",
}

test! {
  name:     multiline_backtick,
  justfile: "
string := `echo hello
echo goodbye
`

a:
  echo '{{string}}'
",
  args:     ("a"),
  stdout:   "hello\ngoodbye\n",
  stderr:   "echo 'hello
goodbye'
",
}

test! {
  name:     multiline_cooked_string,
  justfile: r#"
string := "hello
whatever"

a:
  echo '{{string}}'
"#,
  args:     ("a"),
  stdout:   "hello
whatever
",
  stderr:   "echo 'hello
whatever'
",
}

test! {
  name:     invalid_escape_sequence,
  justfile: r#"x := "\q"
a:"#,
  args:     ("a"),
  stdout:   "",
  stderr:   "error: `\\q` is not a valid escape sequence
  |
1 | x := \"\\q\"
  |      ^^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     error_line_after_multiline_raw_string,
  justfile: "
string := 'hello

whatever' + 'yo'

a:
  echo '{{foo}}'
",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `foo` not defined
  |
7 |   echo '{{foo}}'
  |           ^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     error_column_after_multiline_raw_string,
  justfile: "
string := 'hello

whatever' + bar

a:
  echo '{{string}}'
",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `bar` not defined
  |
4 | whatever' + bar
  |             ^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     multiline_raw_string_in_interpolation,
  justfile: r#"
a:
  echo '{{"a" + '
  ' + "b"}}'
"#,
  args:     ("a"),
  stdout:   "
    a
      b
  ",
  stderr:   "
    echo 'a
      b'
  ",
}

test! {
  name:     error_line_after_multiline_raw_string_in_interpolation,
  justfile: r#"
a:
  echo '{{"a" + '
  ' + "b"}}'

  echo {{b}}
"#,
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `b` not defined
  |
6 |   echo {{b}}
  |          ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_raw_string,
  justfile: "
a b= ':
",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Unterminated string
  |
2 | a b= ':
  |      ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_string,
  justfile: r#"
a b= ":
"#,
  args:     ("a"),
  stdout:   "",
  stderr:   r#"error: Unterminated string
  |
2 | a b= ":
  |      ^
"#,
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_backtick,
  justfile: "
foo a=\t`echo blaaaaaah:
  echo {{a}}",
  stderr:   r#"
    error: Unterminated backtick
      |
    2 | foo a=    `echo blaaaaaah:
      |           ^
  "#,
  status:   EXIT_FAILURE,
}
