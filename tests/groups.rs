use super::*;

#[test]
fn list_groups() {
  Test::new()
    .justfile(
      "
        [group('A')]
        foo:

        [group('B')]
        [group('A')]
        bar:
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
        [group('A')]
        foo:

        [group('B')]
        [group('A')]
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
