use super::*;

#[test]
fn argument_with_different_path_prefix_is_allowed() {
  Test::new()
    .justfile("foo bar:")
    .args(["./foo", "../bar"])
    .success();
}

#[test]
fn passing_dot_as_argument_is_allowed() {
  Test::new()
    .justfile(
      "
        say ARG:
          echo {{ARG}}
      ",
    )
    .write(
      "child/justfile",
      "
        say ARG:
         '{{just_executable()}}' ../say {{ARG}}
      ",
    )
    .current_dir("child")
    .args(["say", "."])
    .stdout(".\n")
    .stderr_regex(unindent(
      "
        '.*' ../say .
        echo .
      ",
    ))
    .success();
}
