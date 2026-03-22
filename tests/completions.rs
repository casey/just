use super::*;

const FLAGS: &str = "(--[^\t\n]+\t[^\n]+\n)*";

fn complete_args<'a>(words: &[&'a str]) -> Vec<&'a str> {
  ["--", "just"]
    .into_iter()
    .chain(words.iter().copied())
    .collect()
}

#[test]
fn completion_scripts() {
  for shell in ["bash", "elvish", "fish", "nushell", "powershell", "zsh"] {
    Test::new()
      .args(["--completions", shell])
      .stdout_regex(if shell == "nushell" {
        ".*"
      } else {
        ".*JUST_COMPLETE.*"
      })
      .success();
  }
}

#[test]
fn recipes() {
  Test::new()
    .justfile(
      "
        foo:
        bar:
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&[""]))
    .stdout_regex(format!("bar\nfoo\n{FLAGS}"))
    .success();
}

#[test]
fn recipe_prefix_filter() {
  Test::new()
    .justfile(
      "
        foo:
        bar:
        baz:
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["b"]))
    .stdout("bar\nbaz\n")
    .success();
}

#[test]
fn private_recipes_excluded() {
  Test::new()
    .justfile(
      "
        foo:
        _bar:
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&[""]))
    .stdout_regex(format!("foo\n{FLAGS}"))
    .success();
}

#[test]
fn doc_comments() {
  Test::new()
    .justfile(
      "
        # doc
        foo:
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&[""]))
    .stdout_regex(format!("foo\tdoc\n{FLAGS}"))
    .success();
}

#[test]
fn show_recipes() {
  Test::new()
    .justfile(
      "
        foo:
        bar:
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--show", ""]))
    .stdout("bar\nfoo\n")
    .success();
}

#[test]
fn variable_completion() {
  Test::new()
    .justfile(
      "
        x := 'a'
        y := 'b'
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--set", ""]))
    .stdout("x\ny\n")
    .success();
}

#[test]
fn variable_completion_filters_by_prefix() {
  Test::new()
    .justfile(
      "
        foo := 'a'
        bar := 'b'
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--set", "f"]))
    .stdout("foo\n")
    .success();
}

#[test]
fn private_variables_excluded() {
  Test::new()
    .justfile(
      "
        x := 'a'
        _y := 'b'
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--set", ""]))
    .stdout("x\n")
    .success();
}

#[test]
fn argument_completion_includes_recipes_and_variables() {
  Test::new()
    .justfile(
      "
        foo:
        x := 'a'
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&[""]))
    .stdout_regex(format!("foo\nx=\n{FLAGS}"))
    .success();
}

#[test]
fn module_recipes() {
  Test::new()
    .justfile(
      "
        mod bar
      ",
    )
    .write("bar.just", "baz:")
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&[""]))
    .stdout_regex(format!("bar::baz\n{FLAGS}"))
    .success();
}

#[test]
fn justfile_flag_in_completion_words() {
  Test::new()
    .no_justfile()
    .write("foo.just", "bar:")
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(["--", "just", "--justfile", "foo.just", ""])
    .stdout_regex(format!("bar\n{FLAGS}"))
    .success();
}

#[test]
fn bash() {
  Test::new()
    .justfile("foo:")
    .shell(false)
    .env("JUST_COMPLETE", "bash")
    .env("_CLAP_COMPLETE_INDEX", "1")
    .args(complete_args(&[""]))
    .stdout_regex("foo\n.*")
    .success();
}

#[test]
fn elvish() {
  Test::new()
    .justfile("foo:")
    .shell(false)
    .env("JUST_COMPLETE", "elvish")
    .env("_CLAP_COMPLETE_INDEX", "1")
    .args(complete_args(&[""]))
    .stdout_regex("foo\n.*")
    .success();
}

#[test]
fn powershell() {
  Test::new()
    .justfile("foo:")
    .shell(false)
    .env("JUST_COMPLETE", "powershell")
    .args(complete_args(&[""]))
    .stdout_regex("foo\n.*")
    .success();
}

#[test]
fn zsh() {
  Test::new()
    .justfile("foo:")
    .shell(false)
    .env("JUST_COMPLETE", "zsh")
    .env("_CLAP_COMPLETE_INDEX", "1")
    .args(complete_args(&[""]))
    .stdout_regex("foo\n.*")
    .success();
}

#[test]
fn set_malformed_override_path() {
  Test::new()
    .justfile("foo:")
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--set", ":::", "foo", ""]))
    .stdout_regex(format!("foo\n{FLAGS}"))
    .success();
}

#[test]
fn positional_malformed_override() {
  Test::new()
    .justfile("foo:")
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&[":::=foo", ""]))
    .stdout_regex(format!("foo\n{FLAGS}"))
    .success();
}

#[test]
fn working_directory_without_justfile() {
  Test::new()
    .justfile("foo:")
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--working-directory", ".", ""]))
    .stdout_regex(format!("foo\n{FLAGS}"))
    .success();
}

#[test]
fn show_malformed_path() {
  Test::new()
    .justfile("foo:")
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--show", ":::", ""]))
    .stdout_regex(format!("(foo\n)+{FLAGS}"))
    .success();
}

#[test]
fn group_completion() {
  Test::new()
    .justfile(
      "
        [group: 'baz']
        foo:

        [group: 'bob']
        bar:
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--group", ""]))
    .stdout("baz\nbob\n")
    .success();
}

#[test]
fn group_completion_filters_by_prefix() {
  Test::new()
    .justfile(
      "
        [group: 'baz']
        foo:

        [group: 'bob']
        bar:
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--group", "ba"]))
    .stdout("baz\n")
    .success();
}

#[test]
fn usage_recipes() {
  Test::new()
    .justfile(
      "
        foo:
        bar:
      ",
    )
    .shell(false)
    .env("JUST_COMPLETE", "fish")
    .args(complete_args(&["--usage", ""]))
    .stdout("bar\nfoo\n")
    .success();
}
