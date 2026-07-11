use super::*;

#[test]
fn code_blocks_are_concatenated() {
  Test::new()
    .write(
      "foo.md",
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
    )
    .args(["--justfile", "foo.md"])
    .stdout("baz\n")
    .success();
}

#[test]
fn non_just_code_blocks_are_ignored() {
  Test::new()
    .write(
      "foo.md",
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
    )
    .args(["--justfile", "foo.md"])
    .stdout("bar\n")
    .success();
}

#[test]
fn extension_is_case_insensitive() {
  Test::new()
    .write(
      "foo.MD",
      "
        ```just
        @foo:
         echo bar
        ```
      ",
    )
    .args(["--justfile", "foo.MD"])
    .stdout("bar\n")
    .success();
}

#[test]
fn working_directory_is_markdown_file_directory() {
  Test::new()
    .write(
      "sub/foo.md",
      "
        ```just
        @foo:
         cat bar
        ```
      ",
    )
    .write("sub/bar", "baz")
    .args(["--justfile", "sub/foo.md"])
    .stdout("baz")
    .success();
}

#[test]
fn with_working_directory() {
  Test::new()
    .write(
      "foo.md",
      "
        ```just
        @foo:
         cat baz
        ```
      ",
    )
    .write("bar/baz", "qux")
    .args(["--justfile", "foo.md", "--working-directory", "bar"])
    .stdout("qux")
    .success();
}

#[test]
fn line_numbers_are_preserved() {
  Test::new()
    .write(
      "foo.md",
      "
        # foo

        ```just
        garbage[
        ```
      ",
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
    .failure();
}

#[test]
fn no_code_blocks() {
  Test::new()
    .write("foo.md", "# foo\n")
    .args(["--justfile", "foo.md"])
    .stderr("error: justfile contains no recipes\n")
    .failure();
}

#[test]
fn format_prints_to_stdout() {
  Test::new()
    .write(
      "foo.md",
      "
        ```just
        foo:
         echo bar
        ```
      ",
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
    .success();
}

#[test]
fn dump() {
  Test::new()
    .write(
      "foo.md",
      "
        ```just
        foo:
         echo bar
        ```
      ",
    )
    .args(["--dump", "--justfile", "foo.md"])
    .stdout("\nfoo:\n    echo bar\n")
    .unindent_stdout(false)
    .success();
}

#[test]
fn init_error() {
  Test::new()
    .write("foo.md", "# foo\n")
    .args(["--init", "--justfile", "foo.md"])
    .stderr_regex("error: justfile `.*foo.md` already exists\n")
    .failure();
}

#[test]
fn justfile_found_by_name_is_tangled() {
  Test::new()
    .write(
      "foo.md",
      "
        ```just
        @foo:
         echo bar
        ```
      ",
    )
    .args(["--justfile-name", "foo.md", "foo"])
    .stdout("bar\n")
    .success();
}
