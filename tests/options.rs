use super::*;

#[test]
fn long_options_may_not_be_empty() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='')]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .stderr(
      "
        error: Option name for parameter `bar` is empty
         ——▶ justfile:1:18
          │
        1 │ [arg('bar', long='')]
          │                  ^^
      ",
    )
    .failure();
}

#[test]
fn short_options_may_not_be_empty() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='')]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .stderr(
      "
        error: Option name for parameter `bar` is empty
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', short='')]
          │                   ^^
      ",
    )
    .failure();
}

#[test]
fn short_options_may_not_have_multiple_characters() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='abc')]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .stderr(
      "
        error: Short option name for parameter `bar` contains multiple characters
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', short='abc')]
          │                   ^^^^^
      ",
    )
    .failure();
}

#[test]
fn parameters_may_be_passed_with_long_options() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .args(["foo", "--bar", "baz"])
    .stdout("bar=baz\n")
    .success();
}

#[test]
fn long_option_defaults_to_parameter_name() {
  Test::new()
    .justfile(
      "
        [arg('bar', long)]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .args(["foo", "--bar", "baz"])
    .stdout("bar=baz\n")
    .success();
}

#[test]
fn parameters_may_be_passed_with_short_options() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b')]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .args(["foo", "-b", "baz"])
    .stdout("bar=baz\n")
    .success();
}

const LONG_SHORT: &str = "
  [arg('bar', long='bar', short='b')]
  @foo bar:
    echo bar={{bar}}
";

#[test]
fn parameters_with_both_long_and_short_option_may_be_passed_as_long() {
  Test::new()
    .justfile(LONG_SHORT)
    .args(["foo", "--bar", "baz"])
    .stdout("bar=baz\n")
    .success();
}

#[test]
fn parameters_with_both_long_and_short_option_may_be_passed_as_short() {
  Test::new()
    .justfile(LONG_SHORT)
    .args(["foo", "-b", "baz"])
    .stdout("bar=baz\n")
    .success();
}

#[test]
fn parameters_with_both_long_and_short_may_not_use_both() {
  Test::new()
    .justfile(LONG_SHORT)
    .args(["foo", "--bar", "baz", "-b", "baz"])
    .stderr("error: Recipe `foo` option `-b` cannot be passed more than once\n")
    .failure();
}

#[test]
fn multiple_short_options_in_one_argument_is_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='a')]
        [arg('baz', short='b')]
        @foo bar baz:
      ",
    )
    .args(["foo", "-ab"])
    .stderr("error: Passing multiple short options (`-ab`) in one argument is not supported\n")
    .failure();
}

#[test]
fn duplicate_long_option_attributes_are_forbidden() {
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
    .failure();
}

#[test]
fn defaulted_duplicate_long_option() {
  Test::new()
    .justfile(
      "
        [arg(
          'aaa',
          long='bar'
        )]
        [arg(      'bar', long)]
        foo aaa bar:
      ",
    )
    .stderr(
      "
        error: Recipe `foo` defines option `--bar` multiple times
         ——▶ justfile:5:19
          │
        5 │ [arg(      'bar', long)]
          │                   ^^^^
      ",
    )
    .failure();
}

#[test]
fn duplicate_short_option_attributes_are_forbidden() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b')]
        [arg('baz', short='b')]
        foo bar baz:
      ",
    )
    .stderr(
      "
        error: Recipe `foo` defines option `-b` multiple times
         ——▶ justfile:2:19
          │
        2 │ [arg('baz', short='b')]
          │                   ^^^
      ",
    )
    .failure();
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
    .failure();
}

#[test]
fn variadics_with_short_options_are_forbidden() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b')]
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
    .failure();
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
        error: Option name for parameter `bar` contains equal sign
         ——▶ justfile:1:18
          │
        1 │ [arg('bar', long='bar=baz')]
          │                  ^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn short_option_names_may_not_contain_equal_sign() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='=')]
        foo bar:
      ",
    )
    .stderr(
      "
        error: Option name for parameter `bar` contains equal sign
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', short='=')]
          │                   ^^^
      ",
    )
    .failure();
}

#[test]
fn long_options_may_follow_an_omitted_positional_argument() {
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
    .success();
}

#[test]
fn short_options_may_follow_an_omitted_positional_argument() {
  Test::new()
    .justfile(
      "
        [arg('baz', short='b')]
        @foo bar='BAR' baz:
          echo bar={{bar}}
          echo baz={{baz}}
      ",
    )
    .args(["foo", "-b", "BAZ"])
    .stdout(
      "
        bar=BAR
        baz=BAZ
      ",
    )
    .success();
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
    .success();
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
    .success();
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
    .success();
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
    .failure();
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
    .success();
}

#[test]
fn positional_and_long_option_arguments_can_be_intermixed() {
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
    .success();
}

#[test]
fn positional_and_short_option_arguments_can_be_intermixed() {
  Test::new()
    .justfile(
      "
        [arg('b', short='b')]
        [arg('d', short='d')]
        @foo a b c d e:
          echo a={{a}}
          echo b={{b}}
          echo c={{c}}
          echo d={{d}}
          echo e={{e}}
      ",
    )
    .args(["foo", "A", "-d", "D", "C", "-b", "B", "E"])
    .stdout(
      "
        a=A
        b=B
        c=C
        d=D
        e=E
      ",
    )
    .success();
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
    .failure();
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
    .failure();
}

#[test]
fn duplicate_long_options_are_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo bar:
      ",
    )
    .args(["foo", "--bar=a", "--bar=b"])
    .stderr("error: Recipe `foo` option `--bar` cannot be passed more than once\n")
    .failure();
}

#[test]
fn duplicate_short_options_are_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b')]
        @foo bar:
      ",
    )
    .args(["foo", "-b=a", "-b=b"])
    .stderr("error: Recipe `foo` option `-b` cannot be passed more than once\n")
    .failure();
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
    .failure();
}

#[test]
fn recipes_with_long_options_have_correct_positional_argument_mismatch_message() {
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
    .failure();
}

#[test]
fn recipes_with_short_options_have_correct_positional_argument_mismatch_message() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b')]
        @foo bar baz:
      ",
    )
    .args(["foo", "-b=value"])
    .stderr(
      "
        error: Recipe `foo` got 0 positional arguments but takes 1
        usage:
            just foo [OPTIONS] baz
      ",
    )
    .failure();
}

#[test]
fn long_options_with_values_are_flags() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar', value='baz')]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .args(["foo", "--bar"])
    .stdout("bar=baz\n")
    .success();
}

#[test]
fn short_options_with_values_are_flags() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b', value='baz')]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .args(["foo", "-b"])
    .stdout("bar=baz\n")
    .success();
}

#[test]
fn flags_cannot_take_values() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b', value='baz')]
        @foo bar:
      ",
    )
    .args(["foo", "-b=hello"])
    .stderr("error: Recipe `foo` flag `-b` does not take value\n")
    .failure();
}

#[test]
fn value_requires_long_or_short() {
  Test::new()
    .justfile(
      "
        [arg('bar', value='baz')]
        @foo bar:
      ",
    )
    .args(["foo", "-b=hello"])
    .stderr(
      "
        error: Argument attribute `value` only valid with `long` or `short`
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', value='baz')]
          │             ^^^^^
      ",
    )
    .failure();
}

#[test]
fn options_arg_passed_as_positional_arguments() {
  Test::new()
    .justfile(
      r#"
set positional-arguments

[arg('bar', short='b')]
@foo bar:
  echo args="$@"
      "#,
    )
    .args(["foo", "-b", "baz"])
    .stdout("args=baz\n")
    .success();
}
