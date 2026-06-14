use super::*;

#[test]
fn parameter_default_values_may_use_earlier_parameters() {
  Test::new()
    .justfile(
      "
        @foo a b=a:
          echo {{ b }}
      ",
    )
    .args(["foo", "bar"])
    .stdout("bar\n")
    .success();
}

#[test]
fn parameter_default_values_may_not_use_later_parameters() {
  Test::new()
    .justfile(
      "
        @foo a b=c c='':
          echo {{ b }}
      ",
    )
    .args(["foo", "bar"])
    .stderr(
      "
        error: variable `c` not defined
         ——▶ justfile:1:10
          │
        1 │ @foo a b=c c='':
          │          ^
      ",
    )
    .failure();
}

#[test]
fn star_may_follow_default() {
  Test::new()
    .justfile(
      "
        foo bar='baz' *bob:
          @echo {{bar}} {{bob}}
      ",
    )
    .args(["foo", "hello", "goodbye"])
    .stdout("hello goodbye\n")
    .success();
}

#[test]
fn variadic_arguments_are_joined_when_passed_to_functions() {
  Test::new()
    .justfile(
      "
        foo *args:
          @printf '%s\\n' {{ quote(args) }}
      ",
    )
    .args(["foo", "bar", "baz bob"])
    .stdout("bar baz bob\n")
    .success();
}

#[test]
fn empty_star_parameter_is_equal_to_empty_string() {
  Test::new()
    .justfile(
      "
        foo *args:
          @echo {{ if args == '' { 'empty' } else { 'nonempty' } }}
      ",
    )
    .args(["foo"])
    .stdout("empty\n")
    .success();
}

#[test]
fn empty_star_parameter_is_falsy() {
  Test::new()
    .justfile(
      "
        set lists

        foo *args:
          @echo {{ args || 'fallback' }}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo"])
    .stdout("fallback\n")
    .success();
}

#[test]
fn exported_variadic_parameter_is_joined_with_spaces() {
  Test::new()
    .justfile(
      "
        foo +$args:
          @echo $args
      ",
    )
    .args(["foo", "bar", "baz"])
    .stdout("bar baz\n")
    .success();
}

#[test]
fn variadic_parameter_passed_to_dependency_is_joined_with_spaces() {
  Test::new()
    .justfile(
      "
        foo *args: (bar args)

        bar first *rest:
          @echo first={{ quote(first) }} rest={{ quote(rest) }}
      ",
    )
    .args(["foo", "bar", "baz"])
    .stdout("first=bar baz rest=\n")
    .success();
}
