use super::*;

test! {
    name: invalid_alias_attribute,
    justfile: "[private]\n[linux]\nalias t := test\n\ntest:\n",
    stderr: "
      error: Alias t has an invalid attribute `linux`
        |
      3 | alias t := test
        |       ^
    ",
    status: EXIT_FAILURE,
}

test! {
  name: expected_keyword,
  justfile: "foo := if '' == '' { '' } arlo { '' }",
  stderr: "
    error: Expected keyword `else` but found identifier `arlo`
      |
    1 | foo := if '' == '' { '' } arlo { '' }
      |                           ^^^^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: unexpected_character,
  justfile: "&~",
  stderr: "
    error: Expected character `&`
      |
    1 | &~
      |  ^
  ",
  status: EXIT_FAILURE,
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
