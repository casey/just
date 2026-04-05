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

#[test]
fn time_reports_time_when_specified() {
  Test::new()
    .justfile(
      "
        foo:
          @echo FOO
      ",
    )
    .arg("--time")
    .stdout("FOO\n")
    .stderr_regex(r"---> foo completed in \d+\.\d+s\n")
    .success();
}
