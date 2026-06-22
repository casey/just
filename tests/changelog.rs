use super::*;

#[test]
fn print_changelog() {
  Test::new()
    .justfile("")
    .args(["--changelog"])
    .stdout(fs::read_to_string("CHANGELOG.md").unwrap())
    .success();
}
