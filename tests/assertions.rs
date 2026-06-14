use super::*;

#[test]
fn assert_pass() {
  Test::new()
    .justfile(
      "
        foo:
          {{ assert('a' == 'a', 'error message') }}
      ",
    )
    .success();
}

#[test]
fn assert_fail() {
  Test::new()
    .justfile(
      "
        foo:
          {{ assert('a' != 'a', 'error message') }}
      ",
    )
    .stderr(
      "
        error: assert failed: error message
         ——▶ justfile:2:6
          │
        2 │   {{ assert('a' != 'a', 'error message') }}
          │      ^^^^^^
      ",
    )
    .failure();
}

#[test]
fn assert_evaluates_to_condition_with_lists() {
  assert_list_eq("assert(['foo', 'bar'], 'fail')", r#"["foo", "bar"]"#);
}

#[test]
fn assert_empty_string_without_lists() {
  Test::new()
    .justfile("x := assert('a' == 'a', 'fail')")
    .args(["--evaluate", "x"])
    .stdout("")
    .unindent_stdout(false)
    .success();
}

#[test]
fn assert_true_in_setting_with_lists() {
  Test::new()
    .justfile(
      "
        set lists
        set dotenv-filename := assert('a' == 'a', 'fail')

        foo:
          @echo $KEY
      ",
    )
    .write("true", "KEY=VAR\n")
    .env("JUST_UNSTABLE", "1")
    .stdout("VAR\n")
    .success();
}
