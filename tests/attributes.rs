use super::*;

#[test]
fn all() {
  Test::new()
    .justfile(
      "
      [macos]
      [linux]
      [openbsd]
      [unix]
      [windows]
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
      [macos,windows,linux,openbsd]
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
      [macos,windows linux,openbsd]
      [no-exit-message]
      foo:
        exit 1
    ",
    )
    .stderr(
      "
        error: Expected ']', ':', ',', or '(', but found identifier
         ——▶ justfile:1:16
          │
        1 │ [macos,windows linux,openbsd]
          │                ^^^^^
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
      [macos, windows, linux, openbsd]
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

#[test]
fn unexpected_attribute_argument() {
  Test::new()
    .justfile(
      "
      [private('foo')]
      foo:
        exit 1
    ",
    )
    .stderr(
      "
        error: Attribute `private` got 1 argument but takes 0 arguments
         ——▶ justfile:1:2
          │
        1 │ [private('foo')]
          │  ^^^^^^^
          ",
    )
    .status(1)
    .run();
}

#[test]
fn multiple_metadata_attributes() {
  Test::new()
    .justfile(
      "
      [metadata('example')]
      [metadata('sample')]
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
fn multiple_metadata_attributes_with_multiple_args() {
  Test::new()
    .justfile(
      "
      [metadata('example', 'arg1')]
      [metadata('sample', 'argument')]
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
fn expected_metadata_attribute_argument() {
  Test::new()
    .justfile(
      "
      [metadata]
      foo:
        exit 1
    ",
    )
    .stderr(
      "
        error: Attribute `metadata` got 0 arguments but takes at least 1 argument
         ——▶ justfile:1:2
          │
        1 │ [metadata]
          │  ^^^^^^^^
          ",
    )
    .status(1)
    .run();
}

#[test]
fn doc_attribute() {
  Test::new()
    .justfile(
      "
    # Non-document comment
    [doc('The real docstring')]
    foo:
      echo foo
  ",
    )
    .args(["--list"])
    .stdout(
      "
    Available recipes:
        foo # The real docstring
        ",
    )
    .run();
}

#[test]
fn doc_attribute_suppress() {
  Test::new()
    .justfile(
      "
        # Non-document comment
        [doc]
        foo:
          echo foo
      ",
    )
    .args(["--list"])
    .stdout(
      "
    Available recipes:
        foo
        ",
    )
    .run();
}

#[test]
fn doc_multiline() {
  Test::new()
    .justfile(
      "
        [doc('multiline
        comment')]
        foo:
      ",
    )
    .args(["--list"])
    .stdout(
      "
    Available recipes:
        # multiline
        # comment
        foo
        ",
    )
    .run();
}

#[test]
fn extension() {
  Test::new()
    .justfile(
      "
        [extension: '.txt']
        baz:
          #!/bin/sh
          echo $0
      ",
    )
    .stdout_regex(r"*baz\.txt\n")
    .run();
}

#[test]
fn extension_on_linewise_error() {
  Test::new()
    .justfile(
      "
        [extension: '.txt']
        baz:
      ",
    )
    .stderr(
      "
  error: Recipe `baz` has invalid attribute `extension`
   ——▶ justfile:2:1
    │
  2 │ baz:
    │ ^^^
",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn duplicate_non_repeatable_attributes_are_forbidden() {
  Test::new()
    .justfile(
      "
        [confirm: 'yes']
        [confirm: 'no']
        baz:
      ",
    )
    .stderr(
      "
  error: Recipe attribute `confirm` first used on line 1 is duplicated on line 2
   ——▶ justfile:2:2
    │
  2 │ [confirm: 'no']
    │  ^^^^^^^
",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn shell_expanded_strings_can_be_used_in_attributes() {
  Test::new()
    .justfile(
      "
        [doc(x'foo')]
        bar:
      ",
    )
    .run();
}

#[test]
fn env_attribute_single() {
  Test::new()
    .justfile(
      "
        [env('MY_VAR', 'my_value')]
        foo:
          echo $MY_VAR
      ",
    )
    .stdout("my_value\n")
    .stderr("echo $MY_VAR\n")
    .run();
}

#[test]
fn env_attribute_multiple() {
  Test::new()
    .justfile(
      "
        [env('VAR1', 'value1')]
        [env('VAR2', 'value 2')]
        foo:
          echo $VAR1 $VAR2
      ",
    )
    .stdout("value1 value 2\n")
    .stderr("echo $VAR1 $VAR2\n")
    .run();
}

#[test]
fn env_attribute_1_arg() {
  Test::new()
    .justfile(
      "
        [env('MY_VAR')]
        foo:
          echo bar
      ",
    )
    .stderr(
      "
  error: Attribute `env` got 1 argument but takes 2 arguments
   ——▶ justfile:1:2
    │
  1 │ [env('MY_VAR')]
    │  ^^^
",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn env_attribute_3_args() {
  Test::new()
    .justfile(
      "
        [env('A', 'B', 'C')]
        foo:
          echo bar
      ",
    )
    .stderr(
      "
  error: Attribute `env` got 3 arguments but takes 2 arguments
   ——▶ justfile:1:2
    │
  1 │ [env('A', 'B', 'C')]
    │  ^^^
",
    )
    .status(EXIT_FAILURE)
    .run();
}
