use super::*;

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
  name:     cooked_string_suppress_newline,
  justfile: r#"
    a := """
      foo\
      bar
    """

    @default:
      printf %s '{{a}}'
  "#,
  stdout: "
    foobar
  ",
}

test! {
  name:     invalid_escape_sequence,
  justfile: r#"x := "\q"
a:"#,
  args:     ("a"),
  stdout:   "",
  stderr:   "error: `\\q` is not a valid escape sequence
 --> justfile:1:6
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
 --> justfile:6:11
  |
6 |   echo '{{foo}}'
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
 --> justfile:3:13
  |
3 | whatever' + bar
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
 --> justfile:5:10
  |
5 |   echo {{b}}
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
  stderr:   "
    error: Unterminated string
     --> justfile:1:6
      |
    1 | a b= ':
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
  stderr:   r#"
    error: Unterminated string
     --> justfile:1:6
      |
    1 | a b= ":
      |      ^
  "#,
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_backtick,
  justfile: "
    foo a=\t`echo blaaaaaah:
      echo {{a}}
  ",
  stderr:   r#"
    error: Unterminated backtick
     --> justfile:1:8
      |
    1 | foo a=    `echo blaaaaaah:
      |           ^
  "#,
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_indented_raw_string,
  justfile: "
    a b= ''':
  ",
  args:     ("a"),
  stdout:   "",
  stderr:   "
    error: Unterminated string
     --> justfile:1:6
      |
    1 | a b= ''':
      |      ^^^
  ",
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_indented_string,
  justfile: r#"
    a b= """:
  "#,
  args:     ("a"),
  stdout:   "",
  stderr:   r#"
    error: Unterminated string
     --> justfile:1:6
      |
    1 | a b= """:
      |      ^^^
  "#,
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_indented_backtick,
  justfile: "
    foo a=\t```echo blaaaaaah:
      echo {{a}}
  ",
  stderr:   r#"
    error: Unterminated backtick
     --> justfile:1:8
      |
    1 | foo a=    ```echo blaaaaaah:
      |           ^^^
  "#,
  status:   EXIT_FAILURE,
}

test! {
  name:     indented_raw_string_contents_indentation_removed,
  justfile: "
    a := '''
      foo
      bar
    '''

    @default:
      printf '{{a}}'
  ",
  stdout: "
    foo
    bar
  ",
}

test! {
  name:     indented_cooked_string_contents_indentation_removed,
  justfile: r#"
    a := """
      foo
      bar
    """

    @default:
      printf '{{a}}'
  "#,
  stdout: "
    foo
    bar
  ",
}

test! {
  name:     indented_backtick_string_contents_indentation_removed,
  justfile: r#"
    a := ```
      printf '
      foo
      bar
      '
    ```

    @default:
      printf '{{a}}'
  "#,
  stdout: "\n\nfoo\nbar",
}

test! {
  name:     indented_raw_string_escapes,
  justfile: r"
    a := '''
      foo\n
      bar
    '''

    @default:
      printf %s '{{a}}'
  ",
  stdout: r"
    foo\n
    bar
  ",
}

test! {
  name:     indented_cooked_string_escapes,
  justfile: r#"
    a := """
      foo\n
      bar
    """

    @default:
      printf %s '{{a}}'
  "#,
  stdout: "
    foo

    bar
  ",
}

test! {
  name:     indented_backtick_string_escapes,
  justfile: r"
    a := ```
      printf %s '
      foo\n
      bar
      '
    ```

    @default:
      printf %s '{{a}}'
  ",
  stdout: "\n\nfoo\\n\nbar",
}

test! {
  name:     shebang_backtick,
  justfile: "
    x := `#!/usr/bin/env sh`
  ",
  stderr:   "
    error: Backticks may not start with `#!`
     --> justfile:1:6
      |
    1 | x := `#!/usr/bin/env sh`
      |      ^^^^^^^^^^^^^^^^^^^
  ",
  status:   EXIT_FAILURE,
}
