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
    .run();
}

#[test]
fn undefined_variable() {
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
    .status(EXIT_FAILURE)
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .run();
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
    .status(EXIT_FAILURE)
    .run();
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
    .status(EXIT_FAILURE)
    .run();
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
    .status(EXIT_FAILURE)
    .run();
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
    .status(EXIT_FAILURE)
    .run();
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
    .status(EXIT_FAILURE)
    .run();
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
    .run();
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
    .run();
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
    .run();
}
