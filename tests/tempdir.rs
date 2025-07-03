use super::*;

pub(crate) fn tempdir() -> TempDir {
  let mut builder = tempfile::Builder::new();

  builder.prefix("just-test-tempdir");

  if let Some(runtime_dir) = dirs::runtime_dir() {
    let path = runtime_dir.join("just");
    fs::create_dir_all(&path).unwrap();
    builder.tempdir_in(path)
  } else {
    builder.tempdir()
  }
  .expect("failed to create temporary directory")
}

#[test]
fn setting() {
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
      bar: {
      }
    })
    .current_dir("bar")
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

#[test]
fn argument_overrides_setting() {
  Test::new()
    .args(["--tempdir", "."])
    .justfile(
      "
      set tempdir := 'hello'
      foo:
          #!/usr/bin/env bash
          cat just*/foo
      ",
    )
    .shell(false)
    .tree(tree! {
      bar: {
      }
    })
    .current_dir("bar")
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
