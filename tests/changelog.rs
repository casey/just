use crate::common::*;

#[test]
fn print_changelog() {
  Test::new()
    .args(&["--changelog"])
    .stdout(fs::read_to_string("CHANGELOG.txt").unwrap())
    .run();
}
