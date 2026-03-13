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

#[test]
fn guard_sigil_may_not_be_used_with_infallible_sigil() {
  Test::new()
    .justfile(
      "
        set guards

        @foo:
          -?exit 2
      ",
    )
    .stderr(
      "
        error: The guard `?` and infallible `-` sigils may not be used together
         ——▶ justfile:4:3
          │
        4 │   -?exit 2
          │   ^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn guard_lines_are_ignored_without_setting() {
  Test::new()
    .justfile(
      "
        @foo:
          ?() { echo bar; }; ?
      ",
    )
    .stdout("bar\n")
    .success();
}
