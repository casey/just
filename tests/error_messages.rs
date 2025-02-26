use super::*;

#[test]
fn invalid_alias_attribute() {
  Test::new()
    .justfile("[private]\n[linux]\nalias t := test\n\ntest:\n")
    .stderr(
      "
    error: Alias `t` has invalid attribute `linux`
     ——▶ justfile:3:7
      │
    3 │ alias t := test
      │       ^
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn expected_keyword() {
  Test::new()
    .justfile("foo := if '' == '' { '' } arlo { '' }")
    .stderr(
      "
    error: Expected keyword `else` but found identifier `arlo`
     ——▶ justfile:1:27
      │
    1 │ foo := if '' == '' { '' } arlo { '' }
      │                           ^^^^
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn unexpected_character() {
  Test::new()
    .justfile("&~")
    .stderr(
      "
    error: Expected character `&`
     ——▶ justfile:1:2
      │
    1 │ &~
      │  ^
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn argument_count_mismatch() {
  Test::new()
    .justfile("foo a b:")
    .args(["foo"])
    .stderr(
      "
      error: Recipe `foo` got 0 arguments but takes 2
      usage:
          just foo a b
    ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn file_path_is_indented_if_justfile_is_long() {
  Test::new()
    .justfile("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nfoo")
    .status(EXIT_FAILURE)
    .stderr(
      "
error: Expected '*', ':', '$', identifier, or '+', but found end of file
  ——▶ justfile:20:4
   │
20 │ foo
   │    ^
",
    )
    .run();
}

#[test]
fn file_paths_are_relative() {
  Test::new()
    .justfile("import 'foo/bar.just'")
    .write("foo/bar.just", "baz")
    .status(EXIT_FAILURE)
    .stderr(format!(
      "
error: Expected '*', ':', '$', identifier, or '+', but found end of file
 ——▶ foo{MAIN_SEPARATOR}bar.just:1:4
  │
1 │ baz
  │    ^
",
    ))
    .run();
}

#[test]
#[cfg(not(windows))]
fn file_paths_not_in_subdir_are_absolute() {
  Test::new()
    .write("foo/justfile", "import '../bar.just'")
    .write("bar.just", "baz")
    .no_justfile()
    .args(["--justfile", "foo/justfile"])
    .status(EXIT_FAILURE)
    .stderr_regex(
      r"error: Expected '\*', ':', '\$', identifier, or '\+', but found end of file
 ——▶ /.*/bar.just:1:4
  │
1 │ baz
  │    \^
",
    )
    .run();
}

#[test]
fn redefinition_errors_properly_swap_types() {
  Test::new()
    .write("foo.just", "foo:")
    .justfile("foo:\n echo foo\n\nmod foo 'foo.just'")
    .status(EXIT_FAILURE)
    .stderr(
      "
error: Recipe `foo` defined on line 1 is redefined as a module on line 4
 ——▶ justfile:4:5
  │
4 │ mod foo 'foo.just'
  │     ^^^
",
    )
    .run();
}
