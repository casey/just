use super::*;

#[test]
fn ignore_comments_in_recipe() {
  Test::new()
    .justfile(
      "
      set ignore-comments

      some_recipe:
        # A recipe-internal comment
        echo something-useful
    ",
    )
    .stdout("something-useful\n")
    .stderr("echo something-useful\n")
    .run();
}

#[test]
fn dont_ignore_comments_in_recipe_by_default() {
  Test::new()
    .justfile(
      "
      some_recipe:
        # A recipe-internal comment
        echo something-useful
    ",
    )
    .stdout("something-useful\n")
    .stderr("# A recipe-internal comment\necho something-useful\n")
    .run();
}

#[test]
fn ignore_recipe_comments_with_shell_setting() {
  Test::new()
    .justfile(
      "
      set shell := ['echo', '-n']
      set ignore-comments

      some_recipe:
        # Alternate shells still ignore comments
        echo something-useful
    ",
    )
    .stdout("something-useful\n")
    .stderr("echo something-useful\n")
    .run();
}

#[test]
fn continuations_with_echo_comments_false() {
  Test::new()
    .justfile(
      "
      set ignore-comments

      some_recipe:
        # Comment lines ignore line continuations \\
        echo something-useful
    ",
    )
    .stdout("something-useful\n")
    .stderr("echo something-useful\n")
    .run();
}

#[test]
fn continuations_with_echo_comments_true() {
  Test::new()
    .justfile(
      "
      set ignore-comments := false

      some_recipe:
        # comment lines can be continued \\
        echo something-useful
    ",
    )
    .stdout("")
    .stderr("# comment lines can be continued echo something-useful\n")
    .run();
}

#[test]
fn dont_evaluate_comments() {
  Test::new()
    .justfile(
      "
      set ignore-comments

      some_recipe:
        # {{ error('foo') }}
    ",
    )
    .run();
}

#[test]
fn dont_analyze_comments() {
  Test::new()
    .justfile(
      "
      set ignore-comments

      some_recipe:
        # {{ bar }}
    ",
    )
    .run();
}

#[test]
fn comments_still_must_be_parsable_when_ignored() {
  Test::new()
    .justfile(
      "
        set ignore-comments

        some_recipe:
          # {{ foo bar }}
      ",
    )
    .stderr(
      "
        error: Expected '}}', '(', '+', or '/', but found identifier
         ——▶ justfile:4:12
          │
        4 │   # {{ foo bar }}
          │            ^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}
