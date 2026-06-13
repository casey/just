use super::*;

#[test]
fn quote_distributes_over_a_literal() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo "{{ quote(["a", "b"]) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("'a' 'b'\n")
    .success();
}

#[test]
fn empty_literal_is_falsy() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ [] || "fallback" }}
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("fallback\n")
    .success();
}

#[test]
fn literals_flatten_element_values() {
  Test::new()
    .justfile(
      r#"
        set lists

        a := ["x", "y"]

        foo:
          @echo "{{ quote(["pre", a, "post"]) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("'pre' 'x' 'y' 'post'\n")
    .success();
}

#[test]
fn nested_literals_flatten() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo "{{ quote([["a", "b"], "c"]) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("'a' 'b' 'c'\n")
    .success();
}

#[test]
fn trailing_comma_is_accepted() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo "{{ quote(["a", "b",]) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("'a' 'b'\n")
    .success();
}

#[test]
fn literals_join_in_interpolation() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ ["a", "b", "c"] }}
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("a b c\n")
    .success();
}

#[test]
fn mapped_dependency_over_a_literal() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo: *(bar *["a", "b c"])

        bar arg:
          @echo "bar: {{ arg }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("bar: a\nbar: b c\n")
    .success();
}

#[test]
fn literal_requires_lists_setting() {
  Test::new()
    .justfile(
      "
        x := []

        foo:
          @echo hi
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: list literals require `set lists`
         ——▶ justfile:1:6
          │
        1 │ x := []
          │      ^
      ",
    )
    .failure();
}

#[test]
fn literal_with_lists_set_to_false_requires_lists_setting() {
  Test::new()
    .justfile(
      "
        set lists := false

        x := []

        foo:
          @echo hi
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: list literals require `set lists`
         ——▶ justfile:3:6
          │
        3 │ x := []
          │      ^
      ",
    )
    .failure();
}

#[test]
fn literal_with_lists_set_is_unstable() {
  Test::new()
    .justfile(
      r#"
        set lists

        foo:
          @echo {{ quote(["a"]) }}
      "#,
    )
    .stderr_regex("error: the `lists` setting is currently unstable.*")
    .failure();
}

#[test]
fn literals_round_trip_through_dump() {
  Test::new()
    .justfile(
      r#"
        set lists

        x := ["a", "b", "c"]

        foo:
          @echo "{{ quote(x) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("--dump")
    .stdout(
      r#"
        set lists

        x := ["a", "b", "c"]

        foo:
            @echo "{{ quote(x) }}"
      "#,
    )
    .success();
}
