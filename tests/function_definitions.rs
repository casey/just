use super::*;

#[test]
fn unstable() {
  Test::new()
    .justfile("foo() := 'bar'")
    .stderr_regex(r"error: user-defined functions are currently unstable,.*")
    .failure();
}

#[test]
fn redefinition() {
  Test::new()
    .justfile(
      "
        foo() := 'bar'
        foo() := 'baz'
      ",
    )
    .stderr(
      "
        error: function `foo` first defined on line 1 is redefined on line 2
         ——▶ justfile:2:1
          │
        2 │ foo() := 'baz'
          │ ^^^
      ",
    )
    .unstable()
    .failure();
}

#[test]
fn wrong_argument_count() {
  Test::new()
    .justfile(
      "
        foo(x) := x
        a := foo('bar', 'baz')
      ",
    )
    .stderr(
      "
        error: function `foo` called with 2 arguments but takes 1
         ——▶ justfile:2:6
          │
        2 │ a := foo('bar', 'baz')
          │      ^^^
      ",
    )
    .unstable()
    .failure();
}

#[test]
fn undefined_variable_in_body() {
  Test::new()
    .justfile(
      "
        foo() := bar
        a := foo()
      ",
    )
    .stderr(
      "
        error: variable `bar` not defined
         ——▶ justfile:1:10
          │
        1 │ foo() := bar
          │          ^^^
      ",
    )
    .unstable()
    .failure();
}

#[test]
fn undefined_in_assignment() {
  Test::new()
    .justfile("a := foo()")
    .stderr(
      "
        error: call to undefined function `foo`
         ——▶ justfile:1:6
          │
        1 │ a := foo()
          │      ^^^
      ",
    )
    .unstable()
    .failure();
}

#[test]
fn undefined_in_setting() {
  Test::new()
    .justfile("set tempdir := foo()")
    .stderr(
      "
        error: call to undefined function `foo`
         ——▶ justfile:1:16
          │
        1 │ set tempdir := foo()
          │                ^^^
      ",
    )
    .unstable()
    .failure();
}

#[test]
fn undefined_in_recipe_parameter_default() {
  Test::new()
    .justfile("bar x=foo():")
    .stderr(
      "
        error: call to undefined function `foo`
         ——▶ justfile:1:7
          │
        1 │ bar x=foo():
          │       ^^^
      ",
    )
    .unstable()
    .failure();
}

#[test]
fn undefined_in_dependency_argument() {
  Test::new()
    .justfile(
      "
        bar x:
        foo: (bar baz())
      ",
    )
    .stderr(
      "
        error: call to undefined function `baz`
         ——▶ justfile:2:11
          │
        2 │ foo: (bar baz())
          │           ^^^
      ",
    )
    .unstable()
    .failure();
}

#[test]
fn undefined_in_confirm_attribute() {
  Test::new()
    .justfile(
      "
        [confirm(foo())]
        bar:
      ",
    )
    .stderr(
      "
        error: call to undefined function `foo`
         ——▶ justfile:1:10
          │
        1 │ [confirm(foo())]
          │          ^^^
      ",
    )
    .unstable()
    .failure();
}

#[test]
fn undefined_in_interpolation() {
  Test::new()
    .justfile(
      "
        bar:
          echo {{foo()}}
      ",
    )
    .stderr(
      "
        error: call to undefined function `foo`
         ——▶ justfile:2:10
          │
        2 │   echo {{foo()}}
          │          ^^^
      ",
    )
    .unstable()
    .failure();
}

#[test]
fn uses_parameter() {
  Test::new()
    .justfile(
      "
        foo(x) := x
        a := foo('bar')
      ",
    )
    .args(["--evaluate", "a"])
    .stdout("bar")
    .unstable()
    .success();
}

#[test]
fn uses_outer_variable() {
  Test::new()
    .justfile(
      "
        x := 'bar'
        foo() := x
        a := foo()
      ",
    )
    .args(["--evaluate", "a"])
    .stdout("bar")
    .unstable()
    .success();
}

#[test]
fn parameter_shadows_variable() {
  Test::new()
    .justfile(
      "
        x := 'bar'
        foo(x) := x
        a := foo('baz')
      ",
    )
    .args(["--evaluate", "a"])
    .stdout("baz")
    .unstable()
    .success();
}

#[test]
fn format_no_args() {
  Test::new()
    .justfile(
      "
        foo() := 'bar'

        a := foo()
      ",
    )
    .arg("--dump")
    .stdout(
      "
        foo() := 'bar'

        a := foo()
      ",
    )
    .unstable()
    .success();
}

#[test]
fn format_one_arg() {
  Test::new()
    .justfile(
      "
        foo(x) := x

        a := foo('bar')
      ",
    )
    .arg("--dump")
    .stdout(
      "
        foo(x) := x

        a := foo('bar')
      ",
    )
    .unstable()
    .success();
}

#[test]
fn format_two_args() {
  Test::new()
    .justfile(
      "
        foo(x, y) := x + y

        a := foo('bar', 'baz')
      ",
    )
    .arg("--dump")
    .stdout(
      "
        foo(x, y) := x + y

        a := foo('bar', 'baz')
      ",
    )
    .unstable()
    .success();
}

#[test]
fn trailing_comma() {
  Test::new()
    .justfile(
      "
        foo(x,) := x
        a := foo('bar')
      ",
    )
    .args(["--evaluate", "a"])
    .stdout("bar")
    .unstable()
    .success();
}

#[test]
fn has_access_to_env_file() {
  Test::new()
    .justfile(
      "
        set dotenv-required

        foo() := env('VAR')

        a := foo()
      ",
    )
    .write(".env", "VAR=VAL")
    .args(["--evaluate", "a"])
    .stdout("VAL")
    .unstable()
    .success();
}

#[test]
fn may_reference_overrides() {
  Test::new()
    .justfile(
      "
        x := 'bar'
        foo() := x

        a:
          @echo {{foo()}}
      ",
    )
    .args(["x=baz", "a"])
    .stdout("baz\n")
    .unstable()
    .success();
}

#[test]
fn inherits_is_dependency() {
  Test::new()
    .justfile(
      "
        foo() := is_dependency()

        bar: baz
          @echo 'bar {{foo()}}'

        baz:
          @echo 'baz {{foo()}}'
      ",
    )
    .arg("bar")
    .stdout("baz true\nbar false\n")
    .unstable()
    .success();
}

#[test]
fn inherits_recipe_name() {
  Test::new()
    .justfile(
      "
        foo() := recipe_name()

        bar:
          @echo {{foo()}}
      ",
    )
    .stdout("bar\n")
    .unstable()
    .success();
}

#[test]
fn may_reference_non_const_assignment() {
  Test::new()
    .justfile(
      "
        foo() := bar

        bar := `echo baz`

        a := foo()
      ",
    )
    .args(["--evaluate", "a"])
    .stdout("baz")
    .unstable()
    .success();
}

#[test]
fn shadow_builtin() {
  #[track_caller]
  fn case(name: &str) {
    Test::new()
      .justfile(format!("{name}(x) := x\na := {name}('bar')"))
      .args(["--evaluate", "a"])
      .stdout("bar")
      .unstable()
      .success();
  }

  case("bool");
  case("show");
  case("which");
}
