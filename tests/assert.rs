use super::*;

test! {
  name: assert_pass,
  justfile: "
    foo:
      {{ assert('a' == 'a', 'error message') }}
  ",
  stdout: "",
  stderr: "",
}

test! {
  name: assert_fail,
  justfile: "
    foo:
      {{ assert('a' != 'a', 'error message') }}
  ",
  stdout: "",
  stderr: "error: Assert failed: error message\n",
  status: EXIT_FAILURE,
}
