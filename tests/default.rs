use super::*;

#[test]
fn default_attribute_overrides_first_recipe() {
  Test::new()
    .justfile(
      "
        foo:
          @echo FOO

        [default]
        bar:
          @echo BAR
      ",
    )
    .stdout("BAR\n")
    .success();
}

#[test]
fn default_attribute_may_only_appear_once_per_justfile() {
  Test::new()
    .justfile(
      "
        [default]
        foo:

        [default]
        bar:
      ",
    )
    .stderr(
      "
        error: Recipe `foo` has duplicate `[default]` attribute, which may only appear once per module
         ——▶ justfile:2:1
          │
        2 │ foo:
          │ ^^^
      "
    )
    .failure();
}
