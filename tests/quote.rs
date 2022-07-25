use super::*;

#[test]
fn single_quotes_are_prepended_and_appended() {
  Test::new()
    .justfile(
      "
      x := quote('abc')
    ",
    )
    .args(&["--evaluate", "x"])
    .stdout("'abc'")
    .run();
}

#[test]
fn quotes_are_escaped() {
  Test::new()
    .justfile(
      r#"
      x := quote("'")
    "#,
    )
    .args(&["--evaluate", "x"])
    .stdout(r"''\'''")
    .run();
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
    .run();
}
