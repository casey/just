use super::*;

#[test]
fn duplicate_long_options_are_forbidden() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        [arg('baz', long='bar')]
        foo bar baz:
      ",
    )
    .stderr(
      "
        error: Recipe `foo` defines option `--bar` multiple times
         ——▶ justfile:2:18
          │
        2 │ [arg('baz', long='bar')]
          │                  ^^^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn variadics_with_long_options_are_forbidden() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        foo +bar:
      ",
    )
    .stderr(
      "
        error: Variadic parameters may not be options
         ——▶ justfile:2:6
          │
        2 │ foo +bar:
          │      ^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn long_option_names_may_not_contain_equal_sign() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar=baz')]
        foo bar:
      ",
    )
    .stderr(
      "
        error: Long option name for parameter `bar` contains equal sign
         ——▶ justfile:1:18
          │
        1 │ [arg('bar', long='bar=baz')]
          │                  ^^^^^^^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn options_may_follow_an_omitted_positional_argument() {
  Test::new()
    .justfile(
      "
        [arg('baz', long='baz')]
        @foo bar='BAR' baz:
          echo bar={{bar}}
          echo baz={{baz}}
      ",
    )
    .args(["foo", "--baz", "BAZ"])
    .stdout(
      "
        bar=BAR
        baz=BAZ
      ",
    )
    .run();
}

#[test]
fn options_with_a_default_may_be_omitted() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar='BAR':
          echo bar={{bar}}
      ",
    )
    .args(["foo"])
    .stdout(
      "
        bar=BAR
      ",
    )
    .run();
}

#[test]
fn variadics_can_follow_options() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar *baz:
          echo bar={{bar}}
          echo baz={{baz}}
      ",
    )
    .args(["foo", "--bar=BAR", "A", "B", "C"])
    .stdout(
      "
        bar=BAR
        baz=A B C
      ",
    )
    .run();
}

#[test]
fn argument_values_starting_with_dashes_are_accepted_if_recipe_does_not_take_options() {
  Test::new()
    .justfile(
      "
        @foo *baz:
          echo baz={{baz}}
      ",
    )
    .args(["foo", "--bar=BAR", "--A", "--B", "--C"])
    .stdout(
      "
        baz=--bar=BAR --A --B --C
      ",
    )
    .run();
}

#[test]
fn argument_values_starting_with_dashes_are_an_error_if_recipe_takes_options() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar *baz:
          echo bar={{bar}}
          echo baz={{baz}}
      ",
    )
    .args(["foo", "--bar=BAR", "--A", "--B", "--C"])
    .stderr("error: Recipe `foo` does not have option `--A`\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn argument_values_starting_with_dashes_are_accepted_after_double_dash() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar *baz:
          echo bar={{bar}}
          echo baz={{baz}}
      ",
    )
    .args(["foo", "--bar=BAR", "--", "--A", "--B", "--C"])
    .stdout(
      "
        bar=BAR
        baz=--A --B --C
      ",
    )
    .run();
}

#[test]
fn positional_and_option_arguments_can_be_intermixed() {
  Test::new()
    .justfile(
      "
        [arg('b', long='b')]
        [arg('d', long='d')]
        @foo a b c d e:
          echo a={{a}}
          echo b={{b}}
          echo c={{c}}
          echo d={{d}}
          echo e={{e}}
      ",
    )
    .args(["foo", "A", "--d", "D", "C", "--b", "B", "E"])
    .stdout(
      "
        a=A
        b=B
        c=C
        d=D
        e=E
      ",
    )
    .run();
}

#[test]
fn unknown_options_are_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar:
      ",
    )
    .args(["foo", "--baz", "BAZ"])
    .stderr("error: Recipe `foo` does not have option `--baz`\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn missing_required_options_are_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar:
      ",
    )
    .arg("foo")
    .stderr("error: Recipe `foo` requires option `--bar`\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn duplicate_options_are_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar:
      ",
    )
    .args(["foo", "--bar=a", "--bar=b"])
    .stderr("error: Recipe `foo` option `--bar` cannot be passed more than once\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn options_require_value() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar:
      ",
    )
    .args(["foo", "--bar"])
    .stderr("error: Recipe `foo` option `--bar` missing value\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn recipes_with_options_have_correct_positional_argument_mismatch_message() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar baz:
      ",
    )
    .args(["foo", "--bar=value"])
    .stderr(
      "
        error: Recipe `foo` got 0 positional arguments but takes 1
        usage:
            just foo [OPTIONS] baz
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}
