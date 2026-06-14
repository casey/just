use super::*;

#[test]
fn equality_true() {
  assert_list_eq(r#""foo" == "foo""#, r#""true""#);
}

#[test]
fn equality_false() {
  assert_list_eq(r#""foo" == "bar""#, "[]");
}

#[test]
fn inequality_true() {
  assert_list_eq(r#""foo" != "bar""#, r#""true""#);
}

#[test]
fn inequality_false() {
  assert_list_eq(r#""foo" != "foo""#, "[]");
}

#[test]
fn regex_match() {
  assert_list_eq(r#""foo" =~ "f.""#, r#""true""#);
}

#[test]
fn regex_mismatch() {
  assert_list_eq(r#""foo" !~ "b.""#, r#""true""#);
}

#[test]
fn combined_with_and() {
  assert_list_eq(r#""foo" == "foo" && "bar" == "bar""#, r#""true""#);
}

#[test]
fn and_short_circuits_on_false_comparison() {
  assert_list_eq(r#""foo" == "bar" && "baz" == "baz""#, "[]");
}

#[test]
fn or_falls_through_false_comparison() {
  assert_list_eq(r#""foo" == "bar" || "baz" == "baz""#, r#""true""#);
}

#[test]
fn value_comparison_requires_lists_setting() {
  Test::new()
    .justfile(
      r#"
        x := "foo" == "bar"

        foo:
          @echo hi
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      r#"
        error: comparison operators require `set lists`
         ——▶ justfile:1:12
          │
        1 │ x := "foo" == "bar"
          │            ^^
      "#,
    )
    .failure();
}

#[test]
fn non_comparison_condition_requires_lists_setting() {
  Test::new()
    .justfile(
      r#"
        x := if "foo" { "t" } else { "f" }

        foo:
          @echo hi
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      r#"
        error: `if` and `assert` conditions other than comparisons require `set lists`
         ——▶ justfile:1:9
          │
        1 │ x := if "foo" { "t" } else { "f" }
          │         ^^^^^
      "#,
    )
    .failure();
}

#[test]
fn comparison_condition_without_lists() {
  Test::new()
    .justfile(
      r#"
        foo:
          @echo {{ if "foo" == "foo" { "yes" } else { "no" } }}
      "#,
    )
    .arg("foo")
    .stdout("yes\n")
    .success();
}

#[test]
fn empty_list_is_falsy_condition() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ if [] { "t" } else { "f" } }}
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("f\n")
    .success();
}

#[test]
fn non_empty_value_is_truthy_condition() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ if "x" { "t" } else { "f" } }}
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("t\n")
    .success();
}

#[test]
fn comparisons_are_not_associative() {
  Test::new()
    .justfile(
      r#"
        x := "foo" == "bar" == "baz"

        foo:
          @echo hi
      "#,
    )
    .stderr(
      r#"
        error: expected '&&', '||', comment, end of file, end of line, '+', or '/', but found '=='
         ——▶ justfile:1:21
          │
        1 │ x := "foo" == "bar" == "baz"
          │                     ^^
      "#,
    )
    .failure();
}
