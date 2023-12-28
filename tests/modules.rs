use super::*;

// todo:
// - test modules in JSON
// - update module tracking issue

#[test]
fn modules_are_unstable() {
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
fn module_recipes_can_be_run_as_subcommands() {
  Test::new()
    .tree(tree!(
    "foo.just":
    "
      foo:
        @echo FOO
    ",
    ))
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
  todo!()
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
  todo!()
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
