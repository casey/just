use super::*;
pub(crate) fn tempdir() -> tempfile::TempDir {
  tempfile::Builder::new()
    .prefix("just-test-tempdir")
    .tempdir()
    .expect("failed to create temporary directory")
}

#[test]
fn test_tempdir_is_set() {
  let foo_contents = r#"
      #!/usr/bin/env bash


      touch $(basename "$(dirname $0)")/foo
      cat $(basename "$(dirname $0)")/foo
      "#;
  Test::new()
    .justfile(
      r#"
      set tempdir := "."
      foo:
          #!/usr/bin/env bash
          touch $(basename "$(dirname $0)")/foo
          cat $(basename "$(dirname $0)")/foo
    "#,
    )
    .shell(false)
    .stdout(foo_contents)
    .stderr("")
    .run();
}
