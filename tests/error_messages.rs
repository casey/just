use crate::common::*;

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
  justfile: "!~",
  stderr: "
    error: Expected character `=`
      |
    1 | !~
      |  ^
  ",
  status: EXIT_FAILURE,
}
