use crate::common::*;

#[test]
fn infallible_after_quiet() {
  Test::new()
    .justfile(
      "
        foo:
          @-exit 1
      ",
    )
    .run();
}

#[test]
fn quiet_after_infallible() {
  Test::new()
    .justfile(
      "
        foo:
          -@exit 1
      ",
    )
    .run();
}
