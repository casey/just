use super::*;

#[test]
fn all_skips_backticks() {
  Test::new()
    .justfile(
      "
        set dry-run := 'all'

        foo:
          echo {{ `echo bar` }}
      ",
    )
    .arg("--dry-run")
    .stderr("echo `echo bar`\n")
    .success();
}

#[test]
fn recipes_evaluates_backticks() {
  Test::new()
    .justfile(
      "
        set dry-run := 'recipes'

        x := `echo bar`

        foo baz=`echo baz`:
          cat foo
          echo {{ x }} {{ baz }}
      ",
    )
    .arg("--dry-run")
    .stderr("cat foo\necho bar baz\n")
    .success();
}

#[test]
fn recipes_evaluates_shell_function() {
  Test::new()
    .justfile(
      "
        set dry-run := 'recipes'

        foo:
          echo {{ shell('echo bar') }}
      ",
    )
    .arg("--dry-run")
    .stderr("echo bar\n")
    .success();
}

#[test]
fn recipes_runs_dotenv_command() {
  Test::new()
    .justfile(
      "
        set dry-run := 'recipes'
        set dotenv-command := 'exit 1'

        foo:
          echo bar
      ",
    )
    .arg("--dry-run")
    .stderr("error: dotenv command `exit 1` failed: process exited with status code 1\n")
    .status(1);
}

#[test]
fn must_be_string_literal() {
  Test::new()
    .justfile("set dry-run := ('rec' + 'ipes')")
    .stderr(
      "
        error: `dry-run` setting must be a plain string literal
         ——▶ justfile:1:5
          │
        1 │ set dry-run := ('rec' + 'ipes')
          │     ^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn invalid_value() {
  Test::new()
    .justfile("set dry-run := 'foo'")
    .stderr(
      "
        error: invalid `dry-run` setting `foo`: expected `all` or `recipes`
         ——▶ justfile:1:16
          │
        1 │ set dry-run := 'foo'
          │                ^^^^^
      ",
    )
    .failure();
}
