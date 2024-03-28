use super::*;

#[test]
fn allow_duplicate_variables() {
  Test::new()
    .justfile(
      "
      a := 'foo'
      a := 'bar'

      set allow-duplicate-variables

      b:
        echo {{a}}
      "
    )
    .arg("b")
    .stdout("bar\n")
    .stderr("echo bar\n")
    .run();
}
