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
fn logical_operators_require_lists_setting() {
  #[track_caller]
  fn case(expression: &str, operator: &str, column: usize) {
    let carets = "^".repeat(operator.len());
    Test::new()
      .justfile(format!(
        "
          x := {expression}

          foo:
            @echo hi
        "
      ))
      .env("JUST_UNSTABLE", "1")
      .arg("foo")
      .stderr(format!(
        "
          error: logical operators require `set lists`
           ——▶ justfile:1:{column}
            │
          1 │ x := {expression}
            │ {0}{carets}
        ",
        " ".repeat(column - 1),
      ))
      .failure();
  }

  case("'foo' && 'bar'", "&&", 12);
  case("'foo' || 'bar'", "||", 12);
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
