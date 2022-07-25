use super::*;

test! {
  name: set_export_parse_error,
  justfile: "
  set export := fals
  ",
  stdout: "",
  stderr: "
    error: Expected keyword `true` or `false` but found identifier `fals`
      |
    1 | set export := fals
      |               ^^^^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: set_export_parse_error_EOL,
  justfile: "
  set export := fals
  ",
  stdout: "",
  stderr: "
    error: Expected keyword `true` or `false` but found `end of line`
      |
    1 | set export := 
      |               ^
  ",
  status: EXIT_FAILURE,
}