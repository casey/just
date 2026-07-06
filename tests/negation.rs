use super::*;

#[test]
fn negates_truthy_value() {
  assert_list("!'foo'", FALSE);
}

#[test]
fn negates_empty_list() {
  assert_list("![]", TRUE);
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
    .unstable()
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

#[test]
fn negate_include() {
  Test::new()
    .justfile(
      "
        set unstable
        set lists

        include := []

        x := !include
      ",
    )
    .args(["--evaluate", "x"])
    .stdout("true")
    .success();
}
