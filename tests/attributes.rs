use super::*;

#[test]
fn all() {
  Test::new()
    .justfile(
      "
      [macos]
      [windows]
      [linux]
      [unix]
      [no-exit-message]
      foo:
        exit 1
    ",
    )
    .stderr("exit 1\n")
    .status(1)
    .run();
}

#[test]
fn duplicate_attributes_are_disallowed() {
  Test::new()
    .justfile(
      "
      [no-exit-message]
      [no-exit-message]
      foo:
        echo bar
    ",
    )
    .stderr(
      "
      error: Recipe attribute `no-exit-message` first used on line 1 is duplicated on line 2
       ——▶ justfile:2:2
        │
      2 │ [no-exit-message]
        │  ^^^^^^^^^^^^^^^
      ",
    )
    .status(1)
    .run();
}

#[test]
fn multiple_attributes_one_line() {
  Test::new()
    .justfile(
      "
      [macos, windows,linux]
      [no-exit-message]
      foo:
        exit 1
    ",
    )
    .stderr("exit 1\n")
    .status(1)
    .run();
}

#[test]
fn multiple_attributes_one_line_error_message() {
  Test::new()
    .justfile(
      "
      [macos, windows linux]
      [no-exit-message]
      foo:
        exit 1
    ",
    )
    .stderr(
      "
        error: Expected ']', ',', or '(', but found identifier
         ——▶ justfile:1:17
          │
        1 │ [macos, windows linux]
          │                 ^^^^^
          ",
    )
    .status(1)
    .run();
}

#[test]
fn multiple_attributes_one_line_duplicate_check() {
  Test::new()
    .justfile(
      "
      [macos, windows, linux]
      [linux]
      foo:
        exit 1
    ",
    )
    .stderr(
      "
      error: Recipe attribute `linux` first used on line 1 is duplicated on line 2
       ——▶ justfile:2:2
        │
      2 │ [linux]
        │  ^^^^^
        ",
    )
    .status(1)
    .run();
}
