use super::*;

#[test]
fn print_cheatsheet() {
  Test::new()
    .args(["--cheatsheet"])
    .stdout(fs::read_to_string("CHEATSHEET.md").unwrap())
    .success();
}
