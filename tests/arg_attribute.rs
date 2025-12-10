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
    .run();
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
        error: Argument `bar` passed to recipe `foo` parameter `bar` does not match pattern 'BAR'
      ",
    )
    .status(EXIT_FAILURE)
    .run();
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
        error: Argument `xbarx` passed to recipe `foo` parameter `bar` does not match pattern 'bar'
      ",
    )
    .status(EXIT_FAILURE)
    .run();
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
        error: Failed to parse argument pattern
         ——▶ justfile:1:21
          │
        1 │ [arg('bar', pattern='{')]
          │                     ^^^
        caused by: regex parse error:
            ^{$
              ^
        error: repetition quantifier expects a valid decimal
      ",
    )
    .status(EXIT_FAILURE)
    .run();
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
    .run();
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
        error: Recipe attribute for argument `bar` first used on line 1 is duplicated on line 2
         ——▶ justfile:2:2
          │
        2 │ [arg('bar', pattern='BAR')]
          │  ^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn extra_keyword_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', pattern='BAR', foo='foo')]
        foo bar:
      ",
    )
    .args(["foo", "BAR"])
    .stderr(
      "
        error: Unknown keyword `foo` for `arg` attribute
         ——▶ justfile:1:28
          │
        1 │ [arg('bar', pattern='BAR', foo='foo')]
          │                            ^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
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
        error: Argument attribute for undefined argument `bar`
         ——▶ justfile:1:6
          │
        1 │ [arg('bar', pattern='BAR')]
          │      ^^^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn split_across_multiple_lines() {
  Test::new()
    .justfile(
      "
        [arg(
          'bar',
          pattern='BAR'
        )]
        foo bar:
      ",
    )
    .args(["foo", "BAR"])
    .run();
}

#[test]
fn optional_trailing_comma() {
  Test::new()
    .justfile(
      "
        [arg(
          'bar',
          pattern='BAR',
        )]
        foo bar:
      ",
    )
    .args(["foo", "BAR"])
    .run();
}

#[test]
fn positional_arguments_cannot_follow_keyword_arguments() {
  Test::new()
    .justfile(
      "
        [arg(pattern='BAR', 'bar')]
        foo bar:
      ",
    )
    .args(["foo", "BAR"])
    .stderr(
      "
        error: Positional attribute arguments cannot follow keyword attribute arguments
         ——▶ justfile:1:21
          │
        1 │ [arg(pattern='BAR', 'bar')]
          │                     ^^^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
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
        error: Argument `bar` passed to recipe `foo` parameter `bar` does not match pattern 'BAR'
      ",
    )
    .status(EXIT_FAILURE)
    .run();
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
        error: Argument `bar` passed to recipe `foo` parameter `bar` does not match pattern 'BAR'
      ",
    )
    .status(EXIT_FAILURE)
    .run();
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
        error: Argument `bar` passed to recipe `foo` parameter `bar` does not match pattern 'BAR'
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}
