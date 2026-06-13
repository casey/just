use super::*;

#[track_caller]
fn evaluate(expression: &str, expected: &str) {
  Test::new()
    .justfile(format!("set lists\nx := {expression}"))
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"])
    .stdout(expected)
    .success();
}

#[test]
fn and_is_unstable() {
  Test::new()
    .justfile(
      "
        x := 'foo' && 'bar'

        foo:
          @echo hi
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: logical operators require `set lists`
         ——▶ justfile:1:12
          │
        1 │ x := 'foo' && 'bar'
          │            ^^
      ",
    )
    .failure();
}

#[test]
fn or_is_unstable() {
  Test::new()
    .justfile(
      "
        x := 'foo' || 'bar'

        foo:
          @echo hi
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: logical operators require `set lists`
         ——▶ justfile:1:12
          │
        1 │ x := 'foo' || 'bar'
          │            ^^
      ",
    )
    .failure();
}

#[test]
fn and_returns_empty_list_if_lhs_is_falsy() {
  evaluate("[] && 'hello'", "");
}

#[test]
fn and_returns_rhs_if_lhs_is_truthy() {
  evaluate("'hello' && 'goodbye'", "goodbye");
}

#[test]
fn or_returns_rhs_if_lhs_is_falsy() {
  evaluate("[] || 'hello'", "hello");
}

#[test]
fn or_returns_lhs_if_lhs_is_truthy() {
  evaluate("'hello' || 'goodbye'", "hello");
}

#[test]
fn empty_string_is_truthy() {
  evaluate("'' || 'fallback'", "");
  evaluate("'' && 'rhs'", "rhs");
}

#[test]
fn only_empty_list_is_falsy() {
  evaluate("'false' && 'rhs'", "rhs");
  evaluate("'0' && 'rhs'", "rhs");
}

#[test]
fn and_has_lower_precedence_than_plus() {
  evaluate("[] && 'goodbye' + 'foo'", "");
  evaluate("'foo' + 'hello' && 'goodbye'", "goodbye");
  evaluate("'foo' + '' && 'goodbye'", "goodbye");
  evaluate("'foo' + 'hello' && 'goodbye' + 'bar'", "goodbyebar");
}

#[test]
fn or_has_lower_precedence_than_plus() {
  evaluate("[] || 'goodbye' + 'foo'", "goodbyefoo");
  evaluate("'foo' + 'hello' || 'goodbye'", "foohello");
  evaluate("'foo' + '' || 'goodbye'", "foo");
  evaluate("'foo' + 'hello' || 'goodbye' + 'bar'", "foohello");
}

#[test]
fn and_has_higher_precedence_than_or() {
  evaluate("([] && 'foo') || 'bar'", "bar");
  evaluate("[] && 'foo' || 'bar'", "bar");
  evaluate("'a' && 'b' || 'c'", "b");
}

#[test]
fn nesting() {
  evaluate("[] || [] || [] || [] || 'foo'", "foo");
  evaluate("'foo' && 'foo' && 'foo' && 'foo' && 'bar'", "bar");
}
