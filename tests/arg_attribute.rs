use super::*;

#[test]
fn pattern_match() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='BAR')]
        foo bar:
      ",
    )
    .args(["foo", "BAR"])
    .success();
}

#[test]
fn pattern_mismatch() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='BAR')]
        foo bar:
      ",
    )
    .args(["foo", "bar"])
    .stderr(
      "
        error: argument `bar` passed to recipe `foo` parameter `bar` does not match pattern `BAR`
      ",
    )
    .failure();
}

#[test]
fn patterns_are_regular_expressions() {
  Test::new()
    .justfile(
      r"
        [arg('bar', pattern='\d+')]
        foo bar:
      ",
    )
    .args(["foo", r"\d+"])
    .stderr(
      r"
        error: argument `\d+` passed to recipe `foo` parameter `bar` does not match pattern `\d+`
      ",
    )
    .failure();
}

#[test]
fn pattern_must_match_entire_string() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='bar')]
        foo bar:
      ",
    )
    .args(["foo", "xbarx"])
    .stderr(
      "
        error: argument `xbarx` passed to recipe `foo` parameter `bar` does not match pattern `bar`
      ",
    )
    .failure();
}

#[test]
fn pattern_invalid_regex_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='{')]
        foo bar:
      ",
    )
    .stderr(
      "
        error: failed to parse argument pattern
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', pattern='{')]
          │             ^^^^^^^
        caused by: regex parse error:
            {
            ^
        error: repetition operator missing expression
      ",
    )
    .failure();
}

#[test]
fn pattern_may_be_expression() {
  Test::new()
    .justfile(
      "
        prefix := 'B'
        [arg('bar', pattern=prefix + 'AR')]
        foo bar:
      ",
    )
    .args(["foo", "bar"])
    .stderr(
      "
        error: argument `bar` passed to recipe `foo` parameter `bar` does not match pattern `BAR`
      ",
    )
    .failure();
}

#[test]
fn pattern_cannot_reference_parameter() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern=bar)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: variable `bar` not defined
         ——▶ justfile:1:21
          │
        1 │ [arg('bar', pattern=bar)]
          │                     ^^^
      ",
    )
    .failure();
}

#[test]
fn pattern_cannot_reference_undefined_variable() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern=undefined)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: variable `undefined` not defined
         ——▶ justfile:1:21
          │
        1 │ [arg('bar', pattern=undefined)]
          │                     ^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn pattern_list_match() {
  Test::new()
    .justfile(
      "
          set lists
          [arg('bar', pattern=['A', 'B'])]
          foo bar:
        ",
    )
    .unstable()
    .args(["foo", "A"])
    .success()
    .test()
    .unstable()
    .args(["foo", "B"])
    .success();
}

#[test]
fn pattern_list_mismatch() {
  Test::new()
    .justfile(
      "
        set lists
        [arg('bar', pattern=['A', 'B'])]
        foo bar:
      ",
    )
    .unstable()
    .args(["foo", "C"])
    .stderr(
      "
        error: argument `C` passed to recipe `foo` parameter `bar` does not match pattern `A` or `B`
      ",
    )
    .failure();
}

#[test]
fn pattern_empty_list_accepts_all_arguments() {
  Test::new()
    .justfile(
      "
        set lists
        [arg('bar', pattern=[])]
        foo bar:
      ",
    )
    .unstable()
    .args(["foo", "anything"])
    .success();
}

#[test]
fn pattern_cannot_reference_non_const_variable() {
  Test::new()
    .justfile(
      "
        bar := `echo BAR`
        [arg('bar', pattern=bar)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: cannot access non-const variable `bar` in const context
         ——▶ justfile:2:21
          │
        2 │ [arg('bar', pattern=bar)]
          │                     ^^^
      ",
    )
    .failure();
}

#[test]
fn dump() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='BAR')]
        foo bar:
      ",
    )
    .arg("--dump")
    .stdout(
      "
        [arg('bar', pattern='BAR')]
        foo bar:
      ",
    )
    .success();
}

#[test]
fn duplicate_attribute_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='BAR')]
        [arg('bar', pattern='BAR')]
        foo bar:
      ",
    )
    .args(["foo", "BAR"])
    .stderr(
      "
        error: attribute for argument `bar` first used on line 1 is duplicated on line 2
         ——▶ justfile:2:2
          │
        2 │ [arg('bar', pattern='BAR')]
          │  ^^^
      ",
    )
    .failure();
}

#[test]
fn unknown_argument_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='BAR')]
        foo:
      ",
    )
    .arg("foo")
    .stderr(
      "
        error: argument attribute for undefined argument `bar`
         ——▶ justfile:1:6
          │
        1 │ [arg('bar', pattern='BAR')]
          │      ^^^^^
      ",
    )
    .failure();
}

#[test]
fn pattern_mismatches_are_caught_before_running_dependencies() {
  Test::new()
    .justfile(
      "
        baz:
          exit 1

        [arg('bar', pattern='BAR')]
        foo bar: baz
      ",
    )
    .args(["foo", "bar"])
    .stderr(
      "
        error: argument `bar` passed to recipe `foo` parameter `bar` does not match pattern `BAR`
      ",
    )
    .failure();
}

#[test]
fn pattern_mismatches_are_caught_before_running_invocation() {
  Test::new()
    .justfile(
      "
        baz:
          exit 1

        [arg('bar', pattern='BAR')]
        foo bar: baz
      ",
    )
    .args(["baz", "foo", "bar"])
    .stderr(
      "
        error: argument `bar` passed to recipe `foo` parameter `bar` does not match pattern `BAR`
      ",
    )
    .failure();
}

#[test]
fn pattern_mismatches_are_caught_in_evaluated_arguments() {
  Test::new()
    .justfile(
      "
        bar: (foo 'ba' + 'r')

        [arg('bar', pattern='BAR')]
        foo bar:
      ",
    )
    .stderr(
      "
        error: argument `bar` passed to recipe `foo` parameter `bar` does not match pattern `BAR`
      ",
    )
    .failure();
}

#[test]
fn alternates_do_not_bind_to_anchors() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='a|b')]
        foo bar:
      ",
    )
    .args(["foo", "aa"])
    .stderr(
      "
        error: argument `aa` passed to recipe `foo` parameter `bar` does not match pattern `a|b`
      ",
    )
    .failure();
}

#[test]
fn pattern_match_variadic() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='BAR')]
        foo *bar:
      ",
    )
    .args(["foo", "BAR", "BAR"])
    .success();
}

#[test]
fn pattern_mismatch_variadic() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='BAR BAR')]
        foo *bar:
      ",
    )
    .args(["foo", "BAR", "BAR"])
    .stderr(
      "
        error: argument `BAR` passed to recipe `foo` parameter `bar` does not match pattern `BAR BAR`
      ",
    )
    .failure();
}

#[test]
fn pattern_mismatch_repeatable_option() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='bar', pattern='BAR')]
        foo +bar:
      ",
    )
    .args(["foo", "--bar", "BAR", "--bar", "BAZ"])
    .stderr(
      "
        error: argument `BAZ` passed to recipe `foo` parameter `bar` does not match pattern `BAR`
      ",
    )
    .failure();
}

#[test]
fn pattern_requires_value() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: attribute key `pattern` requires value
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', pattern)]
          │             ^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn value_requires_value() {
  Test::new()
    .justfile(
      "
        [arg('bar', long, value)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: attribute key `value` requires value
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', long, value)]
          │                   ^^^^^
      ",
    )
    .failure();
}

#[test]
fn help_cannot_reference_parameter() {
  Test::new()
    .justfile(
      "
        [arg('bar', help=bar)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: variable `bar` not defined
         ——▶ justfile:1:18
          │
        1 │ [arg('bar', help=bar)]
          │                  ^^^
      ",
    )
    .failure();
}

#[test]
fn help_cannot_reference_undefined_variable() {
  Test::new()
    .justfile(
      "
        [arg('bar', help=undefined)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: variable `undefined` not defined
         ——▶ justfile:1:18
          │
        1 │ [arg('bar', help=undefined)]
          │                  ^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn help_cannot_reference_non_const_variable() {
  Test::new()
    .justfile(
      "
        bar := `echo BAR`
        [arg('bar', help=bar)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: cannot access non-const variable `bar` in const context
         ——▶ justfile:2:18
          │
        2 │ [arg('bar', help=bar)]
          │                  ^^^
      ",
    )
    .failure();
}

#[test]
fn help_may_be_expression() {
  Test::new()
    .justfile(
      "
        prefix := 'hello '
        [arg('bar', help=prefix + 'world')]
        foo bar:
      ",
    )
    .args(["--usage", "foo"])
    .stdout(
      "
        Usage: just foo bar

        Arguments:
          bar hello world
      ",
    )
    .success();
}

#[test]
fn help_list_is_joined() {
  Test::new()
    .justfile(
      "
        set lists
        [arg('bar', help=['hello', 'world'])]
        foo bar:
      ",
    )
    .unstable()
    .args(["--usage", "foo"])
    .stdout(
      "
        Usage: just foo bar

        Arguments:
          bar hello world
      ",
    )
    .success();
}

#[test]
fn help_empty_list_is_no_help() {
  Test::new()
    .justfile(
      "
        set lists
        [arg('bar', help=[])]
        foo bar:
      ",
    )
    .unstable()
    .args(["--usage", "foo"])
    .stdout(
      "
        Usage: just foo bar

        Arguments:
          bar
      ",
    )
    .success();
}

#[test]
fn variadic_arguments_up_to_max_are_accepted() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', max='2')]
        @foo +bar:
          echo bar='{{ show(bar) }}'
      ",
    )
    .unstable()
    .args(["foo", "a", "b"])
    .stdout(
      r#"
        bar=["a", "b"]
      "#,
    )
    .success();
}

#[test]
fn multiple_arguments_up_to_max_are_accepted() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, multiple, max='2')]
        @foo +bar:
          echo bar='{{ show(bar) }}'
      ",
    )
    .unstable()
    .args(["foo", "--bar=a", "--bar=b"])
    .stdout(
      r#"
        bar=["a", "b"]
      "#,
    )
    .success();
}

#[test]
fn variadic_arguments_exceeding_max_are_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', max='2')]
        foo +bar:
      ",
    )
    .unstable()
    .args(["foo", "a", "b", "c"])
    .stderr("error: recipe `foo` parameter `bar` got 3 values but takes at most 2\n")
    .failure();
}

#[test]
fn max_zero_rejects_all_elements() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', max='0')]
        foo *bar:
      ",
    )
    .unstable()
    .args(["foo", "a"])
    .stderr("error: recipe `foo` parameter `bar` got 1 value but takes at most 0\n")
    .failure();
}

#[test]
fn max_requires_multiple_or_variadic() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', max='2')]
        foo bar:
      ",
    )
    .unstable()
    .stderr(
      "
        error: argument attribute `max` only valid with `multiple` or a variadic parameter
         ——▶ justfile:3:13
          │
        3 │ [arg('bar', max='2')]
          │             ^^^
      ",
    )
    .failure();
}

#[test]
fn max_requires_value() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', max)]
        foo +bar:
      ",
    )
    .unstable()
    .stderr(
      "
        error: attribute key `max` requires value
         ——▶ justfile:3:13
          │
        3 │ [arg('bar', max)]
          │             ^^^
      ",
    )
    .failure();
}

#[test]
fn max_value_must_be_string_literal() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', max=('2'))]
        foo +bar:
      ",
    )
    .unstable()
    .stderr(
      "
        error: attribute `arg` arguments must be string literals
         ——▶ justfile:3:13
          │
        3 │ [arg('bar', max=('2'))]
          │             ^^^
      ",
    )
    .failure();
}

#[test]
fn invalid_max_value_error() {
  #[track_caller]
  fn case(value: &str) {
    Test::new()
      .justfile(format!("[arg('bar', max='{value}')]\nfoo +bar:"))
      .stderr(format!(
        "
          error: invalid `max` value `{value}`
           ——▶ justfile:1:17
            │
          1 │ [arg('bar', max='{value}')]
            │                 {caret}
        ",
        caret = "^".repeat(value.len() + 2),
      ))
      .failure();
  }

  case("x");
  case("+1");
  case("01");
  case("1 ");
}

#[test]
fn max_overflow_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', max='18446744073709551616')]
        foo +bar:
      ",
    )
    .stderr(
      "
        error: invalid `max` value `18446744073709551616`: number too large to fit in target type
         ——▶ justfile:1:17
          │
        1 │ [arg('bar', max='18446744073709551616')]
          │                 ^^^^^^^^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn dependency_list_argument_exceeding_max_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', max='2')]
        foo +bar:

        baz: (foo ['a', 'b', 'c'])
      ",
    )
    .unstable()
    .args(["baz"])
    .stderr("error: recipe `foo` parameter `bar` got 3 values but takes at most 2\n")
    .failure();
}

#[test]
fn dump_max() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', max='2')]
        foo +bar:
      ",
    )
    .unstable()
    .arg("--dump")
    .stdout(
      "
        set lists

        [arg('bar', max='2')]
        foo +bar:
      ",
    )
    .success();
}

#[test]
fn variadic_arguments_meeting_min_are_accepted() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min='2')]
        @foo +bar:
          echo bar='{{ show(bar) }}'
      ",
    )
    .unstable()
    .args(["foo", "a", "b"])
    .stdout(
      r#"
        bar=["a", "b"]
      "#,
    )
    .success();
}

#[test]
fn multiple_arguments_meeting_min_are_accepted() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long, multiple, min='2')]
        @foo +bar:
          echo bar='{{ show(bar) }}'
      ",
    )
    .unstable()
    .args(["foo", "--bar=a", "--bar=b"])
    .stdout(
      r#"
        bar=["a", "b"]
      "#,
    )
    .success();
}

#[test]
fn variadic_arguments_below_min_are_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min='2')]
        foo +bar:
      ",
    )
    .unstable()
    .args(["foo", "a"])
    .stderr("error: recipe `foo` parameter `bar` got 1 value but takes at least 2\n")
    .failure();
}

#[test]
fn star_variadic_without_arguments_below_min_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min='2')]
        foo *bar:
      ",
    )
    .unstable()
    .arg("foo")
    .stderr("error: recipe `foo` parameter `bar` got 0 values but takes at least 2\n")
    .failure();
}

#[test]
fn default_satisfying_min_is_accepted() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min='2')]
        @foo *bar=['a', 'b']:
          echo bar='{{ show(bar) }}'
      ",
    )
    .unstable()
    .arg("foo")
    .stdout(
      r#"
        bar=["a", "b"]
      "#,
    )
    .success();
}

#[test]
fn default_violating_min_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min='2')]
        foo *bar=['a']:
      ",
    )
    .unstable()
    .arg("foo")
    .stderr("error: recipe `foo` parameter `bar` got 1 value but takes at least 2\n")
    .failure();
}

#[test]
fn default_violating_max_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', max='2')]
        foo *bar=['a', 'b', 'c']:
      ",
    )
    .unstable()
    .arg("foo")
    .stderr("error: recipe `foo` parameter `bar` got 3 values but takes at most 2\n")
    .failure();
}

#[test]
fn min_requires_set_lists() {
  Test::new()
    .justfile(
      "
        [arg('bar', min='2')]
        foo +bar:
      ",
    )
    .stderr(
      "
        error: `[arg(min)]` requires `set lists`
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', min='2')]
          │             ^^^
      ",
    )
    .failure();
}

#[test]
fn min_requires_multiple_or_variadic() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min='2')]
        foo bar:
      ",
    )
    .unstable()
    .stderr(
      "
        error: argument attribute `min` only valid with `multiple` or a variadic parameter
         ——▶ justfile:3:13
          │
        3 │ [arg('bar', min='2')]
          │             ^^^
      ",
    )
    .failure();
}

#[test]
fn min_requires_value() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min)]
        foo +bar:
      ",
    )
    .unstable()
    .stderr(
      "
        error: attribute key `min` requires value
         ——▶ justfile:3:13
          │
        3 │ [arg('bar', min)]
          │             ^^^
      ",
    )
    .failure();
}

#[test]
fn min_value_must_be_string_literal() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min=('2'))]
        foo +bar:
      ",
    )
    .unstable()
    .stderr(
      "
        error: attribute `arg` arguments must be string literals
         ——▶ justfile:3:13
          │
        3 │ [arg('bar', min=('2'))]
          │             ^^^
      ",
    )
    .failure();
}

#[test]
fn invalid_min_value_error() {
  #[track_caller]
  fn case(value: &str) {
    Test::new()
      .justfile(format!("[arg('bar', min='{value}')]\nfoo +bar:"))
      .stderr(format!(
        "
          error: invalid `min` value `{value}`
           ——▶ justfile:1:17
            │
          1 │ [arg('bar', min='{value}')]
            │                 {caret}
        ",
        caret = "^".repeat(value.len() + 2),
      ))
      .failure();
  }

  case("x");
  case("+1");
  case("01");
  case("1 ");
}

#[test]
fn min_overflow_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', min='18446744073709551616')]
        foo +bar:
      ",
    )
    .stderr(
      "
        error: invalid `min` value `18446744073709551616`: number too large to fit in target type
         ——▶ justfile:1:17
          │
        1 │ [arg('bar', min='18446744073709551616')]
          │                 ^^^^^^^^^^^^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn min_exceeding_max_is_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', min='3', max='2')]
        foo +bar:
      ",
    )
    .stderr(
      "
        error: argument attribute `min` `3` exceeds `max` `2`
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', min='3', max='2')]
          │             ^^^
      ",
    )
    .failure();
}

#[test]
fn dependency_list_argument_below_min_is_an_error() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min='2')]
        foo +bar:

        baz: (foo ['a'])
      ",
    )
    .unstable()
    .args(["baz"])
    .stderr("error: recipe `foo` parameter `bar` got 1 value but takes at least 2\n")
    .failure();
}

#[test]
fn dump_min() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', min='1', max='2')]
        foo +bar:
      ",
    )
    .unstable()
    .arg("--dump")
    .stdout(
      "
        set lists

        [arg('bar', min='1', max='2')]
        foo +bar:
      ",
    )
    .success();
}
