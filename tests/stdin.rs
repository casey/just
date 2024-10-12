use super::*;

#[test]
fn list_recipes_loaded_from_stdin() {
  Test::new()
    .no_justfile()
    .arg("--list")
    .stdin(
      r#"
        one:
            echo 111

        two:
            echo 222

        three:
            echo 333
    "#,
    )
    .stdout(
      r#"
        one
        two
        three
    "#,
    )
    .status(0)
    .run();
}
