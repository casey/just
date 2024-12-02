use super::*;

#[test]
fn raw_string() {
  Test::new()
    .justfile(
      r#"
export EXPORTED_VARIABLE := '\z'

recipe:
  printf "$EXPORTED_VARIABLE"
"#,
    )
    .stdout("\\z")
    .stderr("printf \"$EXPORTED_VARIABLE\"\n")
    .success();
}

#[test]
fn multiline_raw_string() {
  Test::new()
    .arg("a")
    .justfile(
      "
string := 'hello
whatever'

a:
  echo '{{string}}'
",
    )
    .stdout(
      "hello
whatever
",
    )
    .stderr(
      "echo 'hello
whatever'
",
    )
    .success();
}

#[test]
fn multiline_backtick() {
  Test::new()
    .arg("a")
    .justfile(
      "
string := `echo hello
echo goodbye
`

a:
  echo '{{string}}'
",
    )
    .stdout("hello\ngoodbye\n")
    .stderr(
      "echo 'hello
goodbye'
",
    )
    .success();
}

#[test]
fn multiline_cooked_string() {
  Test::new()
    .arg("a")
    .justfile(
      r#"
string := "hello
whatever"

a:
  echo '{{string}}'
"#,
    )
    .stdout(
      "hello
whatever
",
    )
    .stderr(
      "echo 'hello
whatever'
",
    )
    .success();
}

#[test]
fn cooked_string_suppress_newline() {
  Test::new()
    .justfile(
      r#"
    a := """
      foo\
      bar
    """

    @default:
      printf %s '{{a}}'
  "#,
    )
    .stdout(
      "
    foobar
  ",
    )
    .success();
}

#[test]
fn invalid_escape_sequence() {
  Test::new()
    .arg("a")
    .justfile(
      r#"x := "\q"
a:"#,
    )
    .stderr(
      "error: `\\q` is not a valid escape sequence
 ——▶ justfile:1:6
  │
1 │ x := \"\\q\"
  │      ^^^^
",
    )
    .failure();
}

#[test]
fn error_line_after_multiline_raw_string() {
  Test::new()
    .arg("a")
    .justfile(
      "
string := 'hello

whatever' + 'yo'

a:
  echo '{{foo}}'
",
    )
    .stderr(
      "error: Variable `foo` not defined
 ——▶ justfile:6:11
  │
6 │   echo '{{foo}}'
  │           ^^^
",
    )
    .failure();
}

#[test]
fn error_column_after_multiline_raw_string() {
  Test::new()
    .arg("a")
    .justfile(
      "
string := 'hello

whatever' + bar

a:
  echo '{{string}}'
",
    )
    .stderr(
      "error: Variable `bar` not defined
 ——▶ justfile:3:13
  │
3 │ whatever' + bar
  │             ^^^
",
    )
    .failure();
}

#[test]
fn multiline_raw_string_in_interpolation() {
  Test::new()
    .arg("a")
    .justfile(
      r#"
a:
  echo '{{"a" + '
  ' + "b"}}'
"#,
    )
    .stdout(
      "
    a
      b
  ",
    )
    .stderr(
      "
    echo 'a
      b'
  ",
    )
    .success();
}

#[test]
fn error_line_after_multiline_raw_string_in_interpolation() {
  Test::new()
    .arg("a")
    .justfile(
      r#"
a:
  echo '{{"a" + '
  ' + "b"}}'

  echo {{b}}
"#,
    )
    .stderr(
      "error: Variable `b` not defined
 ——▶ justfile:5:10
  │
5 │   echo {{b}}
  │          ^
",
    )
    .failure();
}

#[test]
fn unterminated_raw_string() {
  Test::new()
    .arg("a")
    .justfile(
      "
    a b= ':
  ",
    )
    .stderr(
      "
    error: Unterminated string
     ——▶ justfile:1:6
      │
    1 │ a b= ':
      │      ^
  ",
    )
    .failure();
}

#[test]
fn unterminated_string() {
  Test::new()
    .arg("a")
    .justfile(
      r#"
    a b= ":
  "#,
    )
    .stderr(
      r#"
    error: Unterminated string
     ——▶ justfile:1:6
      │
    1 │ a b= ":
      │      ^
  "#,
    )
    .failure();
}

#[test]
fn unterminated_backtick() {
  Test::new()
    .justfile(
      "
    foo a=\t`echo blaaaaaah:
      echo {{a}}
  ",
    )
    .stderr(
      r"
    error: Unterminated backtick
     ——▶ justfile:1:8
      │
    1 │ foo a=    `echo blaaaaaah:
      │           ^
  ",
    )
    .failure();
}

#[test]
fn unterminated_indented_raw_string() {
  Test::new()
    .arg("a")
    .justfile(
      "
    a b= ''':
  ",
    )
    .stderr(
      "
    error: Unterminated string
     ——▶ justfile:1:6
      │
    1 │ a b= ''':
      │      ^^^
  ",
    )
    .failure();
}

#[test]
fn unterminated_indented_string() {
  Test::new()
    .arg("a")
    .justfile(
      r#"
    a b= """:
  "#,
    )
    .stderr(
      r#"
    error: Unterminated string
     ——▶ justfile:1:6
      │
    1 │ a b= """:
      │      ^^^
  "#,
    )
    .failure();
}

#[test]
fn unterminated_indented_backtick() {
  Test::new()
    .justfile(
      "
    foo a=\t```echo blaaaaaah:
      echo {{a}}
  ",
    )
    .stderr(
      r"
    error: Unterminated backtick
     ——▶ justfile:1:8
      │
    1 │ foo a=    ```echo blaaaaaah:
      │           ^^^
  ",
    )
    .failure();
}

#[test]
fn indented_raw_string_contents_indentation_removed() {
  Test::new()
    .justfile(
      "
    a := '''
      foo
      bar
    '''

    @default:
      printf '{{a}}'
  ",
    )
    .stdout(
      "
    foo
    bar
  ",
    )
    .success();
}

#[test]
fn indented_cooked_string_contents_indentation_removed() {
  Test::new()
    .justfile(
      r#"
    a := """
      foo
      bar
    """

    @default:
      printf '{{a}}'
  "#,
    )
    .stdout(
      "
    foo
    bar
  ",
    )
    .success();
}

#[test]
fn indented_backtick_string_contents_indentation_removed() {
  Test::new()
    .justfile(
      r"
    a := ```
      printf '
      foo
      bar
      '
    ```

    @default:
      printf '{{a}}'
  ",
    )
    .stdout("\n\nfoo\nbar")
    .success();
}

#[test]
fn indented_raw_string_escapes() {
  Test::new()
    .justfile(
      r"
    a := '''
      foo\n
      bar
    '''

    @default:
      printf %s '{{a}}'
  ",
    )
    .stdout(
      r"
    foo\n
    bar
  ",
    )
    .success();
}

#[test]
fn indented_cooked_string_escapes() {
  Test::new()
    .justfile(
      r#"
    a := """
      foo\n
      bar
    """

    @default:
      printf %s '{{a}}'
  "#,
    )
    .stdout(
      "
    foo

    bar
  ",
    )
    .success();
}

#[test]
fn indented_backtick_string_escapes() {
  Test::new()
    .justfile(
      r"
    a := ```
      printf %s '
      foo\n
      bar
      '
    ```

    @default:
      printf %s '{{a}}'
  ",
    )
    .stdout("\n\nfoo\\n\nbar")
    .success();
}

#[test]
fn shebang_backtick() {
  Test::new()
    .justfile(
      "
    x := `#!/usr/bin/env sh`
  ",
    )
    .stderr(
      "
    error: Backticks may not start with `#!`
     ——▶ justfile:1:6
      │
    1 │ x := `#!/usr/bin/env sh`
      │      ^^^^^^^^^^^^^^^^^^^
  ",
    )
    .failure();
}

#[test]
fn valid_unicode_escape() {
  Test::new()
    .justfile(r#"x := "\u{1f916}\u{1F916}""#)
    .args(["--evaluate", "x"])
    .stdout("🤖🤖")
    .success();
}

#[test]
fn unicode_escapes_with_all_hex_digits() {
  Test::new()
    .justfile(r#"x := "\u{012345}\u{6789a}\u{bcdef}\u{ABCDE}\u{F}""#)
    .args(["--evaluate", "x"])
    .stdout("\u{012345}\u{6789a}\u{bcdef}\u{ABCDE}\u{F}")
    .success();
}

#[test]
fn maximum_valid_unicode_escape() {
  Test::new()
    .justfile(r#"x := "\u{10FFFF}""#)
    .args(["--evaluate", "x"])
    .stdout("\u{10FFFF}")
    .success();
}

#[test]
fn unicode_escape_no_braces() {
  Test::new()
    .justfile("x := \"\\u1234\"")
    .args(["--evaluate", "x"])
    .stderr(
      r#"
error: expected unicode escape sequence delimiter `{` but found `1`
 ——▶ justfile:1:6
  │
1 │ x := "\u1234"
  │      ^^^^^^^^
"#,
    )
    .failure();
}

#[test]
fn unicode_escape_empty() {
  Test::new()
    .justfile("x := \"\\u{}\"")
    .args(["--evaluate", "x"])
    .stderr(
      r#"
error: unicode escape sequences must not be empty
 ——▶ justfile:1:6
  │
1 │ x := "\u{}"
  │      ^^^^^^
"#,
    )
    .failure();
}

#[test]
fn unicode_escape_requires_immediate_opening_brace() {
  Test::new()
    .justfile("x := \"\\u {1f916}\"")
    .args(["--evaluate", "x"])
    .stderr(
      r#"
error: expected unicode escape sequence delimiter `{` but found ` `
 ——▶ justfile:1:6
  │
1 │ x := "\u {1f916}"
  │      ^^^^^^^^^^^^
"#,
    )
    .failure();
}

#[test]
fn unicode_escape_non_hex() {
  Test::new()
    .justfile("x := \"\\u{foo}\"")
    .args(["--evaluate", "x"])
    .stderr(
      r#"
Error: expected hex digit [0-9A-Fa-f] but found `o`
   ╭─[justfile:1:6]
   │
 1 │ x := "\u{foo}"
───╯
"#,
    )
    .failure();
}

#[test]
fn unicode_escape_invalid_character() {
  Test::new()
    .justfile("x := \"\\u{BadBad}\"")
    .args(["--evaluate", "x"])
    .stderr(
      r#"
error: unicode escape sequence value `BadBad` greater than maximum valid code point `10FFFF`
 ——▶ justfile:1:6
  │
1 │ x := "\u{BadBad}"
  │      ^^^^^^^^^^^^
"#,
    )
    .failure();
}

#[test]
fn unicode_escape_too_long() {
  Test::new()
    .justfile("x := \"\\u{FFFFFFFFFF}\"")
    .args(["--evaluate", "x"])
    .stderr(
      r#"
error: unicode escape sequence starting with `\u{FFFFFFF` longer than six hex digits
 ——▶ justfile:1:6
  │
1 │ x := "\u{FFFFFFFFFF}"
  │      ^^^^^^^^^^^^^^^^
"#,
    )
    .failure();
}

#[test]
fn unicode_escape_unterminated() {
  Test::new()
    .justfile("x := \"\\u{1f917\"")
    .args(["--evaluate", "x"])
    .stderr(
      r#"
error: unterminated unicode escape sequence
 ——▶ justfile:1:6
  │
1 │ x := "\u{1f917"
  │      ^^^^^^^^^^
"#,
    )
    .failure();
}
