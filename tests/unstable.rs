use super::*;

#[test]
fn set_unstable_true_with_env_var() {
  let justfile = r#"
default:
    echo 'foo'
  "#;
  Test::new()
    .justfile(justfile)
    .args(["--dump", "--dump-format", "json"])
    .env("JUST_ALLOW_UNSTABLE", "true")
    .status(EXIT_SUCCESS)
    .stdout_regex("*")
    .run();
}

#[test]
fn set_unstable_false_with_env_var() {
  let justfile = r#"
default:
    echo 'foo'
  "#;
  Test::new()
    .justfile(justfile)
    .args(["--dump", "--dump-format", "json"])
    .env("JUST_ALLOW_UNSTABLE", "false")
    .status(EXIT_FAILURE)
    .stderr("error: The JSON dump format is currently unstable. Invoke `just` with the `--unstable` flag to enable unstable features.\n")
    .run();
}
