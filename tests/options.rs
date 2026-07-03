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
        error: option name for parameter `bar` is empty
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
        error: option name for parameter `bar` is empty
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
        error: short option name for parameter `bar` contains multiple characters
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

#[test]
fn short_option_defaults_to_first_character_of_parameter_name() {
  Test::new()
    .justfile(
      "
        [arg('bar', short)]
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
    .stderr("error: recipe `foo` option `-b` cannot be passed more than once\n")
    .failure();
}

#[test]
fn multiple_short_options_may_be_combined() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='a', value='1')]
        [arg('baz', short='b', value='2')]
        [arg('qux', short='c', value='3')]
        @foo bar baz qux:
          echo {{bar}} {{baz}} {{qux}}
      ",
    )
    .args(["foo", "-abc"])
    .stdout("1 2 3\n")
    .success();
}

#[test]
fn combined_short_options_may_end_with_a_value_option() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='a', value='1')]
        [arg('baz', short='b', value='2')]
        [arg('qux', short='c')]
        @foo bar baz qux:
          echo {{bar}} {{baz}} {{qux}}
      ",
    )
    .args(["foo", "-abc", "D"])
    .stdout("1 2 D\n")
    .success();
}

#[test]
fn combined_short_value_option_must_be_last() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='a')]
        [arg('baz', short='b', value='2')]
        @foo bar baz:
      ",
    )
    .args(["foo", "-ab"])
    .stderr(
      "error: recipe `foo` option `-a` takes a value and so must be last when combined with other options\n",
    )
    .failure();
}

#[test]
fn combined_short_options_may_not_repeat() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='a', value='1')]
        [arg('baz', short='b', value='2')]
        @foo bar baz:
      ",
    )
    .args(["foo", "-aab"])
    .stderr("error: recipe `foo` option `-a` cannot be passed more than once\n")
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
        error: recipe `foo` defines option `--bar` multiple times
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
        [arg('aaa', long='bar')]
        [arg('bar', long)]
        foo aaa bar:
      ",
    )
    .stderr(
      "
        error: recipe `foo` defines option `--bar` multiple times
         ——▶ justfile:1:18
          │
        1 │ [arg('aaa', long='bar')]
          │                  ^^^^^
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
        error: recipe `foo` defines option `-b` multiple times
         ——▶ justfile:2:19
          │
        2 │ [arg('baz', short='b')]
          │                   ^^^
      ",
    )
    .failure();
}

#[test]
fn defaulted_duplicate_short_option() {
  Test::new()
    .justfile(
      "
        [arg('bar', short='b')]
        [arg('baz', short)]
        foo bar baz:
      ",
    )
    .stderr(
      "
        error: recipe `foo` defines option `-b` multiple times
         ——▶ justfile:2:13
          │
        2 │ [arg('baz', short)]
          │             ^^^^^
      ",
    )
    .failure();
}

#[test]
fn defaulted_short_option_with_empty_argument_name() {
  Test::new()
    .justfile(
      "
        [arg('', short)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: argument attribute for undefined argument ``
         ——▶ justfile:1:6
          │
        1 │ [arg('', short)]
          │      ^^
      ",
    )
    .failure();
}

#[test]
fn plus_variadic_long_option_is_repeatable() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo +bar:
          echo bar={{bar}}
      ",
    )
    .args(["foo", "--bar", "a", "--bar", "b"])
    .stdout("bar=a b\n")
    .success();
}

#[test]
fn star_variadic_option_may_be_omitted() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo *bar:
          echo bar={{bar}}
      ",
    )
    .arg("foo")
    .stdout("bar=\n")
    .success();
}

#[test]
fn plus_variadic_option_requires_one_argument() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar')]
        @foo +bar:
          echo bar={{bar}}
      ",
    )
    .arg("foo")
    .stderr("error: recipe `foo` requires option `--bar`\n")
    .failure();
}

#[test]
fn variadic_option_is_list() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long='bar')]
        @foo +bar:
          echo bar='{{ show(bar) }}'
      ",
    )
    .unstable()
    .args(["foo", "--bar", "a", "--bar", "b"])
    .stdout("bar=[\"a\", \"b\"]\n")
    .success();
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
        error: option name for parameter `bar` contains equal sign
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
        error: option name for parameter `bar` contains equal sign
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
    .stderr("error: recipe `foo` does not have option `--A`\n")
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
    .stderr("error: recipe `foo` does not have option `--baz`\n")
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
    .stderr("error: recipe `foo` requires option `--bar`\n")
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
    .stderr("error: recipe `foo` option `--bar` cannot be passed more than once\n")
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
    .stderr("error: recipe `foo` option `-b` cannot be passed more than once\n")
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
    .stderr("error: recipe `foo` option `--bar` missing value\n")
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
        error: recipe `foo` got 0 positional arguments but takes 1
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
        error: recipe `foo` got 0 positional arguments but takes 1
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
    .stderr("error: recipe `foo` flag `-b` does not take value\n")
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
        error: argument attribute `value` only valid with `long` or `short`
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', value='baz')]
          │             ^^^^^
      ",
    )
    .failure();
}

#[test]
fn value_may_be_an_expression() {
  Test::new()
    .justfile(
      "
        BAZ := 'baz'

        [arg('bar', long='bar', value=bob + BAZ)]
        @foo bob bar:
          echo {{ bar }}
      ",
    )
    .args(["foo", "hello", "--bar"])
    .stdout("hellobaz\n")
    .success();
}

#[test]
fn value_expression_evaluation_may_fail() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar', value=env('FOO'))]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .args(["foo", "--bar"])
    .stderr(
      "
        error: call to function `env` failed: environment variable `FOO` not present
         ——▶ justfile:1:31
          │
        1 │ [arg('bar', long='bar', value=env('FOO'))]
          │                               ^^^
      ",
    )
    .failure();
}

#[test]
fn value_with_undefined_variable_is_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar', value=BAZ)]
        @foo bar:
      ",
    )
    .stderr(
      "
        error: variable `BAZ` not defined
         ——▶ justfile:1:31
          │
        1 │ [arg('bar', long='bar', value=BAZ)]
          │                               ^^^
      ",
    )
    .failure();
}

#[test]
fn value_expression_is_pattern_checked() {
  Test::new()
    .justfile(
      "
        BAZ := 'baz'

        [arg('bar', long='bar', value=BAZ, pattern='[0-9]+')]
        @foo bar:
      ",
    )
    .args(["foo", "--bar"])
    .stderr(
      "error: argument `baz` passed to recipe `foo` parameter `bar` does not match pattern `[0-9]+`\n",
    )
    .failure();
}

#[test]
fn value_omitted_uses_default() {
  Test::new()
    .justfile(
      "
        BAZ := 'baz'

        [arg('bar', long='bar', value=BAZ)]
        @foo bar='qux':
          echo bar={{bar}}
      ",
    )
    .args(["foo"])
    .stdout("bar=qux\n")
    .success();
}

#[test]
fn value_uses_forwarded_dependency_argument() {
  Test::new()
    .justfile(
      "
        BAZ := 'baz'

        [arg('bar', long='bar', value=BAZ)]
        @foo bar='qux':
          echo bar={{bar}}

        caller: (foo 'forwarded')
      ",
    )
    .args(["caller"])
    .stdout("bar=forwarded\n")
    .success();
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

#[test]
fn flag_passed_is_true() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, flag)]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .unstable()
    .args(["foo", "--bar"])
    .stdout("bar=true\n")
    .success();
}

#[test]
fn flag_omitted_is_empty() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, flag)]
        @foo bar:
          echo bar={{show(bar)}}
      ",
    )
    .unstable()
    .args(["foo"])
    .stdout("bar=[]\n")
    .success();
}

#[test]
fn flag_requires_set_lists() {
  Test::new()
    .justfile(
      "
        [arg('bar', long, flag)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: `flag` arguments require `set lists`
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', long, flag)]
          │                   ^^^^
      ",
    )
    .failure();
}

#[test]
fn flag_conflicts_with_value() {
  Test::new()
    .justfile(
      "
        [arg('bar', long, flag, value='baz')]
        foo bar:
      ",
    )
    .stderr(
      "
        error: argument `bar` may not have both `flag` and `value` attributes
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', long, flag, value='baz')]
          │                   ^^^^
      ",
    )
    .failure();
}

#[test]
fn flag_requires_long_or_short() {
  Test::new()
    .justfile(
      "
        [arg('bar', flag)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: argument attribute `flag` only valid with `long` or `short`
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', flag)]
          │             ^^^^
      ",
    )
    .failure();
}

#[test]
fn flag_takes_no_value() {
  Test::new()
    .justfile(
      "
        [arg('bar', long, flag='baz')]
        foo bar:
      ",
    )
    .stderr(
      "
        error: `flag` attribute for argument `bar` takes no value
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', long, flag='baz')]
          │                   ^^^^
      ",
    )
    .failure();
}

#[test]
fn flag_with_default_is_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', long, flag)]
        foo bar='baz':
      ",
    )
    .stderr(
      "
        error: flag parameter `bar` may not have a default
         ——▶ justfile:2:5
          │
        2 │ foo bar='baz':
          │     ^^^
      ",
    )
    .failure();
}

#[test]
fn flags_passed_with_a_value_are_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, flag)]
        @foo bar:
          echo bar={{bar}}
      ",
    )
    .unstable()
    .args(["foo", "--bar=baz"])
    .stderr("error: recipe `foo` flag `--bar` does not take value\n")
    .failure();
}

#[test]
fn multiple_option_is_list() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, multiple)]
        @foo bar:
          echo bar='{{ show(bar) }}'
      ",
    )
    .unstable()
    .args(["foo", "--bar", "a", "--bar", "b"])
    .stdout(
      r#"
        bar=["a", "b"]
      "#,
    )
    .success();
}

#[test]
fn multiple_flag_counts_occurrences() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('verbose', short, flag, multiple)]
        @foo verbose:
          echo verbose='{{ show(verbose) }}'
      ",
    )
    .unstable()
    .args(["foo", "-vvv"])
    .stdout(
      r#"
        verbose=["true", "true", "true"]
      "#,
    )
    .success();
}

#[test]
fn multiple_value_option_repeats_value() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, value='baz', multiple)]
        @foo bar:
          echo bar='{{ show(bar) }}'
      ",
    )
    .unstable()
    .args(["foo", "--bar", "--bar"])
    .stdout(
      r#"
        bar=["baz", "baz"]
      "#,
    )
    .success();
}

#[test]
fn multiple_value_option_concatenates_list_values() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, value=['a', 'b'], multiple)]
        @foo bar:
          echo bar='{{ show(bar) }}'
      ",
    )
    .unstable()
    .args(["foo", "--bar", "--bar"])
    .stdout(
      r#"
        bar=["a", "b", "a", "b"]
      "#,
    )
    .success();
}

#[test]
fn multiple_requires_set_lists() {
  Test::new()
    .justfile(
      "
        [arg('bar', long, multiple)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: `[arg(multiple)]` requires `set lists`
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', long, multiple)]
          │                   ^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn multiple_requires_long_or_short() {
  Test::new()
    .justfile(
      "
        [arg('bar', multiple)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: argument attribute `multiple` only valid with `long` or `short`
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', multiple)]
          │             ^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn multiple_takes_no_value() {
  Test::new()
    .justfile(
      "
        [arg('bar', long, multiple='baz')]
        foo bar:
      ",
    )
    .stderr(
      "
        error: attribute key `multiple` takes no value
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', long, multiple='baz')]
          │                   ^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn multiple_option_exceeding_max_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, multiple, max='2')]
        foo bar:
      ",
    )
    .unstable()
    .args(["foo", "--bar", "a", "--bar", "b", "--bar", "c"])
    .stderr("error: recipe `foo` parameter `bar` got 3 values but takes at most 2\n")
    .failure();
}

#[test]
fn multiple_option_below_min_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, multiple, min='2')]
        foo bar:
      ",
    )
    .unstable()
    .args(["foo", "--bar", "a"])
    .stderr("error: recipe `foo` parameter `bar` got 1 value but takes at least 2\n")
    .failure();
}
