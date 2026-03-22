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
    .stdout_regex("bar\nbaz\n")
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
