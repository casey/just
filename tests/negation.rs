use super::*;

#[test]
fn negates_truthy_value() {
  assert_list_eq("!'foo'", FALSE);
}

#[test]
fn negates_empty_list() {
  assert_list_eq("![]", TRUE);
}

#[test]
fn requires_lists_setting() {
  Test::new()
    .justfile(
      "
        x := !'foo'

        foo:
          @echo hi
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stderr(
      "
        error: negation operator requires `set lists`
         ——▶ justfile:1:6
          │
        1 │ x := !'foo'
          │      ^
      ",
    )
    .failure();
}
