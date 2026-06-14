use super::*;

#[test]
fn bool_true_values() {
  assert_list_eq(r#"bool("1")"#, TRUE);
  assert_list_eq(r#"bool("true")"#, TRUE);
}

#[test]
fn bool_false_values() {
  assert_list_eq("bool([])", FALSE);
  assert_list_eq(r#"bool("")"#, FALSE);
  assert_list_eq(r#"bool("0")"#, FALSE);
  assert_list_eq(r#"bool("false")"#, FALSE);
}

#[test]
fn bool_invalid_value() {
  Test::new()
    .justfile(
      "
        set lists

        x := bool('foo')
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"])
    .stderr(
      "
        error: call to function `bool` failed: `foo` is not a valid boolean string
         ——▶ justfile:3:6
          │
        3 │ x := bool('foo')
          │      ^^^^
      ",
    )
    .failure();
}

#[test]
fn bool_requires_lists_setting() {
  Test::new()
    .justfile("x := bool('true')")
    .args(["--evaluate", "x"])
    .stderr(
      "
        error: the `bool()` function requires `set lists`
         ——▶ justfile:1:6
          │
        1 │ x := bool('true')
          │      ^^^^
      ",
    )
    .failure();
}

#[test]
fn bool_multiple_elements() {
  Test::new()
    .justfile(
      "
        set lists

        x := bool(['foo', 'bar'])
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"])
    .stderr(
      "
        error: call to function `bool` failed: multi-element lists cannot be converted into booleans
         ——▶ justfile:3:6
          │
        3 │ x := bool(['foo', 'bar'])
          │      ^^^^
      ",
    )
    .failure();
}

#[test]
fn path_exists_true_is_true_string() {
  assert_list_eq("path_exists(justfile())", TRUE);
}

#[test]
fn path_exists_false_is_empty_list() {
  assert_list_eq(r#"path_exists("nonexistent")"#, FALSE);
}

#[test]
fn path_exists_false_falls_through_or() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ path_exists("nonexistent") || "fallback" }}
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("fallback\n")
    .success();
}

#[test]
fn path_exists_without_lists_returns_string() {
  Test::new()
    .justfile(
      r#"
        foo:
          @echo {{ path_exists("nonexistent") }}
      "#,
    )
    .arg("foo")
    .stdout("false\n")
    .success();
}

#[test]
fn is_dependency_false_falls_through_or() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ is_dependency() || "root" }}
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("root\n")
    .success();
}

#[test]
fn semver_matches_true_is_truthy() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ semver_matches("1.0.0", ">=1.0.0") || "no" }}
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("true\n")
    .success();
}

#[test]
fn semver_matches_false_falls_through_or() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ semver_matches("1.0.0", "<1.0.0") || "no" }}
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("no\n")
    .success();
}

#[test]
fn which_missing_falls_through_or() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ which("definitely-not-an-executable") || "fallback" }}
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("fallback\n")
    .success();
}
