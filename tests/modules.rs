use super::*;

#[test]
fn import_succeeds() {
  Test::new()
    .tree(tree!(
    "foo.just":
    "
      bar := 'baz'

      foo:
        @echo {{bar}}
    ",
    ))
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .stdout("B\nA\n")
    .run();
}
