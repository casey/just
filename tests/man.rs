use super::*;

#[test]
fn output() {
  Test::new()
    .justfile("")
    .arg("--man")
    .stdout_regex("(?s).*.TH just 1.*")
    .success();
}
