use super::*;

#[test]
fn fail_on_unknown_recipe() {
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
    .args(["--allow-missing", "execute"])
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
