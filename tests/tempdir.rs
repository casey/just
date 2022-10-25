use super::*;

pub(crate) fn tempdir() -> tempfile::TempDir {
  tempfile::Builder::new()
    .prefix("just-test-tempdir")
    .tempdir()
    .expect("failed to create temporary directory")
}

#[test]
fn test_tempdir_is_set() {
  Test::new()
    .justfile(
      "
      set tempdir := '.'
      foo:
          #!/usr/bin/env bash
          cat just*/foo
      ",
    )
    .shell(false)
    .tree(tree! {
      foo: {
      }
    })
    .current_dir("foo")
    .stdout(
      "
      #!/usr/bin/env bash


      cat just*/foo
      ",
    )
    .run();
}
