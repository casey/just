use super::*;

pub(crate) fn tempdir() -> TempDir {
  let mut builder = tempfile::Builder::new();

  builder.prefix("just-test-tempdir");

  if let Some(cache_dir) = dirs::cache_dir() {
    let path = cache_dir.join("just");
    fs::create_dir_all(&path).unwrap();
    builder.tempdir_in(path)
  } else {
    builder.tempdir()
  }
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
    .stdout(if cfg!(windows) {
      "




      cat just*/foo
      "
    } else {
      "
      #!/usr/bin/env bash


      cat just*/foo
      "
    })
    .run();
}
