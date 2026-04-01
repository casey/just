use super::*;

#[test]
fn all_settings_allow_expressions() {
  Test::new()
    .justfile(
      "
        foo := 'hello'

        set dotenv-filename := foo
        set dotenv-path := foo
        set script-interpreter := [foo, foo, foo]
        set shell := [foo, foo, foo]
        set tempdir := foo
        set windows-shell := [foo, foo, foo]
        set working-directory := foo
      ",
    )
    .arg("--summary")
    .stdout(
      "

      ",
    )
    .stderr("Justfile contains no recipes.\n")
    .success();
}

#[test]
fn undefined_variable_in_working_directory() {
  Test::new()
    .justfile(
      "
        set working-directory := foo
      ",
    )
    .stderr(
      "
      error: Variable `foo` not defined
       ——▶ justfile:1:26
        │
      1 │ set working-directory := foo
        │                          ^^^
    ",
    )
    .failure();
}

#[test]
fn undefined_variable_in_dotenv_filename() {
  Test::new()
    .justfile(
      "
        set dotenv-filename := foo
      ",
    )
    .stderr(
      "
      error: Variable `foo` not defined
       ——▶ justfile:1:24
        │
      1 │ set dotenv-filename := foo
        │                        ^^^
    ",
    )
    .failure();
}

#[test]
fn undefined_variable_in_dotenv_path() {
  Test::new()
    .justfile(
      "
        set dotenv-path := foo
      ",
    )
    .stderr(
      "
      error: Variable `foo` not defined
       ——▶ justfile:1:20
        │
      1 │ set dotenv-path := foo
        │                    ^^^
    ",
    )
    .failure();
}

#[test]
fn undefined_variable_in_tempdir() {
  Test::new()
    .justfile(
      "
        set tempdir := foo
      ",
    )
    .stderr(
      "
      error: Variable `foo` not defined
       ——▶ justfile:1:16
        │
      1 │ set tempdir := foo
        │                ^^^
    ",
    )
    .failure();
}

#[test]
fn undefined_variable_in_script_interpreter_command() {
  Test::new()
    .justfile(
      "
        set script-interpreter := [foo]
      ",
    )
    .stderr(
      "
      error: Variable `foo` not defined
       ——▶ justfile:1:28
        │
      1 │ set script-interpreter := [foo]
        │                            ^^^
    ",
    )
    .failure();
}

#[test]
fn undefined_variable_in_script_interpreter_argument() {
  Test::new()
    .justfile(
      "
        set script-interpreter := ['foo', bar]
      ",
    )
    .stderr(
      "
      error: Variable `bar` not defined
       ——▶ justfile:1:35
        │
      1 │ set script-interpreter := ['foo', bar]
        │                                   ^^^
    ",
    )
    .failure();
}

#[test]
fn undefined_variable_in_shell_command() {
  Test::new()
    .justfile(
      "
        set shell := [foo]
      ",
    )
    .stderr(
      "
      error: Variable `foo` not defined
       ——▶ justfile:1:15
        │
      1 │ set shell := [foo]
        │               ^^^
    ",
    )
    .failure();
}

#[test]
fn undefined_variable_in_shell_argument() {
  Test::new()
    .justfile(
      "
        set shell := ['foo', bar]
      ",
    )
    .stderr(
      "
      error: Variable `bar` not defined
       ——▶ justfile:1:22
        │
      1 │ set shell := ['foo', bar]
        │                      ^^^
    ",
    )
    .failure();
}

#[test]
fn undefined_variable_in_windows_shell_command() {
  Test::new()
    .justfile(
      "
        set windows-shell := [foo]
      ",
    )
    .stderr(
      "
      error: Variable `foo` not defined
       ——▶ justfile:1:23
        │
      1 │ set windows-shell := [foo]
        │                       ^^^
    ",
    )
    .failure();
}

#[test]
fn undefined_variable_in_windows_shell_argument() {
  Test::new()
    .justfile(
      "
        set windows-shell := ['foo', bar]
      ",
    )
    .stderr(
      "
      error: Variable `bar` not defined
       ——▶ justfile:1:30
        │
      1 │ set windows-shell := ['foo', bar]
        │                              ^^^
    ",
    )
    .failure();
}

#[test]
fn built_in_constant() {
  Test::new()
    .justfile(
      "
        set working-directory := HEX

        @foo:
          cat file.txt
      ",
    )
    .write("0123456789abcdef/file.txt", "bar")
    .stdout("bar")
    .success();
}

#[test]
fn variable() {
  Test::new()
    .justfile(
      "
        dir := 'bar'

        set working-directory := dir

        @foo:
          cat file.txt
      ",
    )
    .write("bar/file.txt", "baz")
    .arg("foo")
    .stdout("baz")
    .success();
}

#[test]
fn unused_non_const_assignments() {
  Test::new()
    .justfile(
      "
        baz := `pwd`

        dir := 'bar'

        set working-directory := dir

        @foo:
          cat file.txt
      ",
    )
    .write("bar/file.txt", "baz")
    .arg("foo")
    .stdout("baz")
    .success();
}

#[test]
fn variable_with_override() {
  Test::new()
    .justfile(
      "
        dir := 'bar'

        set working-directory := dir

        @foo:
          cat file.txt
      ",
    )
    .arg("dir=bob")
    .write("bob/file.txt", "baz")
    .arg("foo")
    .stdout("baz")
    .success();
}

#[test]
fn expression() {
  Test::new()
    .justfile(
      "
        dir := 'bar'

        set working-directory := dir + '-bob'

        @foo:
          cat file.txt
      ",
    )
    .write("bar-bob/file.txt", "baz")
    .arg("foo")
    .stdout("baz")
    .success();
}

#[test]
fn expression_with_override() {
  Test::new()
    .justfile(
      "
        dir := 'bar'

        set working-directory := dir + '-bob'

        @foo:
          cat file.txt
      ",
    )
    .write("bob-bob/file.txt", "baz")
    .args(["dir=bob", "foo"])
    .stdout("baz")
    .success();
}

#[test]
fn backtick() {
  Test::new()
    .justfile(
      "
        set working-directory := `pwd`
      ",
    )
    .stderr(
      "
      error: Cannot call backticks in const context
       ——▶ justfile:1:26
        │
      1 │ set working-directory := `pwd`
        │                          ^^^^^
    ",
    )
    .failure();
}

#[test]
fn function_call() {
  Test::new()
    .justfile(
      "
        set working-directory := arch()
      ",
    )
    .stderr(
      "
      error: Cannot call functions in const context
       ——▶ justfile:1:26
        │
      1 │ set working-directory := arch()
        │                          ^^^^
    ",
    )
    .failure();
}

#[test]
fn non_const_variable() {
  Test::new()
    .justfile(
      "
        foo := `pwd`

        set working-directory := foo
      ",
    )
    .stderr(
      "
      error: Cannot access non-const variable `foo` in const context
       ——▶ justfile:3:26
        │
      3 │ set working-directory := foo
        │                          ^^^
    ",
    )
    .failure();
}

#[test]
fn assert() {
  Test::new()
    .justfile(
      "
        set working-directory := assert('foo' == 'bar', 'fail')
      ",
    )
    .stderr(
      "
        error: Assert failed: fail
         ——▶ justfile:1:26
          │
        1 │ set working-directory := assert('foo' == 'bar', 'fail')
          │                          ^^^^^^
      ",
    )
    .failure();
}

#[test]
fn bad_regex() {
  Test::new()
    .justfile(
      "
        set working-directory := if '' =~ '(' {
          'a'
        } else {
          'b'
        }
      ",
    )
    .stderr(
      "
        error: regex parse error:
            (
            ^
        error: unclosed group
      ",
    )
    .failure();
}

#[test]
fn backtick_override() {
  Test::new()
    .justfile(
      "
        bar := `pwd`

        set working-directory := bar

        @foo:
          cat file.txt
      ",
    )
    .test_round_trip(false)
    .arg("bar=foo")
    .write("foo/file.txt", "baz")
    .arg("foo")
    .stdout("baz")
    .success();
}

#[test]
fn submodule_expression() {
  Test::new()
    .write(
      "foo/mod.just",
      "
dir := 'bar'

set working-directory := dir + '-baz'

foo:
  @cat file.txt
",
    )
    .justfile(
      "
        dir := 'hello'

        mod foo
      ",
    )
    .write("foo/bar-baz/file.txt", "ok")
    .args(["foo", "foo"])
    .stdout("ok")
    .success();
}

#[test]
fn overrides_are_ignored_in_submodules() {
  Test::new()
    .write(
      "foo.just",
      "
dir := 'bar'

set working-directory := dir

foo:
  @cat file.txt
",
    )
    .justfile(
      "
        mod foo

        dir := 'root'

        bob := 'baz'
      ",
    )
    .args(["dir=bob", "bob=foo", "foo::foo"])
    .write("bar/file.txt", "ok")
    .stdout("ok")
    .success();
}
