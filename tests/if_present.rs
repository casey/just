use super::*;

#[test]
fn without_if_present() {
  Test::new()
    .arg("execute")
    .justfile(
      "
        build:
          echo \"Building...\"
      ",
    )
    .stderr("error: Justfile does not contain recipe `execute`.\n")
    .stdout("")
    .status(1)
    .run();
}

#[test]
fn ignore_unknown_recipe() {
  Test::new()
    .args(["--if-present", "execute"])
    .justfile(
      "
        build:
          echo \"Building...\"
      ",
    )
    .stderr("")
    .stdout("")
    .status(0)
    .run();
}
