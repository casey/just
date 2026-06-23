use super::*;

#[test]
fn code_blocks_are_concatenated() {
  Test::new()
    .write(
      "foo.md",
      unindent(
        "
          # foo

          ```just
          bar := 'baz'
          ```
          prose
          ```just
          @bob:
           echo {{ bar }}
          ```
        ",
      ),
    )
    .args(["--justfile", "foo.md"])
    .stdout("baz\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn non_just_code_blocks_are_ignored() {
  Test::new()
    .write(
      "foo.md",
      unindent(
        "
          ```sh
          garbage[
          ```

          ````
          ```just
          garbage[
          ```
          ````

          ```just
          @foo:
           echo bar
          ```
        ",
      ),
    )
    .args(["--justfile", "foo.md"])
    .stdout("bar\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn extension_is_case_insensitive() {
  Test::new()
    .write(
      "foo.MD",
      unindent(
        "
          ```just
          @foo:
           echo bar
          ```
        ",
      ),
    )
    .args(["--justfile", "foo.MD"])
    .stdout("bar\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn working_directory_is_markdown_file_directory() {
  Test::new()
    .write(
      "sub/foo.md",
      unindent(
        "
          ```just
          @foo:
           cat bar
          ```
        ",
      ),
    )
    .write("sub/bar", "baz")
    .args(["--justfile", "sub/foo.md"])
    .stdout("baz")
    .test_round_trip(false)
    .success();
}

#[test]
fn with_working_directory() {
  Test::new()
    .write(
      "foo.md",
      unindent(
        "
          ```just
          @foo:
           cat baz
          ```
        ",
      ),
    )
    .write("bar/baz", "qux")
    .args(["--justfile", "foo.md", "--working-directory", "bar"])
    .stdout("qux")
    .test_round_trip(false)
    .success();
}

#[test]
fn line_numbers_are_preserved() {
  Test::new()
    .write(
      "foo.md",
      unindent(
        "
          # foo

          ```just
          garbage[
          ```
        ",
      ),
    )
    .args(["--justfile", "foo.md"])
    .stderr(
      "
        error: expected '*', ':', '$', identifier, or '+', but found '['
         ——▶ justfile:4:8
          │
        4 │ garbage[
          │        ^
      ",
    )
    .test_round_trip(false)
    .failure();
}

#[test]
fn no_code_blocks() {
  Test::new()
    .write("foo.md", "# foo\n")
    .args(["--justfile", "foo.md"])
    .stderr("error: justfile contains no recipes\n")
    .test_round_trip(false)
    .failure();
}

#[test]
fn format_prints_to_stdout() {
  Test::new()
    .write(
      "foo.md",
      unindent(
        "
          ```just
          foo:
           echo bar
          ```
        ",
      ),
    )
    .args(["--fmt", "--justfile", "foo.md"])
    .stdout("\nfoo:\n    echo bar\n")
    .unindent_stdout(false)
    .expect_file(
      "foo.md",
      unindent(
        "
          ```just
          foo:
           echo bar
          ```
        ",
      ),
    )
    .test_round_trip(false)
    .success();
}

#[test]
fn dump() {
  Test::new()
    .write(
      "foo.md",
      unindent(
        "
          ```just
          foo:
           echo bar
          ```
        ",
      ),
    )
    .args(["--dump", "--justfile", "foo.md"])
    .stdout("\nfoo:\n    echo bar\n")
    .unindent_stdout(false)
    .test_round_trip(false)
    .success();
}

#[test]
fn init_error() {
  Test::new()
    .write("foo.md", "# foo\n")
    .args(["--init", "--justfile", "foo.md"])
    .stderr_regex("error: justfile `.*foo.md` already exists\n")
    .test_round_trip(false)
    .failure();
}
