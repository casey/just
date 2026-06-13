use super::*;

#[test]
fn list_literals_are_lists() {
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
fn empty_list_literal_is_falsy() {
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
fn list_literals_flatten_elements() {
  Test::new()
    .justfile(
      r#"
        set lists

        list := ["pre", ["x", "y"], "post"]

        foo:
          @echo "{{ quote(list) }}"
      "#,
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("'pre' 'x' 'y' 'post'\n")
    .success();
}

#[test]
fn list_literals_may_have_trailing_comma() {
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
fn list_literals_requires_lists_setting() {
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
fn list_literals_round_trip_through_dump() {
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
