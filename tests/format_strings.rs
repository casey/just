use crate::common::*;

test! {
  name: unknown_variable_in_format_string,
  justfile: "
    foo := f'{{unknown_variable}}'
  ",
  args: ("--evaluate"),
  stderr: "
    error: Variable `unknown_variable` not defined
      |
    1 | foo := f'{{unknown_variable}}'
      |            ^^^^^^^^^^^^^^^^
  ",
  status: EXIT_FAILURE,
}
