use super::*;

#[test]
fn basic() {
  Test::new()
    .justfile(
      "
      f(x) := x

      foo:
        echo {{f('bar')}}
      ",
    )
    .stderr("echo bar\n")
    .stdout("bar\n")
    .success();
}

#[test]
fn multiple_parameters() {
  Test::new()
    .justfile(
      "
      f(a, b) := a + b

      foo:
        echo {{f('bar', 'baz')}}
      ",
    )
    .stderr("echo barbaz\n")
    .stdout("barbaz\n")
    .success();
}

#[test]
fn zero_parameters() {
  Test::new()
    .justfile(
      "
      f() := 'bar'

      foo:
        echo {{f()}}
      ",
    )
    .stderr("echo bar\n")
    .stdout("bar\n")
    .success();
}

#[test]
fn references_outer_variable() {
  Test::new()
    .justfile(
      "
      x := 'bar'
      f(a) := x + a

      foo:
        echo {{f('baz')}}
      ",
    )
    .stderr("echo barbaz\n")
    .stdout("barbaz\n")
    .success();
}

#[test]
fn shadow_builtin() {
  Test::new()
    .justfile(
      "
      arch() := 'foo'

      bar:
        echo {{arch()}}
      ",
    )
    .stderr("echo foo\n")
    .stdout("foo\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn in_assignment() {
  Test::new()
    .justfile(
      "
      f(x) := x + x
      y := f('bar')

      foo:
        echo {{y}}
      ",
    )
    .stderr("echo barbar\n")
    .stdout("barbar\n")
    .success();
}

#[test]
fn calls_another_function() {
  Test::new()
    .justfile(
      "
      f(x) := x + x
      g(x) := f(x) + f(x)

      foo:
        echo {{g('a')}}
      ",
    )
    .stderr("echo aaaa\n")
    .stdout("aaaa\n")
    .success();
}

#[test]
fn calls_builtin_function() {
  Test::new()
    .justfile(
      "
      f(x) := uppercase(x)

      foo:
        echo {{f('bar')}}
      ",
    )
    .stderr("echo BAR\n")
    .stdout("BAR\n")
    .success();
}

#[test]
fn duplicate_function_error() {
  Test::new()
    .justfile(
      "
      f(x) := x
      f(y) := y
      ",
    )
    .stderr_regex("error: .*[Rr]edefin.*\n.*\n.*\n.*\n")
    .failure();
}

#[test]
fn unknown_function_error() {
  Test::new()
    .justfile(
      "
      x := nope()
      ",
    )
    .stderr_regex("error: Call to unknown function `nope`.*\n.*\n.*\n.*\n")
    .failure();
}

#[test]
fn argument_count_mismatch() {
  Test::new()
    .justfile(
      "
      f(a, b) := a + b
      x := f('foo')
      ",
    )
    .stderr_regex("error: Function `f` called with 1 argument but takes 2.*\n.*\n.*\n.*\n")
    .failure();
}

#[test]
fn undefined_variable_in_body() {
  Test::new()
    .justfile(
      "
      f(x) := y
      ",
    )
    .stderr_regex("error: Variable `y` not defined.*\n.*\n.*\n.*\n")
    .failure();
}

#[test]
fn forward_reference() {
  Test::new()
    .justfile(
      "
      g(x) := f(x)
      f(x) := x + x

      foo:
        echo {{g('a')}}
      ",
    )
    .stderr("echo aa\n")
    .stdout("aa\n")
    .success();
}

#[test]
fn recursive() {
  Test::new()
    .justfile(
      "
      f(x) := if x == '' { '' } else { f('') + 'a' }

      foo:
        echo {{f('x')}}
      ",
    )
    .stderr("echo a\n")
    .stdout("a\n")
    .success();
}

#[test]
fn parameter_shadows_variable() {
  Test::new()
    .justfile(
      "
      x := 'foo'
      f(x) := x

      bar:
        echo {{f('baz')}}
      ",
    )
    .stderr("echo baz\n")
    .stdout("baz\n")
    .success();
}

#[test]
fn in_recipe_default() {
  Test::new()
    .justfile(
      "
      f(x) := x + x

      foo y=f('bar'):
        echo {{y}}
      ",
    )
    .stderr("echo barbar\n")
    .stdout("barbar\n")
    .success();
}
