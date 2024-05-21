use super::*;

#[test]
fn output() {
  Test::new()
    .arg("--man")
    .stdout_regex("(?s).*.TH just 1.*")
    .run();
}
