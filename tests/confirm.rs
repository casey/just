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
    .stderr("Run recipe `requires_confirmation`? echo confirmed\n")
    .stdout("confirmed\n")
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
    .stderr("Run recipe `requires_confirmation`? echo confirmed\necho confirmed2\n")
    .stdout("confirmed\nconfirmed2\n")
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
    .stderr("Run recipe `requires_confirmation`? error: Recipe `requires_confirmation` was not confirmed\n")
    .stdout("")
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
    .stderr("Run recipe `requires_confirmation`? error: Recipe `requires_confirmation` was not confirmed\n")
    .status(1)
    .run();
}

#[test]
fn confirm_recipe_with_prompt() {
  Test::new()
    .justfile(
      "
        [confirm(\"This is dangerous\")]
        requires_confirmation:
            echo confirmed
        ",
    )
    .stderr("This is dangerous - Run recipe `requires_confirmation`? echo confirmed\n")
    .stdout("confirmed\n")
    .stdin("y")
    .run();
}

#[test]
fn confirm_recipe_arg_with_prompt() {
  Test::new()
    .arg("--yes")
    .justfile(
      "
        [confirm(\"this is dangerous!\")]
        requires_confirmation:
            echo confirmed
        ",
    )
    .stderr("echo confirmed\n")
    .stdout("confirmed\n")
    .run();
}
