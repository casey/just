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
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
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
  let output = Test::new()
    .justfile(
      "
        [cache]
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .success();
}

#[test]
fn body_change_invalidates_cache() {
  let output = Test::new()
    .justfile(
      "
        [cache]
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .justfile(
      "
        [cache]
        [script]
        foo:
          echo baz
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("baz\n")
    .success();
}

#[test]
fn different_recipes_do_not_share_entries() {
  let output = Test::new()
    .justfile(
      "
        [cache]
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("foo")
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .justfile(
      "
        [cache]
        [script]
        bob:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("bob")
    .stdout("bar\n")
    .success();
}

#[test]
fn positional_arguments_invalidate_cache() {
  let output = Test::new()
    .justfile(
      "
        [cache]
        [positional-arguments]
        [script]
        foo *args:
          echo $1
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "bar"])
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .args(["foo", "baz"])
    .stdout("baz\n")
    .success();
}

#[test]
fn environment_invalidates_cache() {
  let output = Test::new()
    .justfile(
      "
        export value := 'default'

        [cache]
        [script]
        foo:
          echo $value
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["value=bar", "foo"])
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .args(["value=baz", "foo"])
    .stdout("baz\n")
    .success();
}

#[test]
fn unexported_variable_does_not_invalidate_cache() {
  let output = Test::new()
    .justfile(
      "
        value := 'default'

        [cache]
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["value=bar", "foo"])
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .args(["value=baz", "foo"])
    .success();
}

#[test]
fn interpreter_invalidates_cache() {
  let output = Test::new()
    .justfile(
      "
        set script-interpreter := ['sh', '-eu']

        [cache]
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .justfile(
      "
        set script-interpreter := ['sh', '-u']

        [cache]
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();
}

#[test]
fn working_directory_invalidates_cache() {
  let output = Test::new()
    .justfile(
      "
        [cache]
        [working-directory('a')]
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .create_dir("a")
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .justfile(
      "
        [cache]
        [working-directory('b')]
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .create_dir("b")
    .stdout("bar\n")
    .success();
}

#[test]
fn input_invalidates_cache() {
  let output = Test::new()
    .justfile(
      "
        [cache(inputs = 'foo')]
        [script]
        bar:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write("foo", "a")
    .stdout("bar\n")
    .success();

  let output = Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .write("foo", "b")
    .stdout("bar\n")
    .success();
}

#[test]
fn multiple_inputs_invalidate_cache() {
  let output = Test::new()
    .justfile(
      "
        set lists

        [cache(inputs = ['foo', 'baz'])]
        [script]
        bar:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write("foo", "a")
    .write("baz", "a")
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .write("baz", "b")
    .stdout("bar\n")
    .success();
}

#[test]
fn input_expression_evaluated_with_arguments() {
  let output = Test::new()
    .justfile(
      "
        [cache(inputs = file)]
        [script]
        bar file:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write("foo", "a")
    .args(["bar", "foo"])
    .stdout("bar\n")
    .success();

  let output = Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .args(["bar", "foo"])
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .write("foo", "b")
    .args(["bar", "foo"])
    .stdout("bar\n")
    .success();
}

#[test]
fn symlink_to_file_is_followed() {
  let output = Test::new()
    .justfile(
      "
        [cache(inputs = 'link')]
        [script]
        bar:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .write("foo", "a")
    .symlink("foo", "link")
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .success();
}

#[test]
fn missing_input_is_an_error() {
  Test::new()
    .justfile(
      "
        [cache(inputs = 'foo')]
        [script]
        bar:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stderr_regex(r"error: cache input does not exist: `.*foo`\n")
    .failure();
}

#[test]
fn directory_input_is_an_error() {
  Test::new()
    .justfile(
      "
        [cache(inputs = 'foo')]
        [script]
        bar:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .create_dir("foo")
    .stderr_regex(r"error: cache input is directory: `.*foo`\n")
    .failure();
}

#[test]
fn symlink_to_directory_is_an_error() {
  Test::new()
    .justfile(
      "
        [cache(inputs = 'link')]
        [script]
        bar:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .create_dir("foo")
    .symlink("foo", "link")
    .stderr_regex(r"error: cache input is directory: `.*link`\n")
    .failure();
}

#[test]
fn dry_run_skips_input_checking() {
  Test::new()
    .justfile(
      "
        [cache(inputs = 'foo')]
        [script]
        bar:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("--dry-run")
    .stderr("echo bar\n")
    .success();
}

#[test]
fn missing_output_invalidates_cache() {
  let output = Test::new()
    .justfile(
      "
        [cache(outputs = 'foo')]
        [script]
        bar:
          echo bar
          touch foo
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();

  let output = Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .success();

  fs::remove_file(output.tempdir.path().join("foo")).unwrap();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();
}

#[test]
fn output_expression_evaluated_with_arguments() {
  let output = Test::new()
    .justfile(
      "
        [cache(outputs = file)]
        [script]
        bar file:
          echo bar
          touch {{file}}
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["bar", "foo"])
    .stdout("bar\n")
    .success();

  let output = Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .args(["bar", "foo"])
    .success();

  fs::remove_file(output.tempdir.path().join("foo")).unwrap();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .args(["bar", "foo"])
    .stdout("bar\n")
    .success();
}

#[test]
fn multiple_outputs() {
  let output = Test::new()
    .justfile(
      "
        set lists

        [cache(outputs = ['foo', 'baz'])]
        [script]
        bar:
          echo bar
          touch foo baz
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();

  let output = Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .success();

  fs::remove_file(output.tempdir.path().join("baz")).unwrap();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();
}

#[test]
fn output_directory_is_allowed() {
  let output = Test::new()
    .justfile(
      "
        [cache(outputs = 'foo')]
        [script]
        bar:
          echo bar
          mkdir -p foo
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .success();
}

#[test]
fn outputs_resolve_against_working_directory() {
  let output = Test::new()
    .justfile(
      "
        [cache(outputs = 'foo')]
        [working-directory('sub')]
        [script]
        bar:
          echo bar
          touch foo
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .create_dir("sub")
    .stdout("bar\n")
    .success();

  let output = Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .success();

  fs::remove_file(output.tempdir.path().join("sub/foo")).unwrap();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();
}

#[cfg(unix)]
#[test]
fn dangling_symlink_output_invalidates_cache() {
  let output = Test::new()
    .justfile(
      "
        [cache(outputs = 'link')]
        [script]
        bar:
          echo bar
          touch foo
          ln -sf foo link
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();

  let output = Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .success();

  fs::remove_file(output.tempdir.path().join("foo")).unwrap();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();
}

#[test]
fn missing_output_after_run_is_an_error() {
  let output = Test::new()
    .justfile(
      "
        [cache(outputs = 'foo')]
        [script]
        bar:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .stderr_regex(r"error: recipe `bar` failed to create cache output `foo`\n")
    .failure();

  let entries = fs::read_dir(output.tempdir.path().join(".justcache"))
    .unwrap()
    .map(|entry| entry.unwrap().path())
    .collect::<Vec<PathBuf>>();

  assert_eq!(entries.len(), 1);
  assert_eq!(fs::read_to_string(&entries[0]).unwrap(), "");
}

#[test]
fn dry_run_skips_output_checking() {
  Test::new()
    .justfile(
      "
        [cache(outputs = 'foo')]
        [script]
        bar:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .arg("--dry-run")
    .stderr("echo bar\n")
    .success();
}

#[test]
fn hit_prints_verbose_message() {
  let output = Test::new()
    .justfile(
      "
        [cache]
        [script]
        foo:
          echo bar
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .stdout("bar\n")
    .success();

  Test::with_tempdir(output.tempdir)
    .env("JUST_UNSTABLE", "1")
    .arg("--verbose")
    .stderr(
      "
        ===> running recipe `foo`...
        ===> cache hit, skipping invocation
      ",
    )
    .success();
}
