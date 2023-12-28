use super::*;

#[test]
fn modules_are_unstable() {
  Test::new()
    .justfile(
      "
        mod foo
      ",
    )
    .arg("foo")
    .arg("foo")
    .stderr(
      "error: Modules are currently unstable. \
      Invoke `just` with the `--unstable` flag to enable unstable features.\n",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn default_recipe_in_submodule_must_have_no_arguments() {
  Test::new()
    .write("foo.just", "foo bar:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .stderr("error: Recipe `foo` cannot be used as default recipe since it requires at least 1 argument.\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn module_recipes_can_be_run_as_subcommands() {
  Test::new()
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .run();
}

#[test]
fn assignments_are_evaluated_in_modules() {
  Test::new()
    .write("foo.just", "bar := 'CHILD'\nfoo:\n @echo {{bar}}")
    .justfile(
      "
        mod foo
        bar := 'PARENT'
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .arg("foo")
    .stdout("CHILD\n")
    .run();
}

#[test]
fn module_subcommand_runs_default_recipe() {
  Test::new()
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .stdout("FOO\n")
    .run();
}

#[test]
fn modules_can_contain_other_modules() {
  Test::new()
    .write("bar.just", "baz:\n @echo BAZ")
    .write("foo.just", "mod bar")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .arg("bar")
    .arg("baz")
    .stdout("BAZ\n")
    .run();
}

#[test]
fn circular_module_imports_are_detected() {
  Test::new()
    .write("bar.just", "mod foo")
    .write("foo.just", "mod bar")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .arg("bar")
    .arg("baz")
    .stderr_regex(path_for_regex(
      "error: Import `.*/foo.just` in `.*/bar.just` is circular\n",
    ))
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn modules_use_module_settings() {
  Test::new()
    .write(
      "foo.just",
      "set allow-duplicate-recipes\nfoo:\nfoo:\n @echo FOO\n",
    )
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .run();

  Test::new()
    .write("foo.just", "\nfoo:\nfoo:\n @echo FOO\n")
    .justfile(
      "
        mod foo

        set allow-duplicate-recipes
      ",
    )
    .test_round_trip(false)
    .status(EXIT_FAILURE)
    .arg("--unstable")
    .arg("foo")
    .arg("foo")
    .stderr(
      "
      error: Recipe `foo` first defined on line 2 is redefined on line 3
       --> foo.just:3:1
        |
      3 | foo:
        | ^^^
    ",
    )
    .run();
}

#[test]
fn modules_conflict_with_recipes() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        mod foo
        foo:
      ",
    )
    .stderr(
      "
      error: Module `foo` defined on line 1 is redefined as a recipe on line 2
       --> justfile:2:1
        |
      2 | foo:
        | ^^^
    ",
    )
    .test_round_trip(false)
    .status(EXIT_FAILURE)
    .arg("--unstable")
    .run();
}

#[test]
fn modules_conflict_with_aliases() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        mod foo
        bar:
        alias foo := bar
      ",
    )
    .stderr(
      "
      error: Module `foo` defined on line 1 is redefined as an alias on line 3
       --> justfile:3:7
        |
      3 | alias foo := bar
        |       ^^^
    ",
    )
    .test_round_trip(false)
    .status(EXIT_FAILURE)
    .arg("--unstable")
    .run();
}

#[test]
fn modules_conflict_with_other_modules() {
  Test::new()
    .write("foo.just", "")
    .justfile(
      "
        mod foo
        mod foo

        bar:
      ",
    )
    .test_round_trip(false)
    .status(EXIT_FAILURE)
    .stderr(
      "
      error: Module `foo` first defined on line 1 is redefined on line 2
       --> justfile:2:5
        |
      2 | mod foo
        |     ^^^
    ",
    )
    .arg("--unstable")
    .run();
}

#[test]
fn modules_are_dumped_correctly() {
  Test::new()
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("--dump")
    .stdout("mod foo\n")
    .run();
}

#[test]
fn modules_can_be_in_subdirectory() {
  Test::new()
    .write("foo/mod.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .run();
}

#[test]
fn modules_in_subdirectory_can_be_named_justfile() {
  Test::new()
    .write("foo/justfile", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .run();
}

#[test]
fn modules_in_subdirectory_can_be_named_justfile_with_any_case() {
  Test::new()
    .write("foo/JUSTFILE", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .run();
}

#[test]
fn modules_in_subdirectory_can_have_leading_dot() {
  Test::new()
    .write("foo/.justfile", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .arg("foo")
    .arg("foo")
    .stdout("FOO\n")
    .run();
}

#[test]
fn modules_require_unambiguous_file() {
  Test::new()
    .write("foo/justfile", "foo:\n @echo FOO")
    .write("foo.just", "foo:\n @echo FOO")
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .status(EXIT_FAILURE)
    .stderr(
      "
      error: Found multiple source files for module `foo`: `foo.just` and `foo/justfile`
       --> justfile:1:5
        |
      1 | mod foo
        |     ^^^
      ",
    )
    .run();
}

#[test]
fn missing_module_file_error() {
  Test::new()
    .justfile(
      "
        mod foo
      ",
    )
    .test_round_trip(false)
    .arg("--unstable")
    .status(EXIT_FAILURE)
    .stderr(
      "
      error: Could not find source file for module `foo`.
       --> justfile:1:5
        |
      1 | mod foo
        |     ^^^
      ",
    )
    .run();
}

#[test]
fn list_displays_recipes_in_submodules() {
  todo!()
}

#[test]
fn module_recipes_run_in_module_directory() {
  todo!()
}

#[test]
fn dotenv_files_are_loaded_on_a_per_module_basis() {
  todo!()
}

#[test]
fn justfile_function_returns_submodule_path() {
  todo!()
}

#[test]
fn justfile_directory_function_returns_submodule_directory() {
  todo!()
}
