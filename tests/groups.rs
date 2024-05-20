use super::*;

#[test]
fn list_with_groups() {
  Test::new()
    .justfile(
      "
        [group('alpha')]
        a:
        # Doc comment
        [group('alpha')]
        [group('beta')]
        b:
        c:
        [group('multi word group')]
        d:
        [group('alpha')]
        e:
        [group('beta')]
        [group('alpha')]
        f:
      ",
    )
    .arg("--list")
    .stdout(
      "
        Available recipes:

        (no group)
            c

        [alpha]
            a
            b # Doc comment
            e
            f

        [beta]
            b # Doc comment
            f

        [multi word group]
            d
      ",
    )
    .run();
}

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
