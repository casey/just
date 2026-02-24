use super::*;

#[test]
fn justfile_run_search_stops_at_ceiling_dir() {
  let tempdir = tempdir();

  let ceiling = tempdir.path().join("foo");

  fs::create_dir(&ceiling).unwrap();

  #[cfg(not(windows))]
  let ceiling = ceiling.canonicalize().unwrap();

  Test::with_tempdir(tempdir)
    .justfile(
      "
        foo:
          echo bar
      ",
    )
    .create_dir("foo/bar")
    .current_dir("foo/bar")
    .args(["--ceiling", ceiling.to_str().unwrap()])
    .stderr("error: No justfile found\n")
    .failure();
}

#[test]
fn ceiling_can_be_passed_as_environment_variable() {
  let tempdir = tempdir();

  let ceiling = tempdir.path().join("foo");

  fs::create_dir(&ceiling).unwrap();

  #[cfg(not(windows))]
  let ceiling = ceiling.canonicalize().unwrap();

  Test::with_tempdir(tempdir)
    .justfile(
      "
        foo:
          echo bar
      ",
    )
    .create_dir("foo/bar")
    .current_dir("foo/bar")
    .env("JUST_CEILING", ceiling.to_str().unwrap())
    .stderr("error: No justfile found\n")
    .failure();
}

#[test]
fn justfile_init_search_stops_at_ceiling_dir() {
  let tempdir = tempdir();

  let ceiling = tempdir.path().join("foo");

  fs::create_dir(&ceiling).unwrap();

  #[cfg(not(windows))]
  let ceiling = ceiling.canonicalize().unwrap();

  let Output { tempdir, .. } = Test::with_tempdir(tempdir)
    .no_justfile()
    .test_round_trip(false)
    .create_dir(".git")
    .create_dir("foo/bar")
    .current_dir("foo/bar")
    .args(["--init", "--ceiling", ceiling.to_str().unwrap()])
    .stderr_regex(if cfg!(windows) {
      r"Wrote justfile to `.*\\foo\\bar\\justfile`\n"
    } else {
      "Wrote justfile to `.*/foo/bar/justfile`\n"
    })
    .success();

  assert_eq!(
    fs::read_to_string(tempdir.path().join("foo/bar/justfile")).unwrap(),
    just::INIT_JUSTFILE
  );
}
