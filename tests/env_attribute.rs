use super::*;

#[test]
fn env_attribute_single() {
  Test::new()
    .justfile(
      r#"
[env("THE_ENV=hi")]
test *args:
  echo $THE_ENV
  echo {{args}}
"#,
    )
    .args(["test", "this", "arg", "gets", "assigned"])
    .stdout("hi\nthis arg gets assigned\n")
    .stderr("echo $THE_ENV\necho this arg gets assigned\n")
    .run();
}

#[test]
fn env_attribute_multiple() {
  Test::new()
    .justfile(
      r#"
[env("VAR1=value1", "VAR2=value2")]
test:
  echo $VAR1 $VAR2
"#,
    )
    .stdout("value1 value2\n")
    .stderr("echo $VAR1 $VAR2\n")
    .run();
}

#[test]
fn env_attribute_multiple_attributes() {
  Test::new()
    .justfile(
      r#"
[env("VAR1=value1")]
[env("VAR2=value2")]
test:
  echo $VAR1 $VAR2
"#,
    )
    .stdout("value1 value2\n")
    .stderr("echo $VAR1 $VAR2\n")
    .run();
}

#[test]
fn env_attribute_with_parameters() {
  Test::new()
    .justfile(
      r#"
[env("THE_ENV=hello")]
test param *args:
  echo $THE_ENV
  echo {{param}}
  echo {{args}}
"#,
    )
    .args(["test", "world", "these", "are", "args"])
    .stdout("hello\nworld\nthese are args\n")
    .stderr("echo $THE_ENV\necho world\necho these are args\n")
    .run();
}

#[test]
fn env_attribute_shebang() {
  Test::new()
    .justfile(
      r#"
[env("TEST_VAR=shebang_value")]
test:
  #!/bin/sh
  echo $TEST_VAR
"#,
    )
    .stdout("shebang_value\n")
    .run();
}

#[test]
fn env_attribute_with_spaces() {
  Test::new()
    .justfile(
      r#"
[env("SPACED_VAR=hello world")]
test:
  echo "$SPACED_VAR"
"#,
    )
    .stdout("hello world\n")
    .stderr("echo \"$SPACED_VAR\"\n")
    .run();
}

#[test]
fn env_attribute_override_existing() {
  Test::new()
    .justfile(
      r#"
export EXISTING_VAR := "original"

[env("EXISTING_VAR=overridden")]
test:
  echo $EXISTING_VAR
"#,
    )
    .stdout("overridden\n")
    .stderr("echo $EXISTING_VAR\n")
    .run();
}

#[test]
fn env_attribute_with_other_attributes() {
  Test::new()
    .justfile(
      r#"
[private]
[env("SECRET_VAR=secret")]
test:
  echo $SECRET_VAR
"#,
    )
    .stdout("secret\n")
    .stderr("echo $SECRET_VAR\n")
    .run();
}

#[test]
fn env_attribute_multiple_recipes_different_vars() {
  Test::new()
    .justfile(
      r#"
[env("RECIPE_A_VAR=value_a")]
recipe_a:
  echo $RECIPE_A_VAR

[env("RECIPE_B_VAR=value_b")]
recipe_b:
  echo $RECIPE_B_VAR
"#,
    )
    .args(["recipe_a"])
    .stdout("value_a\n")
    .stderr("echo $RECIPE_A_VAR\n")
    .run();
}

#[test]
fn env_attribute_multiple_recipes_same_var() {
  Test::new()
    .justfile(
      r#"
[env("SHARED_VAR=from_recipe_one")]
recipe_one:
  echo $SHARED_VAR

[env("SHARED_VAR=from_recipe_two")]
recipe_two:
  echo $SHARED_VAR
"#,
    )
    .args(["recipe_one"])
    .stdout("from_recipe_one\n")
    .stderr("echo $SHARED_VAR\n")
    .run();
}

#[test]
fn env_attribute_multiple_recipes_isolation() {
  Test::new()
    .justfile(
      r#"
[env("ISOLATED_VAR=recipe_alpha")]
alpha:
  echo $ISOLATED_VAR

beta:
  echo ${ISOLATED_VAR:-not_set}
"#,
    )
    .args(["beta"])
    .stdout("not_set\n")
    .stderr("echo ${ISOLATED_VAR:-not_set}\n")
    .run();
}

#[test]
fn env_attribute_multiple_recipes_complex_values() {
  Test::new()
    .justfile(
      r#"
[env("CONFIG_PATH=/usr/local/config", "DEBUG_LEVEL=3")]
setup:
  echo "Config: $CONFIG_PATH, Debug: $DEBUG_LEVEL"

[env("CONFIG_PATH=/home/user/config", "DEBUG_LEVEL=1")]
dev_setup:
  echo "Config: $CONFIG_PATH, Debug: $DEBUG_LEVEL"
"#,
    )
    .args(["setup"])
    .stdout("Config: /usr/local/config, Debug: 3\n")
    .stderr("echo \"Config: $CONFIG_PATH, Debug: $DEBUG_LEVEL\"\n")
    .run();
}

#[test]
fn env_attribute_multiple_recipes_with_dependencies() {
  Test::new()
    .justfile(
      r#"
[env("DEP_VAR=dependency_value")]
dependency:
  echo "Dep: $DEP_VAR"

[env("MAIN_VAR=main_value")]
main: dependency
  echo "Main: $MAIN_VAR"
"#,
    )
    .args(["main"])
    .stdout("Dep: dependency_value\nMain: main_value\n")
    .stderr("echo \"Dep: $DEP_VAR\"\necho \"Main: $MAIN_VAR\"\n")
    .run();
}

#[test]
fn env_attribute_multiple_recipes_mixed_attributes() {
  Test::new()
    .justfile(
      r#"
[private]
[env("PRIVATE_VAR=secret")]
private_task:
  echo $PRIVATE_VAR

[env("PUBLIC_VAR=public")]
public_task:
  echo $PUBLIC_VAR
"#,
    )
    .args(["public_task"])
    .stdout("public\n")
    .stderr("echo $PUBLIC_VAR\n")
    .run();
}

#[test]
fn env_attribute_multiple_recipes_numeric_and_special_chars() {
  Test::new()
    .justfile(
      r#"
[env("NUMBER_VAR=12345", "SPECIAL_VAR=hello@world#test")]
task_one:
  echo "$NUMBER_VAR:$SPECIAL_VAR"

[env("BOOLEAN_VAR=true", "PATH_VAR=/usr/bin:/bin")]
task_two:
  echo "$BOOLEAN_VAR:$PATH_VAR"
"#,
    )
    .args(["task_one"])
    .stdout("12345:hello@world#test\n")
    .stderr("echo \"$NUMBER_VAR:$SPECIAL_VAR\"\n")
    .run();
}
