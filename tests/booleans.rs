use super::*;

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
