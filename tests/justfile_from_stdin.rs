use super::*;

#[test]
fn no_working_directory() {
  Test::new()
    .no_justfile()
    .write("bar", "baz")
    .args(["--justfile", "-"])
    .stdin("@foo:\n cat bar\n")
    .stdout("baz")
    .test_round_trip(false)
    .success();
}

#[test]
fn parse_error() {
  Test::new()
    .no_justfile()
    .args(["--justfile", "-"])
    .stdin("garbage[\n")
    .stderr_regex("error: expected .* but found '\\['\n.*")
    .test_round_trip(false)
    .failure();
}

#[test]
fn init_error() {
  Test::new()
    .no_justfile()
    .args(["--justfile", "-", "--init"])
    .stderr("error: cannot use justfile from standard input with `--init`\n")
    .test_round_trip(false)
    .failure();
}

#[test]
fn with_working_directory() {
  Test::new()
    .no_justfile()
    .write("bar/baz", "qux")
    .args(["--justfile", "-", "--working-directory", "bar"])
    .stdin("@foo:\n  cat baz\n")
    .stdout("qux")
    .test_round_trip(false)
    .success();
}
