use super::*;

#[test]
fn equality_true() {
  assert_list_eq(r#""foo" == "foo""#, TRUE);
}

#[test]
fn equality_false() {
  assert_list_eq(r#""foo" == "bar""#, FALSE);
}

#[test]
fn inequality_true() {
  assert_list_eq(r#""foo" != "bar""#, TRUE);
}

#[test]
fn inequality_false() {
  assert_list_eq(r#""foo" != "foo""#, FALSE);
}

#[test]
fn list_equality_is_structural() {
  assert_list_eq(r#"["foo", "bar"] == ["foo", "bar"]"#, TRUE);
}

#[test]
fn equality_distinguishes_element_boundaries() {
  assert_list_eq(r#"["foo", "bar"] == ["foo bar"]"#, FALSE);
}

#[test]
fn inequality_distinguishes_element_boundaries() {
  assert_list_eq(r#"["foo", "bar"] != ["foo bar"]"#, TRUE);
}

#[test]
fn empty_list_does_not_equal_empty_string() {
  assert_list_eq(r#"[] == """#, FALSE);
}

#[test]
fn regex_match() {
  assert_list_eq(r#""foo" =~ "f.""#, TRUE);
}

#[test]
fn regex_mismatch() {
  assert_list_eq(r#""foo" !~ "b.""#, TRUE);
}

#[test]
fn regex_match_is_true_if_any_element_matches() {
  assert_list_eq(r#"["foo", "bar"] =~ "b.""#, TRUE);
}

#[test]
fn regex_match_is_false_if_no_element_matches() {
  assert_list_eq(r#"["foo", "bar"] =~ "z""#, FALSE);
}

#[test]
fn regex_match_of_empty_list_is_false() {
  assert_list_eq(r#"[] =~ ".""#, FALSE);
}

#[test]
fn regex_mismatch_is_true_if_no_element_matches() {
  assert_list_eq(r#"["foo", "bar"] !~ "z""#, TRUE);
}

#[test]
fn regex_mismatch_is_false_if_any_element_matches() {
  assert_list_eq(r#"["foo", "bar"] !~ "b.""#, FALSE);
}

#[test]
fn regex_mismatch_of_empty_list_is_true() {
  assert_list_eq(r#"[] !~ ".""#, TRUE);
}

#[test]
fn combined_with_and() {
  assert_list_eq(r#""foo" == "foo" && "bar" == "bar""#, TRUE);
}

#[test]
fn and_short_circuits_on_false_comparison() {
  assert_list_eq(r#""foo" == "bar" && "baz" == "baz""#, FALSE);
}

#[test]
fn or_falls_through_false_comparison() {
  assert_list_eq(r#""foo" == "bar" || "baz" == "baz""#, TRUE);
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
