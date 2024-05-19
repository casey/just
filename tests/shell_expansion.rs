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
