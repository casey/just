use super::*;

fn test() -> (Test, String, String) {
  let tempdir = tempdir();

  let ceiling = if cfg!(not(windows)) {
    tempdir.path().canonicalize().unwrap()
  } else {
    tempdir.path().to_owned()
  };

  let global = tempdir.path().join("global").join("justfile");

  (
    Test::with_tempdir(tempdir),
    ceiling.to_str().unwrap().to_owned(),
    global.to_str().unwrap().to_owned(),
  )
}

#[test]
fn used_when_no_justfile_is_found() {
  let (test, ceiling, global) = test();

  test
    .write(
      "global/justfile",
      "
        @foo:
          echo bar
      ",
    )
    .create_dir("baz")
    .current_dir("baz")
    .args([
      "--ceiling",
      &ceiling,
      "--global-fallback-justfile",
      &global,
      "foo",
    ])
    .stdout("bar\n")
    .success();
}

#[test]
fn can_be_passed_as_environment_variable() {
  let (test, ceiling, global) = test();

  test
    .write(
      "global/justfile",
      "
        @foo:
          echo bar
      ",
    )
    .create_dir("baz")
    .current_dir("baz")
    .env("JUST_GLOBAL_FALLBACK_JUSTFILE", &global)
    .args(["--ceiling", &ceiling, "foo"])
    .stdout("bar\n")
    .success();
}

#[test]
fn working_directory_is_project_root() {
  let (test, ceiling, global) = test();

  test
    .write(
      "global/justfile",
      "
        @foo:
          echo bar > baz
      ",
    )
    .create_dir(".git")
    .create_dir("foo/bar")
    .current_dir("foo/bar")
    .args([
      "--ceiling",
      &ceiling,
      "--global-fallback-justfile",
      &global,
      "foo",
    ])
    .expect_file("baz", "bar\n")
    .success();
}

#[test]
fn used_when_fallback_chain_is_exhausted() {
  let (test, ceiling, global) = test();

  test
    .write(
      "global/justfile",
      "
        @foo:
          echo bar
      ",
    )
    .write(
      "baz/justfile",
      "
        set fallback

        @bar:
          echo baz
      ",
    )
    .current_dir("baz")
    .args([
      "--ceiling",
      &ceiling,
      "--global-fallback-justfile",
      &global,
      "--verbose",
      "foo",
    ])
    .stderr(format!(
      "
        Trying {global}
        ===> running recipe `foo`...
        echo bar
      "
    ))
    .stdout("bar\n")
    .success();
}

#[test]
fn used_at_most_once() {
  let (test, ceiling, global) = test();

  test
    .write(
      "global/justfile",
      "
        set fallback

        @bar:
          echo bar
      ",
    )
    .write(
      "baz/justfile",
      "
        set fallback

        @bar:
          echo baz
      ",
    )
    .current_dir("baz")
    .args([
      "--ceiling",
      &ceiling,
      "--global-fallback-justfile",
      &global,
      "foo",
    ])
    .stderr("error: justfile does not contain recipe `foo`\n")
    .failure();
}

#[test]
fn ignored_with_justfile() {
  let (test, ceiling, global) = test();

  test
    .write(
      "global/justfile",
      "
        @foo:
          echo bar
      ",
    )
    .write(
      "baz/justfile",
      "
        @bar:
          echo baz
      ",
    )
    .args([
      "--ceiling",
      &ceiling,
      "--global-fallback-justfile",
      &global,
      "--justfile",
      "baz/justfile",
      "foo",
    ])
    .stderr("error: justfile does not contain recipe `foo`\n")
    .failure();
}
