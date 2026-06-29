use super::*;

#[test]
fn equality_true() {
  assert_list(r#""foo" == "foo""#, TRUE);
}

#[test]
fn equality_false() {
  assert_list(r#""foo" == "bar""#, FALSE);
}

#[test]
fn inequality_true() {
  assert_list(r#""foo" != "bar""#, TRUE);
}

#[test]
fn inequality_false() {
  assert_list(r#""foo" != "foo""#, FALSE);
}

#[test]
fn list_equality_is_structural() {
  assert_list(r#"["foo", "bar"] == ["foo", "bar"]"#, TRUE);
}

#[test]
fn equality_distinguishes_element_boundaries() {
  assert_list(r#"["foo", "bar"] == ["foo bar"]"#, FALSE);
}

#[test]
fn inequality_distinguishes_element_boundaries() {
  assert_list(r#"["foo", "bar"] != ["foo bar"]"#, TRUE);
}

#[test]
fn empty_list_does_not_equal_empty_string() {
  assert_list(r#"[] == """#, FALSE);
}

#[test]
fn regex_match() {
  assert_list(r#""foo" =~ "f.""#, TRUE);
}

#[test]
fn regex_mismatch() {
  assert_list(r#""foo" !~ "b.""#, TRUE);
}

#[test]
fn regex_match_is_true_if_any_element_matches() {
  assert_list(r#"["foo", "bar"] =~ "b.""#, TRUE);
}

#[test]
fn regex_match_is_false_if_no_element_matches() {
  assert_list(r#"["foo", "bar"] =~ "z""#, FALSE);
}

#[test]
fn regex_match_of_empty_list_is_false() {
  assert_list(r#"[] =~ ".""#, FALSE);
}

#[test]
fn regex_mismatch_is_true_if_no_element_matches() {
  assert_list(r#"["foo", "bar"] !~ "z""#, TRUE);
}

#[test]
fn regex_mismatch_is_false_if_any_element_matches() {
  assert_list(r#"["foo", "bar"] !~ "b.""#, FALSE);
}

#[test]
fn regex_mismatch_of_empty_list_is_true() {
  assert_list(r#"[] !~ ".""#, TRUE);
}

#[test]
fn regex_match_with_list_is_true_if_any_pattern_matches() {
  assert_list(r#""foo" =~ ["x", "f."]"#, TRUE);
}

#[test]
fn regex_match_with_list_is_false_if_no_pattern_matches() {
  assert_list(r#""foo" =~ ["x", "y"]"#, FALSE);
}

#[test]
fn regex_match_with_empty_pattern_list_is_false() {
  assert_list(r#""foo" =~ []"#, FALSE);
}

#[test]
fn regex_mismatch_with_list_is_true_if_no_pattern_matches() {
  assert_list(r#""foo" !~ ["x", "y"]"#, TRUE);
}

#[test]
fn regex_mismatch_with_list_is_false_if_any_pattern_matches() {
  assert_list(r#""foo" !~ ["x", "f."]"#, FALSE);
}

#[test]
fn regex_mismatch_with_empty_pattern_list_is_true() {
  assert_list(r#""foo" !~ []"#, TRUE);
}

#[test]
fn regex_match_both_operands_lists() {
  assert_list(r#"["foo", "bar"] =~ ["z", "b."]"#, TRUE);
}

#[test]
fn combined_with_and() {
  assert_list(r#""foo" == "foo" && "bar" == "bar""#, TRUE);
}

#[test]
fn and_short_circuits_on_false_comparison() {
  assert_list(r#""foo" == "bar" && "baz" == "baz""#, FALSE);
}

#[test]
fn or_falls_through_false_comparison() {
  assert_list(r#""foo" == "bar" || "baz" == "baz""#, TRUE);
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
    .unstable()
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
    .unstable()
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
fn non_comparison_condition_calling_defined_function_requires_lists_setting() {
  Test::new()
    .justfile(
      r#"
        foo() := "t"

        x := if foo() { "t" } else { "f" }

        bar:
          @echo hi
      "#,
    )
    .unstable()
    .arg("bar")
    .stderr(
      r#"
        error: `if` and `assert` conditions other than comparisons require `set lists`
         ——▶ justfile:3:9
          │
        3 │ x := if foo() { "t" } else { "f" }
          │         ^^^
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
    .unstable()
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
    .unstable()
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
        error: expected '&&', '||', comment, end of file, end of line, '+', '++', or '/', but found '=='
         ——▶ justfile:1:21
          │
        1 │ x := "foo" == "bar" == "baz"
          │                     ^^
      "#,
    )
    .failure();
}
