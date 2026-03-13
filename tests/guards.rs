use super::*;

#[test]
fn guard_lines_halt_execution() {
  Test::new()
    .justfile(
      "
        set guards

        @foo:
          ?[[ 'foo' == 'bar' ]]
          echo baz
      ",
    )
    .success();
}

#[test]
fn guard_lines_have_no_effect_if_successful() {
  Test::new()
    .justfile(
      "
        set guards

        @foo:
          ?[[ 'foo' == 'foo' ]]
          echo baz
      ",
    )
    .stdout("baz\n")
    .success();
}

#[test]
fn exit_codes_above_one_are_reserved() {
  Test::new()
    .justfile(
      "
        set guards

        @foo:
          ?exit 2
      ",
    )
    .stderr("error: Guard line in recipe `foo` on line 4 returned reserved exit code 2\n")
    .failure();
}
