use super::*;

#[test]
fn newline_escape_deps() {
  Test::new()
    .justfile(
      "
      default: a \\
               b \\
               c
      a:
        echo a
      b:
        echo b
      c:
        echo c
    ",
    )
    .stdout("a\nb\nc\n")
    .stderr("echo a\necho b\necho c\n")
    .run();
}

#[test]
fn newline_escape_deps_no_indent() {
  Test::new()
    .justfile(
      "
      default: a\\
      b\\
      c
      a:
        echo a
      b:
        echo b
      c:
        echo c
    ",
    )
    .stdout("a\nb\nc\n")
    .stderr("echo a\necho b\necho c\n")
    .run();
}

#[test]
fn newline_escape_deps_linefeed() {
  Test::new()
    .justfile(
      "
        default: a\\\r
                b
        a:
          echo a
        b:
          echo b
      ",
    )
    .stdout("a\nb\n")
    .stderr("echo a\necho b\n")
    .run();
}

#[test]
fn newline_escape_deps_invalid_esc() {
  Test::new()
    .justfile(
      "
      default: a\\ b
    ",
    )
    .stdout("")
    .stderr(
      "
        error: `\\ ` is not a valid escape sequence
         --> justfile:1:11
          |
        1 | default: a\\ b
          |           ^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn newline_escape_unpaired_linefeed() {
  Test::new()
    .justfile(
      "
      default:\\\ra",
    )
    .stdout("")
    .stderr(
      "
        error: Unpaired carriage return
         --> justfile:1:9
          |
        1 | default:\\\ra
          |         ^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}
