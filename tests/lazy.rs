use super::*;

#[test]
fn lazy_is_unstable() {
  Test::new()
    .justfile("set lazy\n\nfoo:\n  @echo hello")
    .arg("foo")
    .stderr_regex("error: The `lazy` setting is currently unstable\\..*")
    .status(1);
}
