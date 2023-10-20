use super::*;

pub(crate) fn tempdir() -> TempDir {
  if cfg!(unix) {
    let mnt_point = mnt::get_mount(env::temp_dir()).unwrap();
    if mnt_point
      .clone()
      .unwrap()
      .mntops
      .contains(&mnt::MntOps::Exec(false))
    {
      let cache_dir = dirs::cache_dir().unwrap();
      env::set_var("TMPDIR", cache_dir);
    }
  }

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
