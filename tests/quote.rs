use super::*;

#[test]
fn single_quotes_are_prepended_and_appended() {
  assert_eval("quote('abc')", "'abc'");
}

#[test]
fn quotes_are_escaped() {
  assert_eval(r#"quote("'")"#, r"''\'''");
}

#[test]
fn quoted_strings_can_be_used_as_arguments() {
  Test::new()
    .justfile(
      r#"
        file := quote("foo ' bar")

        @foo:
          touch {{ file }}
          ls -1
      "#,
    )
    .stdout("foo ' bar\njustfile\n")
    .success();
}
