use super::*;

#[test]
fn dont_run_duplicate_recipes() {
  Test::new()
    .justfile(
      "
      set dotenv-load # foo
      bar:
      ",
    )
    .run();
}

#[test]
fn check_match() {
  Test::new()
    .justfile(
      r#"
      val := "yep"
      computed := match val { _ => test, }
      "#,
    )
    .run();
}
