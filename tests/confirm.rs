use super::*;

#[test]
fn confirm_recipe_arg() {
  Test::new()
    .arg("--yes")
    .justfile(
      "
        [confirm]
        requires_confirmation:
            echo confirmed
        ",
    )
    .stderr("echo confirmed\n")
    .stdout("confirmed\n")
    .run();
}

#[test]
fn recipe_with_confirm_recipe_dependency_arg() {
  Test::new()
    .arg("--yes")
    .justfile(
      "
        dep_confirmation: requires_confirmation
            echo confirmed2

        [confirm]
        requires_confirmation:
            echo confirmed
        ",
    )
    .stderr("echo confirmed\necho confirmed2\n")
    .stdout("confirmed\nconfirmed2\n")
    .run();
}

#[test]
fn confirm_recipe() {
  Test::new()
    .justfile(
      "
        [confirm]
        requires_confirmation:
            echo confirmed
        ",
    )
    .stderr("echo confirmed\n")
    .stdout("Confirm running requires_confirmation (y/N): confirmed\n")
    .stdin("y")
    .run();
}

#[test]
fn recipe_with_confirm_recipe_dependency() {
  Test::new()
    .justfile(
      "
        dep_confirmation: requires_confirmation
            echo confirmed2

        [confirm]
        requires_confirmation:
            echo confirmed
        ",
    )
    .stderr("echo confirmed\necho confirmed2\n")
    .stdout("Confirm running requires_confirmation (y/N): confirmed\nconfirmed2\n")
    .stdin("y")
    .run();
}

#[test]
fn do_not_confirm_recipe() {
  Test::new()
    .justfile(
      "
        [confirm]
        requires_confirmation:
            echo confirmed
        ",
    )
    .stderr("error: the recipe requires_confirmation was not confirmed, therefore has not run\n")
    .stdout("Confirm running requires_confirmation (y/N): ")
    .status(1)
    .run();
}

#[test]
fn do_not_confirm_recipe_with_confirm_recipe_dependency() {
  Test::new()
    .justfile(
      "
        dep_confirmation: requires_confirmation
            echo mistake

        [confirm]
        requires_confirmation:
            echo confirmed
        ",
    )
    .stderr("error: the recipe requires_confirmation was not confirmed, therefore has not run\n")
    .stdout("Confirm running requires_confirmation (y/N): ")
    .status(1)
    .run();
}
