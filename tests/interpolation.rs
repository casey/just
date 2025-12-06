use super::*;

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
    .run();
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
    .run();
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
    .status(EXIT_FAILURE)
    .run();
}
