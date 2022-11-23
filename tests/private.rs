use super::*;

#[test]
fn attribute() {
  Test::new()
    .justfile(
      "
      [private]
      foo:
      ",
    )
    .args(&["--list"])
    .stdout(
      "
      Available recipes:
      ",
    )
    .run();
}
