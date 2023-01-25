use super::*;

#[test]
fn argument_with_different_path_prefix_is_allowed() {
  Test::new()
    .justfile("foo bar:")
    .args(["./foo", "../bar"])
    .run();
}
