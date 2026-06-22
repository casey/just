use super::*;

#[test]
fn cache_attribute_is_unstable() {
  Test::new()
    .justfile(
      "
        [cache]
        [script('sh')]
        foo:
          echo bar
      ",
    )
    .stderr_regex("error: cached recipes are currently unstable.*")
    .failure();
}

#[test]
fn cache_attribute_requires_script_recipe() {
  Test::new()
    .justfile(
      "
        [cache]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr(
      "
        error: recipe `foo` has invalid attribute `cache`
         ——▶ justfile:2:1
          │
        2 │ foo:
          │ ^^^
      ",
    )
    .failure();
}

#[test]
fn entry_is_created_with_empty_object() {
  let output = Test::new()
    .justfile(
      "
        [cache]
        [script('sh')]
        foo:
          echo bar >> count
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .expect_file("count", "bar\n")
    .success();

  let entries = fs::read_dir(output.tempdir.path().join(".justcache"))
    .unwrap()
    .map(|entry| entry.unwrap().path())
    .collect::<Vec<PathBuf>>();

  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].extension().unwrap(), "json");
  assert_eq!(fs::read_to_string(&entries[0]).unwrap(), "{}");
}

#[test]
fn hit_skips_execution() {
  let justfile = "
    [cache]
    [script('sh')]
    foo:
      echo bar >> count
  ";

  let output = Test::new()
    .justfile(justfile)
    .env("JUST_UNSTABLE", "1")
    .expect_file("count", "bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .justfile(justfile)
    .env("JUST_UNSTABLE", "1")
    .expect_file("count", "bar\n")
    .success();
}

#[test]
fn body_change_invalidates_cache() {
  let output = Test::new()
    .justfile(
      "
        [cache]
        [script('sh')]
        foo:
          echo bar >> count
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .expect_file("count", "bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .justfile(
      "
        [cache]
        [script('sh')]
        foo:
          echo baz >> count
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .expect_file("count", "bar\nbaz\n")
    .success();
}

#[test]
fn hit_prints_verbose_message() {
  let justfile = "
    [cache]
    [script('sh')]
    foo:
      echo bar >> count
  ";

  let output = Test::new()
    .justfile(justfile)
    .env("JUST_UNSTABLE", "1")
    .success();

  Test::with_tempdir(output.tempdir)
    .justfile(justfile)
    .env("JUST_UNSTABLE", "1")
    .arg("--verbose")
    .stderr(
      "
        ===> running recipe `foo`...
        ===> cache hit, skipping execution
      ",
    )
    .success();
}
