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
fn modules_are_formatted_correctly() {
  todo!()
}

#[test]
fn circular_module_imports_are_detected() {
  todo!()
}

#[test]
fn modules_conflict_with_recipes() {
  todo!()
}

#[test]
fn modules_conflict_with_aliases() {
  todo!()
}

#[test]
fn modules_conflict_with_other_modules() {
  todo!()
}

#[test]
fn default_recipe_in_submodule_must_have_no_arguments() {
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
fn modules_can_contain_other_modules() {
  todo!()
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
fn modules_can_be_in_subdirectory() {
  todo!()
}

#[test]
fn modules_in_subdirectory_can_be_named_justfile() {
  todo!()
}

#[test]
fn modules_require_unambiguous_file() {
  todo!()
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
fn list_displays_recipes_in_submodules() {
  todo!()
}

#[test]
fn module_recipes_run_in_module_directory() {
  todo!()
}

#[test]
fn modules_use_module_settings() {
  todo!()
}

#[test]
fn dotenv_files_are_loaded_on_a_per_module_basis() {
  todo!()
}
