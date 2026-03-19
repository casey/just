use super::*;

#[track_caller]
fn evaluate(expression: &str, expected: &str) {
  Test::new()
    .justfile(format!("x := {expression}"))
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"])
    .stdout(expected)
    .success();
}

#[test]
fn logical_operators_are_unstable() {
  Test::new()
    .justfile("x := 'foo' && 'bar'")
    .args(["--evaluate", "x"])
    .stderr_regex(r"error: The logical operators `&&` and `\|\|` are currently unstable. .*")
    .failure();

  Test::new()
    .justfile("x := 'foo' || 'bar'")
    .args(["--evaluate", "x"])
    .stderr_regex(r"error: The logical operators `&&` and `\|\|` are currently unstable. .*")
    .failure();
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
  evaluate("'a' && 'b' || 'c'", "b");
}

#[test]
fn nesting() {
  evaluate("'' || '' || '' || '' || 'foo'", "foo");
  evaluate("'foo' && 'foo' && 'foo' && 'foo' && 'bar'", "bar");
}
