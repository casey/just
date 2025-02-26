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
        [confirm(\"This is dangerous - are you sure you want to run it?\")]
        requires_confirmation:
            echo confirmed
        ",
    )
    .stderr("This is dangerous - are you sure you want to run it? echo confirmed\n")
    .stdout("confirmed\n")
    .stdin("y")
    .run();
}

#[test]
fn confirm_recipe_with_prompt_too_many_args() {
  Test::new()
    .justfile(
      r#"
        [confirm("PROMPT","EXTRA")]
        requires_confirmation:
            echo confirmed
      "#,
    )
    .stderr(
      r#"
        error: Attribute `confirm` got 2 arguments but takes at most 1 argument
         ——▶ justfile:1:2
          │
        1 │ [confirm("PROMPT","EXTRA")]
          │  ^^^^^^^
      "#,
    )
    .status(1)
    .run();
}

#[test]
fn confirm_attribute_is_formatted_correctly() {
  Test::new()
    .justfile(
      "
        [confirm('prompt')]
        foo:
      ",
    )
    .arg("--dump")
    .stdout("[confirm('prompt')]\nfoo:\n")
    .run();
}
