use super::*;

#[test]
fn list_groups() {
  Test::new()
    .justfile(
      "
        [group('B')]
        bar:

        [group('A')]
        [group('B')]
        foo:

      ",
    )
    .args(["--groups"])
    .stdout(
      "
      Recipe groups:
          A
          B
      ",
    )
    .run();
}

#[test]
fn list_groups_with_custom_prefix() {
  Test::new()
    .justfile(
      "
        [group('B')]
        foo:

        [group('A')]
        [group('B')]
        bar:
      ",
    )
    .args(["--groups", "--list-prefix", "..."])
    .stdout(
      "
      Recipe groups:
      ...A
      ...B
      ",
    )
    .run();
}
