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
        error: Argument `bar` passed to recipe `foo` parameter `bar` does not match pattern 'BAR'
      ",
    )
    .failure();
}

#[test]
fn patterns_are_regulare_expressions() {
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
        error: Argument `\d+` passed to recipe `foo` parameter `bar` does not match pattern '\d+'
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
        error: Argument `xbarx` passed to recipe `foo` parameter `bar` does not match pattern 'bar'
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
        error: Failed to parse argument pattern
         ——▶ justfile:1:21
          │
        1 │ [arg('bar', pattern='{')]
          │                     ^^^
        caused by: regex parse error:
            {
            ^
        error: repetition operator missing expression
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
        error: Recipe attribute for argument `bar` first used on line 1 is duplicated on line 2
         ——▶ justfile:2:2
          │
        2 │ [arg('bar', pattern='BAR')]
          │  ^^^
      ",
    )
    .failure();
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
        error: Argument attribute for undefined argument `bar`
         ——▶ justfile:1:6
          │
        1 │ [arg('bar', pattern='BAR')]
          │      ^^^^^
      ",
    )
    .failure();
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
    .success();
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
    .success();
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
        error: Argument `bar` passed to recipe `foo` parameter `bar` does not match pattern 'BAR'
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
        error: Argument `bar` passed to recipe `foo` parameter `bar` does not match pattern 'BAR'
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
        error: Argument `bar` passed to recipe `foo` parameter `bar` does not match pattern 'BAR'
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
        error: Argument `aa` passed to recipe `foo` parameter `bar` does not match pattern 'a|b'
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
        error: Argument `BAR` passed to recipe `foo` parameter `bar` does not match pattern 'BAR BAR'
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
        error: Attribute key `pattern` requires value
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', pattern)]
          │             ^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn short_requires_value() {
  Test::new()
    .justfile(
      "
        [arg('bar', short)]
        foo bar:
      ",
    )
    .stderr(
      "
        error: Attribute key `short` requires value
         ——▶ justfile:1:13
          │
        1 │ [arg('bar', short)]
          │             ^^^^^
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
        error: Attribute key `value` requires value
         ——▶ justfile:1:19
          │
        1 │ [arg('bar', long, value)]
          │                   ^^^^^
      ",
    )
    .failure();
}
