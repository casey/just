use super::*;

#[test]
fn confirm_recipe_arg() {
  Test::new()
    .arg("--yes")
    .justfile(
      "
        [confirm]
        @foo:
          echo FOO
      ",
    )
    .stdout("FOO\n")
    .success();
}

#[test]
fn recipe_with_confirm_recipe_dependency_arg() {
  Test::new()
    .arg("--yes")
    .justfile(
      "
        @bar: foo
          echo BAR

        [confirm]
        @foo:
          echo FOO
      ",
    )
    .stdout("FOO\nBAR\n")
    .success();
}

#[test]
fn confirm_recipe() {
  Test::new()
    .justfile(
      "
        [confirm]
        @foo:
          echo FOO
      ",
    )
    .stderr("Run recipe `foo`? ")
    .stdout("FOO\n")
    .stdin("y")
    .success();
}

#[test]
fn recipe_with_confirm_recipe_dependency() {
  Test::new()
    .justfile(
      "
        @bar: foo
          echo BAR

        [confirm]
        @foo:
          echo FOO
      ",
    )
    .stderr("Run recipe `foo`? ")
    .stdout("FOO\nBAR\n")
    .stdin("y")
    .success();
}

#[test]
fn do_not_confirm_recipe() {
  Test::new()
    .justfile(
      "
        [confirm]
        @foo:
          echo FOO
      ",
    )
    .stderr("Run recipe `foo`? error: recipe `foo` was not confirmed\n")
    .failure();
}

#[test]
fn do_not_confirm_recipe_with_confirm_recipe_dependency() {
  Test::new()
    .justfile(
      "
        bar: foo
          echo BAR

        [confirm]
        @foo:
          echo FOO
      ",
    )
    .stderr("Run recipe `foo`? error: recipe `foo` was not confirmed\n")
    .failure();
}

#[test]
fn confirm_recipe_with_prompt() {
  Test::new()
    .justfile(
      "
        [confirm('Sure?')]
        @foo:
            echo FOO
      ",
    )
    .stderr("Sure? ")
    .stdout("FOO\n")
    .stdin("y")
    .success();
}

#[test]
fn confirm_recipe_with_prompt_too_many_args() {
  Test::new()
    .justfile(
      "
        [confirm('A', 'B')]
        foo:
      ",
    )
    .stderr(
      "
        error: attribute `confirm` got 2 arguments but takes at most 1 argument
         ——▶ justfile:1:2
          │
        1 │ [confirm('A', 'B')]
          │  ^^^^^^^
      ",
    )
    .failure();
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
    .stdout(
      "
        [confirm('prompt')]
        foo:
      ",
    )
    .success();
}

#[test]
fn confirm_with_expression() {
  Test::new()
    .justfile(
      "
        target := 'production'

        [confirm('Deploy to ' + target + '?')]
        deploy:
            echo deployed
      ",
    )
    .stdin("y")
    .stderr("Deploy to production? echo deployed\n")
    .stdout("deployed\n")
    .success();
}

#[test]
fn confirm_with_recipe_parameter() {
  Test::new()
    .justfile(
      "
        [confirm('Deploy to ' + target + '?')]
        deploy target:
            echo 'deployed to {{target}}'
      ",
    )
    .args(["deploy", "staging"])
    .stdin("y")
    .stderr("Deploy to staging? echo 'deployed to staging'\n")
    .stdout("deployed to staging\n")
    .success();
}

#[test]
fn confirm_expression_dump() {
  Test::new()
    .justfile(
      "
        target := 'production'

        [confirm('Deploy to ' + target + '?')]
        deploy:
            echo deployed
      ",
    )
    .arg("--dump")
    .stdout(
      "
        target := 'production'

        [confirm('Deploy to ' + target + '?')]
        deploy:
            echo deployed
      ",
    )
    .success();
}
