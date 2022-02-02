use crate::common::*;

#[test]
fn bash() {
  Test::new()
    .invocation("--completions bash")
    .stdout_regex("_just().*")
    .run();
}

#[test]
fn zsh() {
  Test::new()
    .invocation("--completions zsh")
    .stdout_regex("_just().*")
    .run();
}
