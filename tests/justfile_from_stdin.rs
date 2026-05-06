use super::*;

#[test]
fn run() {
  Test::new()
    .no_justfile()
    .args(["--justfile", "-"])
    .stdin("foo:\n  echo bar\n")
    .stderr("echo bar\n")
    .stdout("bar\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn run_short() {
  Test::new()
    .no_justfile()
    .args(["-f", "-"])
    .stdin("foo:\n  echo bar\n")
    .stderr("echo bar\n")
    .stdout("bar\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn default_recipe() {
  Test::new()
    .no_justfile()
    .args(["--justfile", "-"])
    .stdin("foo:\n  echo bar\n\nbaz:\n  echo qux\n")
    .stderr("echo bar\n")
    .stdout("bar\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn list() {
  Test::new()
    .no_justfile()
    .args(["--justfile", "-", "--list"])
    .stdin("foo:\n  echo bar\n")
    .stdout(
      "
      Available recipes:
          foo
    ",
    )
    .test_round_trip(false)
    .success();
}

#[test]
fn evaluate() {
  Test::new()
    .no_justfile()
    .args(["--justfile", "-", "--evaluate"])
    .stdin("x := 'foo'\n")
    .stdout("x := \"foo\"\n")
    .unindent_stdout(false)
    .test_round_trip(false)
    .success();
}

#[test]
fn empty() {
  Test::new()
    .no_justfile()
    .args(["--justfile", "-"])
    .stdin("")
    .stderr("error: justfile contains no recipes\n")
    .test_round_trip(false)
    .failure();
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
fn init_error_with_working_directory() {
  Test::new()
    .no_justfile()
    .args(["--justfile", "-", "--working-directory", ".", "--init"])
    .stderr("error: cannot use justfile from standard input with `--init`\n")
    .test_round_trip(false)
    .failure();
}

#[test]
fn working_directory_long() {
  Test::new()
    .no_justfile()
    .tree(tree! {
      sub: {
        data: "qux\n",
      },
    })
    .args(["--justfile", "-", "--working-directory", "sub"])
    .stdin("foo:\n  cat data\n")
    .stderr("cat data\n")
    .stdout("qux\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn working_directory_short() {
  Test::new()
    .no_justfile()
    .tree(tree! {
      sub: {
        data: "qux\n",
      },
    })
    .args(["-f", "-", "-d", "sub"])
    .stdin("foo:\n  cat data\n")
    .stderr("cat data\n")
    .stdout("qux\n")
    .test_round_trip(false)
    .success();
}
