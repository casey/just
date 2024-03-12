use super::*;

#[test]
fn single_recipe() {
  Test::new()
    .justfile("foo:")
    .arg("--tree")
    .stdout(
      "
        └── foo
      ",
    )
    .run();
}

#[test]
fn multiple_recipes() {
  Test::new()
    .justfile(
      "
        foo:
        bar:
        baz:
      ",
    )
    .arg("--tree")
    .stdout(
      "
        ├── bar
        ├── baz
        └── foo
      ",
    )
    .run();
}

#[test]
fn dependencies() {
  Test::new()
    .justfile(
      "
        foo: bar baz
        bar: baz
        baz:
      ",
    )
    .arg("--tree")
    .stdout(
      "
        ├── bar
        │   └── baz
        ├── baz
        └── foo
            ├── bar
            └── baz
      ",
    )
    .run();
}

#[test]
fn submodule() {
  Test::new()
    .justfile(
      "
        mod foo
      ",
    )
    .write("foo.just", "bar:")
    .test_round_trip(false)
    .args(["--tree", "--unstable"])
    .stdout(
      "
        └── foo
            └── bar
      ",
    )
    .run();
}

#[test]
fn multiple_submodules() {
  Test::new()
    .justfile(
      "
        mod foo
        mod baz
      ",
    )
    .write("foo.just", "bar:")
    .write("baz.just", "qux:")
    .test_round_trip(false)
    .args(["--tree", "--unstable"])
    .stdout(
      "
        ├── baz
        │   └── qux
        └── foo
            └── bar
      ",
    )
    .run();
}
