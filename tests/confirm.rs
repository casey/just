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
    .success();
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
    .failure();
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
    .success();
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
    .success();
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
    .success();
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
    .failure();
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
    .failure();
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
    .success();
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
    .success();
}

#[test]
fn confirm_with_variable() {
  Test::new()
    .justfile(
      r#"
        target := "production"

        [confirm(target)]
        deploy:
            echo deployed
      "#,
    )
    .stdin("y")
    .stderr("production echo deployed\n")
    .stdout("deployed\n")
    .success();
}

#[test]
fn confirm_with_concatenation() {
  Test::new()
    .justfile(
      r#"
        target := "production"

        [confirm("Deploy to " + target + "?")]
        deploy:
            echo deployed
      "#,
    )
    .stdin("y")
    .stderr("Deploy to production? echo deployed\n")
    .stdout("deployed\n")
    .success();
}

#[test]
fn confirm_with_expression_and_yes_flag() {
  Test::new()
    .justfile(
      r#"
        target := "production"

        [confirm("Deploy to " + target + "?")]
        deploy:
            echo deployed
      "#,
    )
    .arg("--yes")
    .stderr("echo deployed\n")
    .stdout("deployed\n")
    .success();
}

#[test]
fn confirm_with_recipe_parameter() {
  Test::new()
    .justfile(
      r#"
        [confirm("Deploy to " + target + "?")]
        deploy target:
            echo "deployed to {{target}}"
      "#,
    )
    .args(["deploy", "staging"])
    .stdin("y")
    .stderr("Deploy to staging? echo \"deployed to staging\"\n")
    .stdout("deployed to staging\n")
    .success();
}

#[test]
fn confirm_with_function_call() {
  Test::new()
    .justfile(
      r#"
        target := "production"

        [confirm("Deploy to " + uppercase(target) + "?")]
        deploy:
            echo deployed
      "#,
    )
    .stdin("y")
    .stderr("Deploy to PRODUCTION? echo deployed\n")
    .stdout("deployed\n")
    .success();
}

#[test]
fn confirm_expression_dump() {
  Test::new()
    .justfile(
      r#"
        target := "production"

        [confirm("Deploy to " + target + "?")]
        deploy:
            echo deployed
      "#,
    )
    .arg("--dump")
    .stdout(
      "target := \"production\"\n\n[confirm(\"Deploy to \" + target + \"?\")]\ndeploy:\n    echo deployed\n",
    )
    .success();
}
