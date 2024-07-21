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
fn list_with_groups_unsorted() {
  Test::new()
    .justfile(
      "
        [group('beta')]
        [group('alpha')]
        f:

        [group('alpha')]
        e:

        [group('multi word group')]
        d:

        c:

        # Doc comment
        [group('alpha')]
        [group('beta')]
        b:

        [group('alpha')]
        a:

      ",
    )
    .args(["--list", "--unsorted"])
    .stdout(
      "
        Available recipes:
            c

            [alpha]
            f
            e
            b # Doc comment
            a

            [beta]
            f
            b # Doc comment

            [multi word group]
            d
      ",
    )
    .run();
}

#[test]
fn list_with_groups_unsorted_group_order() {
  Test::new()
    .justfile(
      "
        [group('y')]
        [group('x')]
        f:

        [group('b')]
        b:

        [group('a')]
        e:

        c:
      ",
    )
    .args(["--list", "--unsorted"])
    .stdout(
      "
        Available recipes:
            c

            [x]
            f

            [y]
            f

            [b]
            b

            [a]
            e
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

#[test]
fn list_groups_with_shorthand_syntax() {
  Test::new()
    .justfile(
      "
        [group: 'B']
        foo:

        [group: 'A', group: 'B']
        bar:
      ",
    )
    .arg("--groups")
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
fn list_groups_unsorted() {
  Test::new()
    .justfile(
      "
        [group: 'Z']
        baz:

        [group: 'B']
        foo:

        [group: 'A', group: 'B']
        bar:
      ",
    )
    .args(["--groups", "--unsorted"])
    .stdout(
      "
      Recipe groups:
          Z
          B
          A
      ",
    )
    .run();
}

#[test]
fn list_groups_private_unsorted() {
  Test::new()
    .justfile(
      "
        [private]
        [group: 'A']
        foo:

        [group: 'B']
        bar:

        [group: 'A']
        baz:
      ",
    )
    .args(["--groups", "--unsorted"])
    .stdout(
      "
      Recipe groups:
          B
          A
      ",
    )
    .run();
}

#[test]
fn list_groups_private() {
  Test::new()
    .justfile(
      "
        [private]
        [group: 'A']
        foo:

        [group: 'B']
        bar:
      ",
    )
    .args(["--groups", "--unsorted"])
    .stdout(
      "
      Recipe groups:
          B
      ",
    )
    .run();
}
