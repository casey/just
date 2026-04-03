use super::*;

#[test]
fn dont_run_duplicate_recipes() {
  Test::new()
    .justfile(
      "
        @foo:
          echo foo
      ",
    )
    .args(["foo", "foo"])
    .stdout("foo\n")
    .success();
}

#[test]
fn one_flag_only_allows_one_invocation() {
  Test::new()
    .justfile(
      "
        @foo:
          echo foo
      ",
    )
    .args(["--one", "foo"])
    .stdout("foo\n")
    .success();

  Test::new()
    .justfile(
      "
        @foo:
          echo foo

        @bar:
          echo bar
      ",
    )
    .args(["--one", "foo", "bar"])
    .stderr("error: Expected 1 command-line recipe invocation but found 2.\n")
    .failure();
}
