use super::*;

// todo:
// - what is truthy and what is falsy?
// - set empty-false := true and require setting to use `&&` or `||`
// - land as unstable?
// - deprecate functions?
//
// `is_dependency`
// `path_exists`
// `semver_matches`
// test that this works:
// foo := path_exists(foo_path) && foo_path || '/fallback'

#[track_caller]
fn evaluate(expression: &str, expected: &str) {
  Test::new()
    .justfile(format!("x := {expression}"))
    .args(["--evaluate", "x"])
    .stdout(expected)
    .run();
}

#[test]
fn and_returns_empty_string_if_lhs_is_empty() {
  evaluate("'' && 'hello'", "");
}

#[test]
fn and_returns_rhs_if_lhs_is_non_empty() {
  evaluate("'hello' && 'goodbye'", "goodbye");
}

#[test]
fn and_has_lower_precedence_than_plus() {
  evaluate("'' && 'goodbye' + 'foo'", "");

  evaluate("'foo' + 'hello' && 'goodbye'", "goodbye");

  evaluate("'foo' + '' && 'goodbye'", "goodbye");

  evaluate("'foo' + 'hello' && 'goodbye' + 'bar'", "goodbyebar");
}

#[test]
fn or_returns_rhs_if_lhs_is_empty() {
  evaluate("'' || 'hello'", "hello");
}

#[test]
fn or_returns_lhs_if_lhs_is_non_empty() {
  evaluate("'hello' || 'goodbye'", "hello");
}

#[test]
fn or_has_lower_precedence_than_plus() {
  evaluate("'' || 'goodbye' + 'foo'", "goodbyefoo");

  evaluate("'foo' + 'hello' || 'goodbye'", "foohello");

  evaluate("'foo' + '' || 'goodbye'", "foo");

  evaluate("'foo' + 'hello' || 'goodbye' + 'bar'", "foohello");
}

#[test]
fn and_has_higher_precedence_than_or() {
  evaluate("('' && 'foo') || 'bar'", "bar");
  evaluate("'' && 'foo' || 'bar'", "bar");
}

#[test]
fn misc() {
  evaluate("'' || '' || '' || '' || 'foo'", "foo");
  evaluate("'foo' && 'foo' && 'foo' && 'foo' && 'bar'", "bar");
}
