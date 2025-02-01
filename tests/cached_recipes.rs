use super::*;

#[test]
fn cached_attribute_is_unstable() {
  Test::new()
    .justfile(
      "
    [cached]
    recipe:
  ",
    )
    .stderr_regex(r#"error: The \[cached\] attribute is currently unstable\..*"#)
    .status(1)
    .run();
}

#[test]
fn cached_recipe_cannot_depend_on_uncached_recipe() {
  Test::new()
    .justfile(
      "
      set unstable
      [cached]
      a: b

      b:

      ",
    )
    .stderr(
      "
      error: Cached recipe `a` depends on uncached recipe `b`
       ——▶ justfile:3:1
        │
      3 │ a: b
        │ ^
        ",
    )
    .status(1)
    .run();
}
