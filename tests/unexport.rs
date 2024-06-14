use super::*;

#[test]
fn unexport_environment_variable_linewise() {
  Test::new()
    .justfile(
      "
     unexport JUST_TEST_VARIABLE

     @recipe:
         echo ${JUST_TEST_VARIABLE:-unset}
      ",
    )
    .env("JUST_TEST_VARIABLE", "foo")
    .stdout("unset\n")
    .run();
}

#[test]
fn unexport_environment_variable_shebang() {
  Test::new()
    .justfile(
      "
     unexport JUST_TEST_VARIABLE

     recipe:
         #!/usr/bin/env bash
         echo ${JUST_TEST_VARIABLE:-unset}
      ",
    )
    .env("JUST_TEST_VARIABLE", "foo")
    .stdout("unset\n")
    .run();
}

#[test]
fn duplicate_unexport_fails() {
  Test::new()
    .justfile(
      "
     unexport JUST_TEST_VARIABLE

     recipe:
         echo \"variable: $JUST_TEST_VARIABLE\"

     unexport JUST_TEST_VARIABLE
      ",
    )
    .env("JUST_TEST_VARIABLE", "foo")
    .stderr(
      "
        error: Variable `JUST_TEST_VARIABLE` is unexported multiple times
         ——▶ justfile:6:10
          │
        6 │ unexport JUST_TEST_VARIABLE
          │          ^^^^^^^^^^^^^^^^^^
      ",
    )
    .status(1)
    .run();
}

#[test]
fn export_unexport_conflict() {
  Test::new()
    .justfile(
      "
     unexport JUST_TEST_VARIABLE

     recipe:
         echo variable: $JUST_TEST_VARIABLE

     export JUST_TEST_VARIABLE := 'foo'
      ",
    )
    .stderr(
      "
        error: Variable JUST_TEST_VARIABLE is both exported and unexported
         ——▶ justfile:6:8
          │
        6 │ export JUST_TEST_VARIABLE := 'foo'
          │        ^^^^^^^^^^^^^^^^^^
      ",
    )
    .status(1)
    .run();
}

#[test]
fn unexport_doesnt_override_local_recipe_export() {
  Test::new()
    .justfile(
      "
     unexport JUST_TEST_VARIABLE

     recipe $JUST_TEST_VARIABLE:
         @echo \"variable: $JUST_TEST_VARIABLE\"
      ",
    )
    .args(["recipe", "value"])
    .stdout("variable: value\n")
    .run();
}

#[test]
fn unexport_does_not_conflict_with_recipe_syntax() {
  Test::new()
    .justfile(
      "
        unexport foo:
          @echo {{foo}}
      ",
    )
    .args(["unexport", "bar"])
    .stdout("bar\n")
    .run();
}

#[test]
fn unexport_does_not_conflict_with_assignment_syntax() {
  Test::new()
    .justfile("unexport := 'foo'")
    .args(["--evaluate", "unexport"])
    .stdout("foo")
    .run();
}
