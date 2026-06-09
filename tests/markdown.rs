use super::*;

#[test]
fn code_blocks_are_concatenated() {
  Test::new()
    .no_justfile()
    .write(
      "foo.md",
      "# foo\n\n```just\nbar := 'baz'\n```\nprose\n```just\n@bob:\n echo {{ bar }}\n```\n",
    )
    .args(["--justfile", "foo.md"])
    .stdout("baz\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn non_just_code_blocks_are_ignored() {
  Test::new()
    .no_justfile()
    .write(
      "foo.md",
      "```sh\ngarbage[\n```\n\n````\n```just\ngarbage[\n```\n````\n\n```just\n@foo:\n echo bar\n```\n",
    )
    .args(["--justfile", "foo.md"])
    .stdout("bar\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn extension_is_case_insensitive() {
  Test::new()
    .no_justfile()
    .write("foo.MD", "```just\n@foo:\n echo bar\n```\n")
    .args(["--justfile", "foo.MD"])
    .stdout("bar\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn working_directory_is_markdown_file_directory() {
  Test::new()
    .no_justfile()
    .write("sub/foo.md", "```just\n@foo:\n cat bar\n```\n")
    .write("sub/bar", "baz")
    .args(["--justfile", "sub/foo.md"])
    .stdout("baz")
    .test_round_trip(false)
    .success();
}

#[test]
fn with_working_directory() {
  Test::new()
    .no_justfile()
    .write("foo.md", "```just\n@foo:\n cat baz\n```\n")
    .write("bar/baz", "qux")
    .args(["--justfile", "foo.md", "--working-directory", "bar"])
    .stdout("qux")
    .test_round_trip(false)
    .success();
}

#[test]
fn line_numbers_are_preserved() {
  Test::new()
    .no_justfile()
    .write("foo.md", "# foo\n\n```just\ngarbage[\n```\n")
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
    .no_justfile()
    .write("foo.md", "# foo\n")
    .args(["--justfile", "foo.md"])
    .stderr("error: justfile contains no recipes\n")
    .test_round_trip(false)
    .failure();
}

#[test]
fn format_prints_to_stdout() {
  Test::new()
    .no_justfile()
    .write("foo.md", "```just\nfoo:\n echo bar\n```\n")
    .args(["--fmt", "--justfile", "foo.md"])
    .stdout("\nfoo:\n    echo bar\n")
    .unindent_stdout(false)
    .expect_file("foo.md", "```just\nfoo:\n echo bar\n```\n")
    .test_round_trip(false)
    .success();
}

#[test]
fn dump() {
  Test::new()
    .no_justfile()
    .write("foo.md", "```just\nfoo:\n echo bar\n```\n")
    .args(["--dump", "--justfile", "foo.md"])
    .stdout("\nfoo:\n    echo bar\n")
    .unindent_stdout(false)
    .test_round_trip(false)
    .success();
}

#[test]
fn init_error() {
  Test::new()
    .no_justfile()
    .write("foo.md", "# foo\n")
    .args(["--init", "--justfile", "foo.md"])
    .stderr_regex("error: justfile `.*foo.md` already exists\n")
    .test_round_trip(false)
    .failure();
}
