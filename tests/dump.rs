use super::*;

#[test]
fn dump() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
        # this recipe does something
        recipe a b +d:
          @exit 100
      ",
    )
    .stdout(
      "
        # this recipe does something
        recipe a b +d:
            @exit 100
      ",
    )
    .success();
}

#[test]
fn json() {
  Test::new()
    .arg("--json")
    .justfile(
      "
        foo:
      ",
    )
    .stdout_regex(r"\{.*\}\n")
    .success();
}
