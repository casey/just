use super::*;

#[test]
fn strings_are_shell_expanded() {
  Test::new()
    .justfile(
      "
        x := x'$JUST_TEST_VARIABLE'
      ",
    )
    .env("JUST_TEST_VARIABLE", "FOO")
    .args(["--evaluate", "x"])
    .stdout("FOO")
    .run();
}

#[test]
fn shell_expanded_error_messages_highlight_string_token() {
  Test::new()
    .justfile(
      "
        x := x'$FOOOOOOOOOOOOOOOOOOOOOOOOOOOOO'
      ",
    )
    .env("JUST_TEST_VARIABLE", "FOO")
    .args(["--evaluate", "x"])
    .status(1)
    .stderr(
    "
      error: Shell expansion failed: error looking key 'FOOOOOOOOOOOOOOOOOOOOOOOOOOOOO' up: environment variable not found
       ——▶ justfile:1:7
        │
      1 │ x := x'$FOOOOOOOOOOOOOOOOOOOOOOOOOOOOO'
        │       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
      ")
    .run();
}

#[test]
fn shell_expanded_strings_are_dumped_correctly() {
  Test::new()
    .justfile(
      "
        x := x'$JUST_TEST_VARIABLE'
      ",
    )
    .env("JUST_TEST_VARIABLE", "FOO")
    .args(["--dump", "--unstable"])
    .stdout("x := x'$JUST_TEST_VARIABLE'\n")
    .run();
}

#[test]
fn shell_expanded_strings_can_be_used_in_settings() {
  Test::new()
    .justfile(
      "
        set dotenv-filename := x'$JUST_TEST_VARIABLE'

        @foo:
          echo $DOTENV_KEY
      ",
    )
    .env("JUST_TEST_VARIABLE", ".env")
    .stdout("dotenv-value\n")
    .run();
}
