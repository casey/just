use super::*;

#[test]
fn guard_lines_halt_executation() {
  Test::new()
    .justfile(
      "
        set guards

        @foo:
          ?[[ 'foo' == 'bar' ]]
          echo baz
      ",
    )
    .run();
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
    .run();
}
