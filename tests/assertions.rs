use super::*;

#[test]
fn assert_pass() {
  Test::new()
    .justfile("
    foo:
      {{ assert('a' == 'a', 'error message') }}
  ")
    .stdout("")
    .stderr("")
    .run();
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
