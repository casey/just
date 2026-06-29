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
        error: recipe attribute for argument `bar` first used on line 1 is duplicated on line 2
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
