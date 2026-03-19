use super::*;

#[test]
fn closing_curly_brace_can_abut_interpolation_close() {
  Test::new()
    .justfile(
      "
        foo:
          echo {{if 'a' == 'b' { 'c' } else { 'd' }}}
      ",
    )
    .stderr("echo d\n")
    .stdout("d\n")
    .success();
}

#[test]
fn eol_with_continuation_in_interpolation() {
  Test::new()
    .justfile(
      "
        foo:
          echo {{(
            'a'
          )}}
      ",
    )
    .stderr("echo a\n")
    .stdout("a\n")
    .success();
}

#[test]
fn eol_without_continuation_in_interpolation() {
  Test::new()
    .justfile(
      "
        foo:
          echo {{
            'a'
          }}
      ",
    )
    .stderr("echo a\n")
    .stdout("a\n")
    .success();
}

#[test]
fn comment_in_interopolation() {
  Test::new()
    .justfile(
      "
        foo:
          echo {{ # hello
            'a'
          }}
      ",
    )
    .stderr(
      "
        error: Expected backtick, identifier, '(', '/', or string, but found comment
         ——▶ justfile:2:11
          │
        2 │   echo {{ # hello
          │           ^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn indent_and_dedent_are_ignored_in_interpolation() {
  Test::new()
    .justfile(
      "
        foo:
          echo {{
            'a'
        + 'b'
               + 'c'
          }}
          echo foo
      ",
    )
    .stderr("echo abc\necho foo\n")
    .stdout("abc\nfoo\n")
    .success();
}

#[test]
fn shebang_line_numbers_are_correct_with_multi_line_interpolations() {
  Test::new()
    .justfile(
      "
        foo:
          #!/usr/bin/env cat
          echo {{
            'a'
        + 'b'
               + 'c'
          }}
          echo foo
      ",
    )
    .stdout(if cfg!(windows) {
      "


        echo abc




        echo foo
      "
    } else {
      "
        #!/usr/bin/env cat

        echo abc




        echo foo
      "
    })
    .success();
}
