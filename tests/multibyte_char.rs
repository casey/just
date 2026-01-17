use super::*;

#[test]
fn bugfix() {
  Test::new().justfile("foo:\nx := '''ǩ'''").run_success();
}
