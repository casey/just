use super::*;

#[test]
fn ignore_leading_byte_order_mark() {
  Test::new()
    .justfile(
      "
      \u{feff}foo:
        echo bar
    ",
    )
    .stderr("echo bar\n")
    .stdout("bar\n")
    .run();
}

#[test]
fn non_leading_byte_order_mark_produces_error() {
  Test::new()
    .justfile(
      "
      foo:
        echo bar
      \u{feff}
    ",
    )
    .stderr(
      "
      error: Expected \'@\', '!', \'[\', comment, end of file, end of line, or identifier, but found byte order mark
       --> justfile:3:1
        |
      3 | \u{feff}
        | ^
      ")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn dont_mention_byte_order_mark_in_errors() {
  Test::new()
    .justfile("{")
    .stderr(
      "
      error: Expected '@', '!', '[', comment, end of file, end of line, or identifier, but found '{'
       --> justfile:1:1
        |
      1 | {
        | ^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}
